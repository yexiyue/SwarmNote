//! `YDocManager` — per-workspace singleton that owns every in-memory Y.Doc
//! for a workspace, drives debounced writeback, and processes external `.md`
//! reloads.
//!
//! Construction injects all platform-abstracted dependencies:
//! - [`Arc<dyn FileSystem>`] — workspace-relative text I/O for `.md` writeback.
//! - [`Arc<dyn EventBus>`] — emits `AppEvent::DocFlushed` / `ExternalUpdate`
//!   / `ExternalConflict` events.
//! - [`Arc<DatabaseConnection>`] — per-workspace DB for `yjs_state` /
//!   `state_vector` / `file_hash` persistence.
//! - Workspace UUID + creator PeerId for row stamping.
//!
//! The `docs` map is keyed by doc_uuid only — there is no `label` dimension
//! (see `extract-swarmnote-core` PR #2: desktop multi-window no longer shards
//! Y.Doc instances per window; multiple windows on the same workspace share
//! one `Arc<WorkspaceCore>` → one `YDocManager` → one `DocEntry` per doc).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use dashmap::DashMap;
use entity::workspace::documents;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tokio::task::JoinHandle;
use uuid::Uuid;
use yrs::updates::encoder::Encode;
use yrs::{Doc, ReadTxn, StateVector, Text, Transact};

use crate::document::title_from_rel_path;
use crate::error::{AppError, AppResult};
use crate::events::{AppEvent, EventBus};
use crate::fs::FileSystem;

use super::FRAGMENT_NAME;

const DEBOUNCE_MS: u64 = 1500;
const POLL_INTERVAL_MS: u64 = 500;

// ── Return types ─────────────────────────────────────────────

/// Returned by [`YDocManager::open_doc`] so the frontend knows the stable UUID.
#[derive(Debug, Clone, serde::Serialize)]
pub struct OpenDocResult {
    /// Stable document UUID (database primary key).
    pub doc_uuid: Uuid,
    /// Full Y.Doc state as binary v1 update.
    pub yjs_state: Vec<u8>,
}

/// Outcome of [`YDocManager::reload_from_file`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReloadStatus {
    /// Document was open and content was replaced from disk.
    Reloaded,
    /// Document was open but the file hash matched (self-write) — no action.
    Skipped,
    /// Document is not loaded.
    NotOpen,
    /// Document had unsaved edits; `ExternalConflict` event was emitted.
    Conflict,
}

// ── DocEntry ──────────────────────────────────────────────────

struct DocSnapshot {
    yjs_state: Vec<u8>,
    state_vector: Vec<u8>,
    markdown: String,
}

/// Per-document Y.Doc entry held in memory.
///
/// `Doc` uses `tokio::sync::Mutex` (not `std::sync::Mutex`) so the lock
/// never poisons on panic and all callers are already in async context.
struct DocEntry {
    doc: tokio::sync::Mutex<Doc>,
    rel_path: RwLock<String>,
    doc_db_id: Uuid,
    dirty: AtomicBool,
    last_update_ms: AtomicU64,
    writeback_handle: tokio::sync::Mutex<Option<JoinHandle<()>>>,
    /// Blake3 hash of the last `.md` file we wrote — used to skip self-writes
    /// from the file watcher.
    file_hash: RwLock<Vec<u8>>,
    /// Mutual exclusion between writeback and external reload.
    reload_lock: tokio::sync::Mutex<()>,
}

