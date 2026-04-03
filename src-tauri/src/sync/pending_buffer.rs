//! In-memory buffer for GossipSub updates targeting closed documents.
//!
//! Accumulates raw yrs updates per (workspace_uuid, doc_uuid) and flushes
//! them in batch after a configurable debounce interval (default 3 s).
//! Enforces a per-doc update cap to prevent unbounded memory growth.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tauri::AppHandle;
use tokio::sync::Mutex;
use tracing::{info, warn};
use uuid::Uuid;

use super::doc_sync;

/// Debounce interval: flush pending updates 3 s after the last write.
const FLUSH_DEBOUNCE: Duration = Duration::from_secs(3);
/// Tick interval for the background flush task.
const TICK_INTERVAL: Duration = Duration::from_millis(500);
/// Maximum number of buffered updates per document before forcing an early flush.
const MAX_UPDATES_PER_DOC: usize = 500;

#[derive(Debug)]
struct PendingEntry {
    workspace_uuid: Uuid,
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
    ) -> Option<(Uuid, Vec<Vec<u8>>)> {
        let mut map = self.entries.lock().await;
        let entry = map.entry(doc_uuid).or_insert_with(|| PendingEntry {
            workspace_uuid,
            updates: Vec::new(),
            last_write: Instant::now(),
        });
        entry.updates.push(update);
        entry.last_write = Instant::now();

        if entry.updates.len() >= MAX_UPDATES_PER_DOC {
            let drained = map.remove(&doc_uuid).unwrap();
            Some((drained.workspace_uuid, drained.updates))
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
                let ready: Vec<(Uuid, Uuid, Vec<Vec<u8>>)> = {
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
                                std::mem::take(&mut entry.updates),
                            ));
                        }
                    }

                    to_flush
                };

                // Apply each batch outside the lock
                for (doc_uuid, workspace_uuid, updates) in ready {
                    flush_updates(&app, workspace_uuid, doc_uuid, updates, &entries).await;
                }
            }
        });
        handle.abort_handle()
    }
}

/// Flush a batch of updates for a single document.
/// On failure, remaining updates are re-inserted into the buffer for retry.
async fn flush_updates(
    app: &AppHandle,
    workspace_uuid: Uuid,
    doc_uuid: Uuid,
    updates: Vec<Vec<u8>>,
    entries: &Arc<Mutex<HashMap<Uuid, PendingEntry>>>,
) {
    let total = updates.len();
    for (i, update) in updates.iter().enumerate() {
        if let Err(e) = doc_sync::apply_remote_update(app, workspace_uuid, doc_uuid, update).await {
            warn!("Pending buffer flush failed for doc {doc_uuid} at update {i}/{total}: {e}");

            // Re-buffer remaining updates for next tick
            let remaining: Vec<Vec<u8>> = updates[i..].to_vec();
            let mut map = entries.lock().await;
            let entry = map.entry(doc_uuid).or_insert_with(|| PendingEntry {
                workspace_uuid,
                updates: Vec::new(),
                last_write: Instant::now(),
            });
            // Prepend remaining before any new updates that arrived since
            let mut merged = remaining;
            merged.append(&mut entry.updates);
            entry.updates = merged;
            return;
        }
    }
    info!("Flushed {total} pending updates for closed doc {doc_uuid}");
}
