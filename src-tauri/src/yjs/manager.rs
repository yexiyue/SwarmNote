use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, RwLock};
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
use yrs::{Doc, OffsetKind, Options, ReadTxn, StateVector, Transact, Update};

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
/// `Doc` uses `tokio::sync::Mutex` (not `std::sync::Mutex`) so that:
/// - The lock never poisons on panic — other operations remain unaffected.
/// - All callers are already in async context, so `.await` is natural.
struct DocEntry {
    doc: tokio::sync::Mutex<Doc>,
    rel_path: RwLock<String>,
    workspace_path: String,
    asset_url_prefix: String,
    doc_db_id: Uuid,
    dirty: AtomicBool,
    last_update_ms: AtomicU64,
    writeback_handle: tokio::sync::Mutex<Option<JoinHandle<()>>>,
    /// Blake3 hash of the last .md file we wrote, for self-write detection.
    file_hash: RwLock<Vec<u8>>,
    /// Timestamp (ms) of the last .md file we wrote (reserved for future optimization).
    _last_write_ms: AtomicU64,
    /// Mutual exclusion between writeback and external reload.
    reload_lock: tokio::sync::Mutex<()>,
}

impl DocEntry {
    async fn apply_update(&self, update: &[u8]) -> AppResult<()> {
        let doc = self.doc.lock().await;
        apply_binary_update(&doc, update)
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Release);
        self.last_update_ms.store(now_ms(), Ordering::Release);
    }

    async fn encode_full_state(&self) -> Vec<u8> {
        let doc = self.doc.lock().await;
        let txn = doc.transact();
        txn.encode_state_as_update_v1(&StateVector::default())
    }

    async fn snapshot(&self) -> AppResult<DocSnapshot> {
        let doc = self.doc.lock().await;
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

    fn file_hash(&self) -> Vec<u8> {
        self.file_hash
            .read()
            .expect("file_hash lock poisoned")
            .clone()
    }

    fn set_file_hash(&self, hash: &[u8]) {
        *self.file_hash.write().expect("file_hash lock poisoned") = hash.to_vec();
    }

    /// Replace the Y.Doc content from markdown and return the incremental diff.
    ///
    /// Must be called while holding `reload_lock`.
    async fn replace_content_from_md(&self, md: &str) -> Vec<u8> {
        let doc = self.doc.lock().await;
        let sv_before = {
            let txn = doc.transact();
            txn.state_vector()
        };
        yrs_blocknote::replace_doc_content(&doc, md, FRAGMENT_NAME);
        let txn = doc.transact();
        txn.encode_state_as_update_v1(&sv_before)
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

    // ── Open / Close / Update ────────────────────────────────

    /// Open a document: look up or create UUID, load Y.Doc, return UUID + state.
    ///
    /// If the document has no DB record (e.g. externally copied .md file),
    /// a new record is inserted automatically (upsert semantics).
    pub async fn open_doc(
        &self,
        app: &AppHandle,
        label: &str,
        rel_path: &str,
        workspace_id: Uuid,
        asset_url_prefix: &str,
    ) -> AppResult<OpenDocResult> {
        let ws_state = app.state::<WorkspaceState>();
        let ws_path = ws_state.workspace_path_for(label).await?;
        let db_state = app.state::<DbState>();

        // If already open for this rel_path, return current state (fast path).
        // This also prevents duplicate entries when React strict mode remounts.
        if let Some((existing_uuid, entry)) = self.find_entry_by_rel_path(label, rel_path) {
            return Ok(OpenDocResult {
                doc_uuid: existing_uuid,
                yjs_state: entry.encode_full_state().await,
            });
        }

        // Look up document by rel_path → get stable UUID.
        // If no DB record exists, INSERT one immediately (upsert).
        let guard = db_state.workspace_db_for(label).await?;
        let db_doc = documents::Entity::find()
            .filter(documents::Column::RelPath.eq(rel_path))
            .one(guard.conn())
            .await?;

        let doc_uuid = match &db_doc {
            Some(model) => model.id,
            None => {
                let new_id = Uuid::now_v7();
                let identity = app.state::<crate::identity::IdentityState>();
                let now = chrono::Utc::now().timestamp();
                let title = rel_path
                    .rsplit('/')
                    .next()
                    .unwrap_or(rel_path)
                    .trim_end_matches(".md")
                    .to_owned();

                let model = documents::ActiveModel {
                    id: sea_orm::Set(new_id),
                    workspace_id: sea_orm::Set(workspace_id),
                    folder_id: sea_orm::Set(None),
                    title: sea_orm::Set(title),
                    rel_path: sea_orm::Set(rel_path.to_owned()),
                    lamport_clock: sea_orm::Set(0),
                    created_by: sea_orm::Set(identity.peer_id()?),
                    created_at: sea_orm::Set(now),
                    updated_at: sea_orm::Set(now),
                    ..Default::default()
                };
                model.insert(guard.conn()).await?;
                tracing::info!("Auto-created DB record for {rel_path} → {new_id}");
                new_id
            }
        };
        drop(guard);

        // Use Utf16 offset kind to match frontend JS yjs — Bytes (yrs default) causes
        // panics in block_offset when processing CJK characters.
        let doc = Doc::with_options(Options {
            offset_kind: OffsetKind::Utf16,
            ..Options::default()
        });
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

        let init_hash = db_doc
            .as_ref()
            .and_then(|m| m.file_hash.clone())
            .unwrap_or_default();

        let entry = Arc::new(DocEntry {
            doc: tokio::sync::Mutex::new(doc),
            rel_path: RwLock::new(rel_path.to_owned()),
            workspace_path: ws_path,
            asset_url_prefix: asset_url_prefix.to_owned(),
            doc_db_id: doc_uuid,
            dirty: AtomicBool::new(false),
            last_update_ms: AtomicU64::new(now_ms()),
            writeback_handle: tokio::sync::Mutex::new(None),
            file_hash: RwLock::new(init_hash),
            _last_write_ms: AtomicU64::new(0),
            reload_lock: tokio::sync::Mutex::new(()),
        });

        if !has_yjs_state {
            let snapshot = entry.snapshot().await?;
            Self::persist_snapshot(app, label, &entry, snapshot).await?;
        }

        let handle =
            spawn_writeback_task(app.clone(), label.to_owned(), doc_uuid, Arc::clone(&entry));
        *entry.writeback_handle.lock().await = Some(handle);

        self.docs.insert((label.to_owned(), doc_uuid), entry);
        Ok(OpenDocResult {
            doc_uuid,
            yjs_state: state,
        })
    }

    /// Apply an incremental Y.Doc update from the frontend.
    pub async fn apply_update(&self, label: &str, doc_uuid: Uuid, update: &[u8]) -> AppResult<()> {
        let key = (label.to_owned(), doc_uuid);
        let entry = self
            .docs
            .get(&key)
            .ok_or_else(|| AppError::DocNotOpen(doc_uuid.to_string()))?;

        entry.apply_update(update).await?;
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
            let snapshot = entry.snapshot().await?;
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

    // ── External file reload ─────────────────────────────────

    /// Find an open DocEntry by its workspace-relative path.
    fn find_entry_by_rel_path(&self, label: &str, rel_path: &str) -> Option<(Uuid, Arc<DocEntry>)> {
        self.docs
            .iter()
            .find(|e| e.key().0 == label && e.rel_path() == rel_path)
            .map(|e| (e.key().1, Arc::clone(e.value())))
    }

    /// Called by the file watcher when a .md file changes on disk.
    ///
    /// Compares blake3 hashes to skip self-writes, then either silently reloads
    /// (not dirty) or notifies the frontend of a conflict (dirty).
    pub async fn reload_from_file(
        &self,
        app: &AppHandle,
        label: &str,
        rel_path: &str,
    ) -> AppResult<()> {
        let Some((doc_uuid, entry)) = self.find_entry_by_rel_path(label, rel_path) else {
            return Ok(()); // not open — next open_ydoc will load from disk
        };

        // Read the new file content and compute hash
        let full_path = PathBuf::from(&entry.workspace_path).join(rel_path);
        let new_content = match tokio::fs::read_to_string(&full_path).await {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(AppError::Io(e)),
        };
        let new_hash = blake3::hash(new_content.as_bytes());

        // Self-write detection: if hash matches what we last wrote, skip
        if new_hash.as_bytes() == entry.file_hash().as_slice() {
            return Ok(());
        }

        if entry.dirty.load(Ordering::Acquire) {
            // Document has unsaved edits — ask the user
            tracing::info!("External conflict for {rel_path} (doc {doc_uuid})");
            let _ = app.emit_to(
                label,
                "yjs:external-conflict",
                serde_json::json!({
                    "docUuid": doc_uuid.to_string(),
                    "relPath": rel_path,
                }),
            );
            return Ok(());
        }

        // Silent reload
        tracing::info!("External reload for {rel_path} (doc {doc_uuid})");
        self.do_reload(app, label, doc_uuid, &entry, &new_content)
            .await
    }

    /// Called after the user confirms reload in the conflict dialog.
    pub async fn reload_confirmed(
        &self,
        app: &AppHandle,
        label: &str,
        doc_uuid: Uuid,
    ) -> AppResult<()> {
        let entry = {
            let key = (label.to_owned(), doc_uuid);
            let guard = self
                .docs
                .get(&key)
                .ok_or_else(|| AppError::DocNotOpen(doc_uuid.to_string()))?;
            Arc::clone(guard.value())
            // `guard` (DashMap Ref) dropped here at scope exit
        };

        // Re-read the file (it may have changed again since the conflict was reported)
        let full_path = PathBuf::from(&entry.workspace_path).join(entry.rel_path());
        let content = tokio::fs::read_to_string(&full_path)
            .await
            .map_err(AppError::Io)?;

        entry.dirty.store(false, Ordering::Release);
        self.do_reload(app, label, doc_uuid, &entry, &content).await
    }

    /// Shared reload logic: replace Y.Doc content, persist, and notify frontend.
    async fn do_reload(
        &self,
        app: &AppHandle,
        label: &str,
        doc_uuid: Uuid,
        entry: &DocEntry,
        raw_md: &str,
    ) -> AppResult<()> {
        let _guard = entry.reload_lock.lock().await;

        let md = relative_to_asset_url(raw_md, &entry.asset_url_prefix);
        let diff = entry.replace_content_from_md(&md).await;

        // Persist the new state
        let snapshot = entry.snapshot().await?;
        Self::persist_snapshot(app, label, entry, snapshot).await?;

        // Notify the frontend with the incremental diff
        let _ = app.emit_to(
            label,
            "yjs:external-update",
            serde_json::json!({
                "docUuid": doc_uuid.to_string(),
                "update": diff,
            }),
        );

        Ok(())
    }

    // ── Persistence ──────────────────────────────────────────

    /// Persist a snapshot to DB + write .md file.
    ///
    /// After writing the file, immediately updates the in-memory `file_hash`
    /// and `file_hash` so the file watcher can distinguish self-writes.
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

        // Update in-memory hash before DB write, so watcher can skip self-writes
        entry.set_file_hash(&file_hash);
        entry._last_write_ms.store(now_ms(), Ordering::Release);

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

            // Acquire reload_lock to prevent racing with external reload
            let _guard = entry.reload_lock.lock().await;

            // Re-check dirty after acquiring lock (reload may have cleared it)
            if !entry.dirty.load(Ordering::Acquire) {
                continue;
            }

            let snapshot = match entry.snapshot().await {
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
