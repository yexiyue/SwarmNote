pub(crate) mod asset_sync;
pub mod commands;
mod doc_sync;
mod full_sync;
mod manager;
mod pending_buffer;

pub use manager::{parse_sync_topic, parse_ws_topic, SyncManager};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

// ── GossipSub workspace topic payload encoding ──

/// GossipSub topic format for workspace-level document updates.
pub fn ws_topic(workspace_uuid: &Uuid) -> String {
    format!("swarmnote/ws/{workspace_uuid}")
}

/// Encode a workspace GossipSub payload: `[16 bytes doc_uuid][update bytes]`
pub fn encode_ws_gossip(doc_uuid: &Uuid, update: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(16 + update.len());
    buf.extend_from_slice(doc_uuid.as_bytes());
    buf.extend_from_slice(update);
    buf
}

/// Decode a workspace GossipSub payload into (doc_uuid, update_bytes).
/// Rejects payloads with no actual update content (must be > 16 bytes).
pub fn decode_ws_gossip(data: &[u8]) -> Option<(Uuid, &[u8])> {
    if data.len() <= 16 {
        return None;
    }
    let uuid_bytes: [u8; 16] = data[..16].try_into().ok()?;
    let doc_uuid = Uuid::from_bytes(uuid_bytes);
    Some((doc_uuid, &data[16..]))
}
