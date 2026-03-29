use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex as StdMutex, RwLock};
use std::time::Duration;

use chrono::Utc;
use dashmap::DashMap;
use entity::workspace::documents;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tauri::{AppHandle, Emitter, Manager};
use tokio::task::JoinHandle;
use uuid::Uuid;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, ReadTxn, StateVector, Transact, Update};

use crate::error::{AppError, AppResult};
use crate::workspace::state::{DbState, WorkspaceState};

const FRAGMENT_NAME: &str = "document-store";
const DEBOUNCE_MS: u64 = 1500;
const POLL_INTERVAL_MS: u64 = 500;

static MD_IMG_RE: LazyLock<regex_lite::Regex> =
    LazyLock::new(|| regex_lite::Regex::new(r"(!\[[^\]]*]\()([^)]+)(\))").unwrap());
static HTML_SRC_RE: LazyLock<regex_lite::Regex> =
    LazyLock::new(|| regex_lite::Regex::new(r#"(src=")([^"]+)(")"#).unwrap());

// ── Return type for open_ydoc ─────────────────────────────────

/// Returned by `open_ydoc` so the frontend knows the stable UUID.
#[derive(serde::Serialize)]
pub struct OpenDocResult {
    /// Stable document UUID (database primary key).
    pub doc_uuid: Uuid,
    /// Full Y.Doc state as binary v1 update.
    pub yjs_state: Vec<u8>,
}

// ── DocEntry ──────────────────────────────────────────────────

struct DocSnapshot {
    yjs_state: Vec<u8>,
    state_vector: Vec<u8>,
    markdown: String,
}

/// Per-document Y.Doc entry held in memory.
///
/// `Doc` is wrapped in `StdMutex` because yrs `Doc` is `Send` but not `Sync`.
/// `rel_path` uses `RwLock` so rename can update it without rebuilding the entry.
struct DocEntry {
    doc: StdMutex<Doc>,
    rel_path: RwLock<String>,
    workspace_path: String,
    asset_url_prefix: String,
    doc_db_id: Uuid,
    dirty: AtomicBool,
    last_update_ms: AtomicU64,
    writeback_handle: tokio::sync::Mutex<Option<JoinHandle<()>>>,
}

impl DocEntry {
    fn apply_update(&self, update: &[u8]) -> AppResult<()> {
        let doc = self.doc.lock().expect("doc lock poisoned");
        apply_binary_update(&doc, update)
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Release);
        self.last_update_ms.store(now_ms(), Ordering::Release);
    }

    fn encode_full_state(&self) -> Vec<u8> {
        let doc = self.doc.lock().expect("doc lock poisoned");
        let txn = doc.transact();
        let state = txn.encode_state_as_update_v1(&StateVector::default());
        drop(txn);
        state
    }

    fn snapshot(&self) -> AppResult<DocSnapshot> {
        let doc = self.doc.lock().expect("doc lock poisoned");
        let txn = doc.transact();
        let yjs_state = txn.encode_state_as_update_v1(&StateVector::default());
        let state_vector = txn.state_vector().encode_v1();
        drop(txn);
        let markdown = yrs_blocknote::doc_to_markdown(&doc, FRAGMENT_NAME)
            .map_err(|e| AppError::Yjs(format!("doc_to_markdown: {e}")))?;
        Ok(DocSnapshot {
            yjs_state,
            state_vector,
            markdown,
        })
    }

    fn rel_path(&self) -> String {
        self.rel_path
            .read()
            .expect("rel_path lock poisoned")
            .clone()
    }

    fn set_rel_path(&self, new_path: &str) {
        *self.rel_path.write().expect("rel_path lock poisoned") = new_path.to_owned();
    }
}

// ── YDocManager ───────────────────────────────────────────────

/// Manages in-memory Y.Doc instances keyed by `(window_label, doc_uuid)`.
pub struct YDocManager {
    docs: DashMap<(String, Uuid), Arc<DocEntry>>,
}

impl YDocManager {
    pub fn new() -> Self {
        Self {
            docs: DashMap::new(),
        }
    }

    /// Open a document: look up or create UUID, load Y.Doc, return UUID + state.
    pub async fn open_doc(
        &self,
        app: &AppHandle,
        label: &str,
        rel_path: &str,
        _workspace_id: Uuid,
        asset_url_prefix: &str,
    ) -> AppResult<OpenDocResult> {
        let ws_state = app.state::<WorkspaceState>();
        let ws_path = ws_state.workspace_path_for(label).await?;
        let db_state = app.state::<DbState>();

        // Look up document by rel_path → get stable UUID
        let guard = db_state.workspace_db_for(label).await?;
        let db_doc = documents::Entity::find()
            .filter(documents::Column::RelPath.eq(rel_path))
            .one(guard.conn())
            .await?;
        let doc_uuid = db_doc.as_ref().map(|m| m.id).unwrap_or_else(Uuid::now_v7);
        drop(guard);

        let key = (label.to_owned(), doc_uuid);

        // If already open, return current state
        if let Some(entry) = self.docs.get(&key) {
            return Ok(OpenDocResult {
                doc_uuid,
                yjs_state: entry.encode_full_state(),
            });
        }

        let doc = Doc::new();
        // Pre-register the fragment name so it exists even after applying updates
        doc.get_or_insert_xml_fragment(FRAGMENT_NAME);

        let has_yjs_state = db_doc
            .as_ref()
            .and_then(|m| m.yjs_state.as_ref())
            .map(|yjs_state| apply_binary_update(&doc, yjs_state))
            .transpose()?
            .is_some();

        if !has_yjs_state {
            let full_path = PathBuf::from(&ws_path).join(rel_path);
            let md =
                tokio::task::spawn_blocking(move || match std::fs::read_to_string(&full_path) {
                    Ok(content) => Ok(content),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
                    Err(e) => Err(AppError::Io(e)),
                })
                .await
                .map_err(|e| AppError::Yjs(e.to_string()))??;

            let md = relative_to_asset_url(&md, asset_url_prefix);
            let init_doc = yrs_blocknote::markdown_to_doc(&md, FRAGMENT_NAME);
            let init_state = init_doc
                .transact()
                .encode_state_as_update_v1(&StateVector::default());
            apply_binary_update(&doc, &init_state)?;
        }

        let state = doc
            .transact()
            .encode_state_as_update_v1(&StateVector::default());

        let entry = Arc::new(DocEntry {
            doc: StdMutex::new(doc),
            rel_path: RwLock::new(rel_path.to_owned()),
            workspace_path: ws_path,
            asset_url_prefix: asset_url_prefix.to_owned(),
            doc_db_id: doc_uuid,
            dirty: AtomicBool::new(false),
            last_update_ms: AtomicU64::new(now_ms()),
            writeback_handle: tokio::sync::Mutex::new(None),
        });

        if !has_yjs_state {
            let snapshot = entry.snapshot()?;
            Self::persist_snapshot(app, label, &entry, snapshot).await?;
        }

        let handle =
            spawn_writeback_task(app.clone(), label.to_owned(), doc_uuid, Arc::clone(&entry));
        *entry.writeback_handle.lock().await = Some(handle);

        self.docs.insert(key, entry);
        Ok(OpenDocResult {
            doc_uuid,
            yjs_state: state,
        })
    }

    /// Apply an incremental Y.Doc update from the frontend.
    pub fn apply_update(&self, label: &str, doc_uuid: Uuid, update: &[u8]) -> AppResult<()> {
        let key = (label.to_owned(), doc_uuid);
        let entry = self
            .docs
            .get(&key)
            .ok_or_else(|| AppError::DocNotOpen(doc_uuid.to_string()))?;

        entry.apply_update(update)?;
        entry.mark_dirty();
        Ok(())
    }

    /// Close a document: flush if dirty, stop writeback task, remove from map.
    pub async fn close_doc(&self, app: &AppHandle, label: &str, doc_uuid: Uuid) -> AppResult<()> {
        let key = (label.to_owned(), doc_uuid);
        let Some((_, entry)) = self.docs.remove(&key) else {
            return Ok(());
        };

        if let Some(handle) = entry.writeback_handle.lock().await.take() {
            handle.abort();
        }

        if entry.dirty.load(Ordering::Acquire) {
            let snapshot = entry.snapshot()?;
            Self::persist_snapshot(app, label, &entry, snapshot).await?;
        }

        Ok(())
    }

    /// Rename a document's rel_path in-place (UUID stays the same, no close/reopen).
    pub fn rename_doc(&self, label: &str, doc_uuid: Uuid, new_rel_path: &str) {
        let key = (label.to_owned(), doc_uuid);
        if let Some(entry) = self.docs.get(&key) {
            entry.set_rel_path(new_rel_path);
        }
    }

    /// Close all docs for a window (called on window destroy).
    pub async fn close_all_for_window(&self, app: &AppHandle, label: &str) {
        let keys: Vec<(String, Uuid)> = self
            .docs
            .iter()
            .filter(|e| e.key().0 == label)
            .map(|e| e.key().clone())
            .collect();

        for key in keys {
            if let Err(e) = self.close_doc(app, &key.0, key.1).await {
                tracing::warn!("Failed to close doc {} on window cleanup: {e}", key.1);
            }
        }
    }

    /// Persist a snapshot to DB + write .md file.
    async fn persist_snapshot(
        app: &AppHandle,
        label: &str,
        entry: &DocEntry,
        snapshot: DocSnapshot,
    ) -> AppResult<()> {
        let md = asset_url_to_relative(&snapshot.markdown, &entry.asset_url_prefix);
        let rel = entry.rel_path();

        let ws_path = PathBuf::from(&entry.workspace_path);
        let file_hash = tokio::task::spawn_blocking(move || -> AppResult<Vec<u8>> {
            let full_path = ws_path.join(&rel);
            std::fs::write(&full_path, &md)?;
            let hash = blake3::hash(md.as_bytes());
            Ok(hash.as_bytes().to_vec())
        })
        .await
        .map_err(|e| AppError::Yjs(e.to_string()))??;

        let now = Utc::now().timestamp();
        let db_state = app.state::<DbState>();
        let guard = db_state.workspace_db_for(label).await?;

        if let Some(existing) = documents::Entity::find_by_id(entry.doc_db_id)
            .one(guard.conn())
            .await?
        {
            let mut model: documents::ActiveModel = existing.into();
            model.yjs_state = Set(Some(snapshot.yjs_state));
            model.state_vector = Set(Some(snapshot.state_vector));
            model.file_hash = Set(Some(file_hash));
            model.rel_path = Set(entry.rel_path());
            model.updated_at = Set(now);
            model.update(guard.conn()).await?;
        }

        Ok(())
    }
}

// ── Free helper functions ──

fn apply_binary_update(doc: &Doc, data: &[u8]) -> AppResult<()> {
    let update =
        Update::decode_v1(data).map_err(|e| AppError::Yjs(format!("decode update: {e}")))?;
    let mut txn = doc.transact_mut();
    txn.apply_update(update)
        .map_err(|e| AppError::Yjs(format!("apply update: {e}")))?;
    Ok(())
}

fn spawn_writeback_task(
    app: AppHandle,
    label: String,
    doc_uuid: Uuid,
    entry: Arc<DocEntry>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(POLL_INTERVAL_MS));
        loop {
            interval.tick().await;

            if !entry.dirty.load(Ordering::Acquire) {
                continue;
            }

            let last = entry.last_update_ms.load(Ordering::Acquire);
            if now_ms() - last < DEBOUNCE_MS {
                continue;
            }

            let snapshot = match entry.snapshot() {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("snapshot failed for doc {doc_uuid}: {e}");
                    continue;
                }
            };

            entry.dirty.store(false, Ordering::Release);

            if let Err(e) = YDocManager::persist_snapshot(&app, &label, &entry, snapshot).await {
                tracing::warn!("writeback failed for doc {doc_uuid}: {e}");
                entry.dirty.store(true, Ordering::Release);
                continue;
            }

            let _ = app.emit_to(
                &label,
                "yjs:flushed",
                serde_json::json!({ "docUuid": doc_uuid.to_string() }),
            );
        }
    })
}

