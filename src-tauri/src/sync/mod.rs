pub(crate) mod asset_sync;
pub mod commands;
mod doc_sync;
mod full_sync;
mod manager;

pub use manager::{parse_sync_topic, SyncManager};

use serde::{Deserialize, Serialize};

/// Per-document sync status, emitted to frontend via Tauri events.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DocSyncStatus {
    /// Fully synced with all connected peers.
    Synced,
    /// Currently receiving/sending updates.
    Syncing,
    /// Queued for sync (full sync hasn't reached this doc yet).
    Pending,
    /// Local-only modifications, no peers connected.
    LocalOnly,
}