impl DocEntry {
    async fn apply_update(&self, update: &[u8]) -> AppResult<()> {
        let doc = self.doc.lock().await;
        super::apply_update_to_doc(&doc, update, "DocEntry apply")
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
        let markdown = super::doc_to_markdown(&doc);
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

    /// Replace the Y.Doc content from markdown and return the incremental
    /// diff. Must be called while holding `reload_lock`.
    async fn replace_content_from_md(&self, md: &str) -> Vec<u8> {
        let doc = self.doc.lock().await;
        let sv_before = {
            let txn = doc.transact();
            txn.state_vector()
        };
        super::replace_doc_content(&doc, md);
        let txn = doc.transact();
        txn.encode_state_as_update_v1(&sv_before)
    }
}

// ── YDocManager ───────────────────────────────────────────────

/// Per-workspace Y.Doc lifecycle manager.
pub struct YDocManager {
    docs: DashMap<Uuid, Arc<DocEntry>>,
    workspace_id: Uuid,
    fs: Arc<dyn FileSystem>,
    event_bus: Arc<dyn EventBus>,
    db: Arc<DatabaseConnection>,
    /// Local device PeerId, stamped on newly-created document rows.
    peer_id: String,
}

impl YDocManager {
    pub fn new(
        workspace_id: Uuid,
        fs: Arc<dyn FileSystem>,
        event_bus: Arc<dyn EventBus>,
        db: Arc<DatabaseConnection>,
        peer_id: String,
    ) -> Arc<Self> {
        Arc::new(Self {
            docs: DashMap::new(),
            workspace_id,
            fs,
            event_bus,
            db,
            peer_id,
        })
    }

    // ── Open / Close / Update ────────────────────────────────

