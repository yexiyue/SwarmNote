//! Platform-abstracted event push channel.
//!
//! Replaces direct `tauri::AppHandle::emit()` calls inside the core layer.
//! Every event the core needs to emit goes through `EventBus::emit(AppEvent)`;
//! each host implementation pattern-matches the `AppEvent` enum and translates
//! it into its native form (Tauri IPC topic, uniffi callback, etc.).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Non-blocking event sink. `emit` MUST NOT acquire any core-layer locks —
/// implementations may hop onto another thread if they need to.
pub trait EventBus: Send + Sync + 'static {
    fn emit(&self, event: AppEvent);
}

/// All events produced by the core layer. Variants carry business keys
/// (`workspace_id`, `doc_id`, `peer_id`) but never platform-specific fields —
/// host implementations route/filter based on the key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AppEvent {
    // ── YDoc / documents ──
    /// Document has been flushed to disk + DB (writeback task).
    DocFlushed { doc_id: Uuid },
    /// Remote update applied to an open document — the editor for that doc
    /// should refresh from the supplied update bytes.
    ExternalUpdate {
        doc_id: Uuid,
        #[serde(with = "serde_bytes")]
        update: Vec<u8>,
    },
    /// An external editor modified a `.md` file while the user had unsaved
    /// edits — frontend MUST prompt for reload/keep.
    ExternalConflict { doc_id: Uuid, rel_path: String },

    // ── File tree ──
    /// Workspace file tree has changed (external editor created / deleted /
    /// moved a file). Frontend should re-scan.
    FileTreeChanged { workspace_id: Uuid },

    // ── Devices / discovery ──
    PeerConnected { peer_id: String, device_name: String },
    PeerDisconnected { peer_id: String },
    DevicesChanged,

    // ── Pairing ──
    PairingRequestReceived { peer_id: String, device_name: String },
    PairedDeviceAdded { peer_id: String },
    PairedDeviceRemoved { peer_id: String },

    // ── Network / P2P node ──
    NetworkStatusChanged { status: NetworkStatus },
    NodeStarted,
    NodeStopped,

    // ── Sync ──
    FullSyncProgress {
        workspace_id: Uuid,
        completed: u32,
        total: u32,
    },
    FullSyncDone { workspace_id: Uuid },

    // ── Navigation ──
    /// Host should route to `route` in the window identified by `target`.
    /// Only meaningful on desktop; mobile implementations MAY ignore.
    NavigateTo { target: String, route: String },
}

/// Snapshot of network node status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NetworkStatus {
    Stopped,
    Running,
    Error { message: String },
}