fn relative_to_asset_url(md: &str, asset_url_prefix: &str) -> String {
    if md.is_empty() || asset_url_prefix.is_empty() {
        return md.to_owned();
    }
    let prefix = normalize_prefix(asset_url_prefix);
    let result = prefix_relative_urls(md, &MD_IMG_RE, &prefix);
    prefix_relative_urls(&result, &HTML_SRC_RE, &prefix)
}

fn prefix_relative_urls(text: &str, re: &regex_lite::Regex, prefix: &str) -> String {
    re.replace_all(text, |caps: &regex_lite::Captures| {
        let url = &caps[2];
        if is_absolute_or_special_url(url) {
            return caps[0].to_string();
        }
        format!("{}{prefix}{url}{}", &caps[1], &caps[3])
    })
    .into_owned()
}

fn asset_url_to_relative(md: &str, asset_url_prefix: &str) -> String {
    if md.is_empty() || asset_url_prefix.is_empty() {
        return md.to_owned();
    }
    let prefix = normalize_prefix(asset_url_prefix);
    md.replace(&prefix, "")
}

fn normalize_prefix(prefix: &str) -> String {
    if prefix.ends_with('/') {
        prefix.to_owned()
    } else {
        format!("{prefix}/")
    }
}

fn is_absolute_or_special_url(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("data:")
        || url.starts_with("blob:")
        || url.starts_with('/')
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time after epoch")
        .as_millis() as u64
}