    /// Open a document: upsert the DB record (gets UUID), load or seed Y.Doc,
    /// return UUID + full state.
    ///
    /// Auto-creates a document row for files without one (e.g. externally
    /// copied `.md`).
    pub async fn open_doc(self: &Arc<Self>, rel_path: &str) -> AppResult<OpenDocResult> {
        // Fast path: already open → return current state.
        if let Some((existing_uuid, entry)) = self.find_entry_by_rel_path(rel_path) {
            return Ok(OpenDocResult {
                doc_uuid: existing_uuid,
                yjs_state: entry.encode_full_state().await,
            });
        }

        // Find-first: on the hot path the document row already exists
        // (reconcile ran on workspace open), so this one SELECT is all we
        // need. Only on a cold path (externally-copied `.md`) do we fall
        // through to INSERT + re-SELECT to survive concurrent opens.
        let doc_model = match find_doc_row(&self.db, self.workspace_id, rel_path).await? {
            Some(existing) => existing,
            None => {
                let new_id = Uuid::now_v7();
                let now = chrono::Utc::now();
                let insert_model = documents::ActiveModel {
                    id: Set(new_id),
                    workspace_id: Set(self.workspace_id),
                    folder_id: Set(None),
                    title: Set(title_from_rel_path(rel_path)),
                    rel_path: Set(rel_path.to_owned()),
                    lamport_clock: Set(0),
                    created_by: Set(self.peer_id.clone()),
                    created_at: Set(now),
                    updated_at: Set(now),
                    ..Default::default()
                };
                match documents::Entity::insert(insert_model)
                    .on_conflict(
                        sea_orm::sea_query::OnConflict::columns([
                            documents::Column::WorkspaceId,
                            documents::Column::RelPath,
                        ])
                        .do_nothing()
                        .to_owned(),
                    )
                    .exec(&*self.db)
                    .await
                {
                    Ok(_) | Err(sea_orm::DbErr::RecordNotInserted) => {}
                    Err(e) => {
                        tracing::error!("Failed to upsert document record for {rel_path}: {e}");
                        return Err(AppError::Database(e));
                    }
                }
                let row = find_doc_row(&self.db, self.workspace_id, rel_path)
                    .await?
                    .ok_or_else(|| {
                        AppError::Yjs(format!(
                            "Document record missing after upsert for {rel_path}"
                        ))
                    })?;
                if row.id == new_id {
                    tracing::info!("Auto-created DB record for {rel_path} → {new_id}");
                }
                row
            }
        };
        let doc_uuid = doc_model.id;

        // Creates a Utf16-offset Y.Doc with the "document" Y.Text pre-registered.
        let doc = super::create_doc();

        // Migration: clear yjs_state if it contains old absolute asset URLs
        // (pre-sync era stored tauri:// or asset:// URLs in Y.Doc).
        let restorable_state = doc_model.yjs_state.as_ref().filter(|state| {
            !state
                .windows(8)
                .any(|w| w.starts_with(b"tauri://") || w.starts_with(b"asset://"))
        });

        // Try to restore from persisted state. Falls back to `.md` if:
        //  - decode fails, or
        //  - decoded state leaves Y.Text empty (legacy BlockNote schema
        //    stored content under `XmlFragment("document-store")`, not
        //    `Y.Text("document")`).
        let mut loaded_from_state = false;
        if let Some(yjs_state) = restorable_state {
            match super::apply_update_to_doc(&doc, yjs_state, "open_doc restore") {
                Ok(()) => {
                    let text = doc.get_or_insert_text(FRAGMENT_NAME);
                    let txn = doc.transact();
                    if text.len(&txn) > 0 {
                        loaded_from_state = true;
                    } else {
                        drop(txn);
                        tracing::info!(
                            "yjs_state for {rel_path} decoded but Y.Text empty (legacy schema), rebuilding from .md"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "yjs_state decode failed for {rel_path}, rebuilding from .md: {e}"
                    );
                }
            }
        }

        let doc = if loaded_from_state {
            doc
        } else {
            let md = match self.fs.read_text(rel_path).await {
                Ok(c) => c,
                Err(AppError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
                Err(e) => return Err(e),
            };
            // Discard the partially-loaded doc (may still hold a legacy
            // XmlFragment from a failed restore) and start fresh.
            let fresh = super::create_doc();
            super::fill_doc_with_markdown(&fresh, &md);
            fresh
        };

        let state = doc
            .transact()
            .encode_state_as_update_v1(&StateVector::default());

        let init_hash = doc_model.file_hash.clone().unwrap_or_default();

        let entry = Arc::new(DocEntry {
            doc: tokio::sync::Mutex::new(doc),
            rel_path: RwLock::new(rel_path.to_owned()),
            doc_db_id: doc_uuid,
            dirty: AtomicBool::new(false),
            last_update_ms: AtomicU64::new(now_ms()),
            writeback_handle: tokio::sync::Mutex::new(None),
            file_hash: RwLock::new(init_hash),
            reload_lock: tokio::sync::Mutex::new(()),
        });

        if !loaded_from_state {
            // Fresh state — persist so subsequent opens skip the fallback path.
            let snapshot = entry.snapshot().await?;
            self.persist_snapshot(&entry, snapshot).await?;
        }

        let handle = spawn_writeback_task(self.clone(), doc_uuid, Arc::clone(&entry));
        *entry.writeback_handle.lock().await = Some(handle);

        self.docs.insert(doc_uuid, entry);
        Ok(OpenDocResult {
            doc_uuid,
            yjs_state: state,
        })
    }

    /// Apply an incremental update (from the frontend editor).
    pub async fn apply_update(&self, doc_uuid: Uuid, update: &[u8]) -> AppResult<()> {
        let entry = self
            .docs
            .get(&doc_uuid)
            .ok_or_else(|| AppError::DocNotOpen(doc_uuid.to_string()))?;

        entry.apply_update(update).await?;
        entry.mark_dirty();
        Ok(())
    }

    /// Close a document: flush if dirty, stop writeback task, remove.
    pub async fn close_doc(&self, doc_uuid: Uuid) -> AppResult<()> {
        let Some((_, entry)) = self.docs.remove(&doc_uuid) else {
            return Ok(());
        };

        if let Some(handle) = entry.writeback_handle.lock().await.take() {
            handle.abort();
        }

        if entry.dirty.load(Ordering::Acquire) {
            let snapshot = entry.snapshot().await?;
            self.persist_snapshot(&entry, snapshot).await?;
        }

        Ok(())
    }

    /// Rename a document's `rel_path` in-place (UUID stays the same).
    pub fn rename_doc(&self, doc_uuid: Uuid, new_rel_path: &str) {
        if let Some(entry) = self.docs.get(&doc_uuid) {
            entry.set_rel_path(new_rel_path);
        }
    }

    // ── Sync layer methods ──

    /// Apply a remote sync update to an open document. Returns `None` if
    /// the document is not currently open.
    pub async fn apply_sync_update(&self, doc_uuid: &Uuid, update: &[u8]) -> Option<AppResult<()>> {
        let entry = self.docs.get(doc_uuid).map(|e| Arc::clone(&*e))?;

        let result = entry.apply_update(update).await;
        if result.is_ok() {
            entry.mark_dirty();
            self.event_bus.emit(AppEvent::ExternalUpdate {
                doc_id: *doc_uuid,
                update: update.to_vec(),
            });
        }
        Some(result)
    }

    /// Get the state vector for an open document. Returns `None` if not open.
    pub async fn get_state_vector(&self, doc_uuid: &Uuid) -> Option<Vec<u8>> {
        let entry = self.docs.get(doc_uuid).map(|e| Arc::clone(&*e))?;
        let doc = entry.doc.lock().await;
        let txn = doc.transact();
        Some(txn.state_vector().encode_v1())
    }

    /// Encode the diff that a peer with the given state vector is missing.
    /// Returns `None` if not open.
    pub async fn encode_diff_for_sv(
        &self,
        doc_uuid: &Uuid,
        remote_sv: &StateVector,
    ) -> Option<Vec<u8>> {
        let entry = self.docs.get(doc_uuid).map(|e| Arc::clone(&*e))?;
        let doc = entry.doc.lock().await;
        let txn = doc.transact();
        Some(txn.encode_state_as_update_v1(remote_sv))
    }

    /// Encode the full state of an open document. Returns `None` if not open.
    pub async fn encode_full_state(&self, doc_uuid: &Uuid) -> Option<Vec<u8>> {
        let entry = self.docs.get(doc_uuid).map(|e| Arc::clone(&*e))?;
        Some(entry.encode_full_state().await)
    }

    pub fn is_doc_open(&self, doc_uuid: &Uuid) -> bool {
        self.docs.contains_key(doc_uuid)
    }

    pub fn list_open_doc_uuids(&self) -> Vec<Uuid> {
        self.docs.iter().map(|e| *e.key()).collect()
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    /// Close every doc still open (flushes if dirty). Called by
    /// `WorkspaceCore::close`.
    pub async fn close_all(&self) {
        let uuids: Vec<Uuid> = self.docs.iter().map(|e| *e.key()).collect();
        for uuid in uuids {
            if let Err(e) = self.close_doc(uuid).await {
                tracing::warn!("Failed to close doc {uuid}: {e}");
            }
        }
    }

    // ── External file reload ─────────────────────────────────

    fn find_entry_by_rel_path(&self, rel_path: &str) -> Option<(Uuid, Arc<DocEntry>)> {
        self.docs
            .iter()
            .find(|e| e.rel_path() == rel_path)
            .map(|e| (*e.key(), Arc::clone(&*e)))
    }

    /// Called by the file watcher when a `.md` file changes on disk.
    pub async fn reload_from_file(&self, rel_path: &str) -> AppResult<ReloadStatus> {
        let Some((doc_uuid, entry)) = self.find_entry_by_rel_path(rel_path) else {
            return Ok(ReloadStatus::NotOpen);
        };

        let new_content = match self.fs.read_text(rel_path).await {
            Ok(c) => c,
            Err(AppError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(ReloadStatus::Skipped);
            }
            Err(e) => return Err(e),
        };
        let new_hash = blake3::hash(new_content.as_bytes());

        if new_hash.as_bytes() == entry.file_hash().as_slice() {
            return Ok(ReloadStatus::Skipped);
        }

        if entry.dirty.load(Ordering::Acquire) {
            tracing::info!("External conflict for {rel_path} (doc {doc_uuid})");
            self.event_bus.emit(AppEvent::ExternalConflict {
                doc_id: doc_uuid,
                rel_path: rel_path.to_owned(),
            });
            return Ok(ReloadStatus::Conflict);
        }

        tracing::info!("External reload for {rel_path} (doc {doc_uuid})");
        self.do_reload(doc_uuid, &entry, &new_content).await?;
        Ok(ReloadStatus::Reloaded)
    }

    /// Called after the user confirms reload in the conflict dialog.
    pub async fn reload_confirmed(&self, doc_uuid: Uuid) -> AppResult<()> {
        let entry = self
            .docs
            .get(&doc_uuid)
            .map(|e| Arc::clone(&*e))
            .ok_or_else(|| AppError::DocNotOpen(doc_uuid.to_string()))?;

        let rel_path = entry.rel_path();
        let content = self.fs.read_text(&rel_path).await?;

        entry.dirty.store(false, Ordering::Release);
        self.do_reload(doc_uuid, &entry, &content).await
    }

    /// Shared reload: replace Y.Doc content, persist, and notify frontend.
    async fn do_reload(&self, doc_uuid: Uuid, entry: &DocEntry, raw_md: &str) -> AppResult<()> {
        let _guard = entry.reload_lock.lock().await;
        let diff = entry.replace_content_from_md(raw_md).await;

        let snapshot = entry.snapshot().await?;
        self.persist_snapshot(entry, snapshot).await?;

        self.event_bus.emit(AppEvent::ExternalUpdate {
            doc_id: doc_uuid,
            update: diff,
        });
        Ok(())
    }

    // ── Persistence ──────────────────────────────────────────

    /// Persist snapshot to DB + write `.md` via the FileSystem trait.
    /// Updates the in-memory `file_hash` so self-writes are skipped by the
    /// watcher.
    async fn persist_snapshot(&self, entry: &DocEntry, snapshot: DocSnapshot) -> AppResult<()> {
        let md = snapshot.markdown;
        let rel = entry.rel_path();

        self.fs.write_text(&rel, &md).await?;
        let file_hash = blake3::hash(md.as_bytes()).as_bytes().to_vec();

        entry.set_file_hash(&file_hash);

        if let Some(existing) = documents::Entity::find_by_id(entry.doc_db_id)
            .one(&*self.db)
            .await?
        {
            let mut model: documents::ActiveModel = existing.into();
            model.yjs_state = Set(Some(snapshot.yjs_state));
            model.state_vector = Set(Some(snapshot.state_vector));
            model.file_hash = Set(Some(file_hash));
            model.rel_path = Set(entry.rel_path());
            model.update(&*self.db).await?;
        }

        Ok(())
    }
}

// ── Free helper functions ──

fn spawn_writeback_task(
    mgr: Arc<YDocManager>,
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

            // Acquire reload_lock to prevent racing with external reload.
            let _guard = entry.reload_lock.lock().await;

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

            if let Err(e) = mgr.persist_snapshot(&entry, snapshot).await {
                tracing::warn!("writeback failed for doc {doc_uuid}: {e}");
                entry.dirty.store(true, Ordering::Release);
                continue;
            }

            mgr.event_bus
                .emit(AppEvent::DocFlushed { doc_id: doc_uuid });
        }
    })
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time after epoch")
        .as_millis() as u64
}

/// Look up the document row matching `(workspace_id, rel_path)`. Extracted
/// so `open_doc`'s two call sites share one query shape.
async fn find_doc_row(
    db: &DatabaseConnection,
    workspace_id: Uuid,
    rel_path: &str,
) -> AppResult<Option<documents::Model>> {
    Ok(documents::Entity::find()
        .filter(documents::Column::RelPath.eq(rel_path))
        .filter(documents::Column::WorkspaceId.eq(workspace_id))
        .one(db)
        .await?)
}
