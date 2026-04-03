//! In-memory buffer for GossipSub updates targeting closed documents.
//!
//! Accumulates raw yrs updates per (workspace_uuid, doc_uuid) and flushes
//! them in batch after a configurable debounce interval (default 3 s).
//! Enforces a per-doc update cap to prevent unbounded memory growth.
//! After flush, triggers asset sync for the affected documents.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use swarm_p2p_core::libp2p::PeerId;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;
use tracing::{info, warn};
use uuid::Uuid;

use super::{asset_sync, doc_sync};

/// (doc_uuid, workspace_uuid, source_peer, updates) ready to flush.
type FlushBatch = Vec<(Uuid, Uuid, Option<PeerId>, Vec<Vec<u8>>)>;

/// Debounce interval: flush pending updates 3 s after the last write.
const FLUSH_DEBOUNCE: Duration = Duration::from_secs(3);
/// Tick interval for the background flush task.
const TICK_INTERVAL: Duration = Duration::from_millis(500);
/// Maximum number of buffered updates per document before forcing an early flush.
const MAX_UPDATES_PER_DOC: usize = 500;

#[derive(Debug)]
struct PendingEntry {
    workspace_uuid: Uuid,
    /// Most recent source peer (for asset sync after flush).
    source_peer: Option<PeerId>,
    updates: Vec<Vec<u8>>,
    last_write: Instant,
}

/// Thread-safe buffer that accumulates yrs updates for closed documents
/// and periodically flushes them to DB + filesystem.
pub struct PendingUpdateBuffer {
    entries: Arc<Mutex<HashMap<Uuid, PendingEntry>>>,
}

impl PendingUpdateBuffer {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Push a raw yrs update for a closed document.
    ///
    /// If the per-doc update count exceeds [`MAX_UPDATES_PER_DOC`], the entry
    /// is drained and returned for immediate flushing by the caller.
    pub async fn push(
        &self,
        workspace_uuid: Uuid,
        doc_uuid: Uuid,
        update: Vec<u8>,
        source: Option<PeerId>,
    ) -> Option<(Uuid, Option<PeerId>, Vec<Vec<u8>>)> {
        let mut map = self.entries.lock().await;
        let entry = map.entry(doc_uuid).or_insert_with(|| PendingEntry {
            workspace_uuid,
            source_peer: source,
            updates: Vec::new(),
            last_write: Instant::now(),
        });
        entry.updates.push(update);
        entry.last_write = Instant::now();
        // Keep the most recent source peer for asset sync
        if source.is_some() {
            entry.source_peer = source;
        }

        if entry.updates.len() >= MAX_UPDATES_PER_DOC {
            let drained = map.remove(&doc_uuid).unwrap();
            Some((drained.workspace_uuid, drained.source_peer, drained.updates))
        } else {
            None
        }
    }

    /// Spawn a background task that periodically flushes stale entries.
    /// Returns an `AbortHandle` the caller can use to stop the task.
    pub fn spawn_flush_task(&self, app: AppHandle) -> tokio::task::AbortHandle {
        let entries = Arc::clone(&self.entries);
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(TICK_INTERVAL);
            loop {
                interval.tick().await;

                // Collect entries that are ready to flush (move, not clone)
                let ready: FlushBatch = {
                    let mut map = entries.lock().await;
                    let now = Instant::now();
                    let mut to_flush = Vec::new();
                    let mut to_remove = Vec::new();

                    for (doc_uuid, entry) in map.iter() {
                        if now.duration_since(entry.last_write) >= FLUSH_DEBOUNCE {
                            to_remove.push(*doc_uuid);
                        }
                    }

                    for key in &to_remove {
                        if let Some(mut entry) = map.remove(key) {
                            to_flush.push((
                                *key,
                                entry.workspace_uuid,
                                entry.source_peer,
                                std::mem::take(&mut entry.updates),
                            ));
                        }
                    }

                    to_flush
                };

                // Apply each batch outside the lock
                for (doc_uuid, workspace_uuid, source_peer, updates) in ready {
                    let flushed =
                        flush_updates(&app, workspace_uuid, doc_uuid, updates, &entries).await;

                    // Trigger asset sync after successful flush
                    if flushed {
                        schedule_closed_doc_asset_sync(&app, source_peer, workspace_uuid, doc_uuid)
                            .await;
                    }
                }
            }
        });
        handle.abort_handle()
    }
}

/// Flush a batch of updates for a single document.
/// On failure, remaining updates are re-inserted into the buffer for retry.
/// Returns `true` if at least one update was applied successfully.
async fn flush_updates(
    app: &AppHandle,
    workspace_uuid: Uuid,
    doc_uuid: Uuid,
    updates: Vec<Vec<u8>>,
    entries: &Arc<Mutex<HashMap<Uuid, PendingEntry>>>,
) -> bool {
    let total = updates.len();
    for (i, update) in updates.iter().enumerate() {
        if let Err(e) = doc_sync::apply_remote_update(app, workspace_uuid, doc_uuid, update).await {
            warn!("Pending buffer flush failed for doc {doc_uuid} at update {i}/{total}: {e}");

            // Re-buffer remaining updates for next tick
            let remaining: Vec<Vec<u8>> = updates[i..].to_vec();
            let mut map = entries.lock().await;
            let entry = map.entry(doc_uuid).or_insert_with(|| PendingEntry {
                workspace_uuid,
                source_peer: None,
                updates: Vec::new(),
                last_write: Instant::now(),
            });
            // Prepend remaining before any new updates that arrived since
            let mut merged = remaining;
            merged.append(&mut entry.updates);
            entry.updates = merged;
            return i > 0; // true if at least one update succeeded before failure
        }
    }
    info!("Flushed {total} pending updates for closed doc {doc_uuid}");
    true
}

/// Trigger asset sync for a closed document after its pending updates are flushed.
async fn schedule_closed_doc_asset_sync(
    app: &AppHandle,
    source_peer: Option<PeerId>,
    workspace_uuid: Uuid,
    doc_uuid: Uuid,
) {
    let Some(peer) = source_peer else { return };

    // Look up rel_path from DB
    let db_state = app.state::<crate::workspace::state::DbState>();
    let Ok(guard) = db_state.workspace_db(&workspace_uuid).await else {
        return;
    };

    use entity::workspace::documents;
    use sea_orm::EntityTrait;

    let rel_path = match documents::Entity::find_by_id(doc_uuid)
        .one(guard.conn())
        .await
    {
        Ok(Some(doc)) => doc.rel_path,
        _ => return,
    };

    // Need the SyncManager's client for asset sync
    if let Some(net_state) = app.try_state::<crate::network::NetManagerState>() {
        if let Ok(sync_mgr) = net_state.sync().await {
            if let Err(e) = asset_sync::sync_doc_assets(
                app,
                &sync_mgr.client,
                peer,
                workspace_uuid,
                doc_uuid,
                &rel_path,
            )
            .await
            {
                warn!("Asset sync after buffer flush failed for doc {doc_uuid}: {e}");
            }
        }
    }
}
