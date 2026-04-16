//! `EventBus` impl that forwards `AppEvent` variants to the Tauri IPC layer
//! via `app.emit(topic, payload)` broadcasts. The frontend subscribes to the
//! specific topic names (preserved from the pre-refactor event surface) and
//! filters the payload by `doc_id` / `workspace_id` / `peer_id` as needed.
//!
//! PR #1 skeleton — populated match arms are added as their corresponding
//! core modules come online in PR #2 (yjs / fs events) and PR #3 (network /
//! pairing / sync events).

use serde_json::json;
use tauri::{AppHandle, Emitter};

use swarmnote_core::{AppEvent, EventBus};

/// `EventBus` implementation backed by Tauri's `AppHandle::emit`. Broadcasts
/// to all windows — the frontend filters by the business keys in the payload
/// rather than by a target label.
pub struct TauriEventBus {
    app: AppHandle,
}

impl TauriEventBus {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl EventBus for TauriEventBus {
    fn emit(&self, event: AppEvent) {
        // Each arm translates a core event into one or more Tauri topic emits
        // preserving the frontend-visible event name + payload shape.
        match event {
            // ── YDoc / fs (PR #2) — stubs until YDocManager is ported. ──
            AppEvent::DocFlushed { doc_id } => {
                let _ = self
                    .app
                    .emit("yjs:flushed", json!({ "docUuid": doc_id.to_string() }));
            }
            AppEvent::ExternalUpdate { doc_id, update } => {
                let _ = self.app.emit(
                    "yjs:external-update",
                    json!({
                        "docUuid": doc_id.to_string(),
                        "update": update,
                    }),
                );
            }
            AppEvent::ExternalConflict { doc_id, rel_path } => {
                let _ = self.app.emit(
                    "yjs:external-conflict",
                    json!({
                        "docUuid": doc_id.to_string(),
                        "relPath": rel_path,
                    }),
                );
            }
            AppEvent::FileTreeChanged { workspace_id } => {
                let _ = self.app.emit(
                    "fs:tree-changed",
                    json!({ "workspaceId": workspace_id.to_string() }),
                );
            }

            // ── Devices / network / pairing / sync (PR #3) — stubs. ──
            AppEvent::PeerConnected {
                peer_id,
                device_name,
            } => {
                let _ = self.app.emit(
                    "peer-connected",
                    json!({ "peerId": peer_id, "deviceName": device_name }),
                );
            }
            AppEvent::PeerDisconnected { peer_id } => {
                let _ = self
                    .app
                    .emit("peer-disconnected", json!({ "peerId": peer_id }));
            }
            AppEvent::DevicesChanged => {
                // Frontend re-queries on this event — no payload needed.
                let _ = self.app.emit("devices-changed", ());
            }
            AppEvent::PairingRequestReceived {
                peer_id,
                device_name,
            } => {
                let _ = self.app.emit(
                    "pairing-request-received",
                    json!({ "peerId": peer_id, "deviceName": device_name }),
                );
            }
            AppEvent::PairedDeviceAdded { peer_id } => {
                let _ = self
                    .app
                    .emit("paired-device-added", json!({ "peerId": peer_id }));
            }
            AppEvent::PairedDeviceRemoved { peer_id } => {
                let _ = self
                    .app
                    .emit("paired-device-removed", json!({ "peerId": peer_id }));
            }
            AppEvent::NetworkStatusChanged { status } => {
                let _ = self.app.emit("network-status-changed", status);
            }
            AppEvent::NodeStarted => {
                let _ = self.app.emit("node-started", ());
            }
            AppEvent::NodeStopped => {
                let _ = self.app.emit("node-stopped", ());
            }
            AppEvent::FullSyncProgress {
                workspace_id,
                completed,
                total,
            } => {
                let _ = self.app.emit(
                    "full-sync-progress",
                    json!({
                        "workspaceId": workspace_id.to_string(),
                        "completed": completed,
                        "total": total,
                    }),
                );
            }
            AppEvent::FullSyncDone { workspace_id } => {
                let _ = self.app.emit(
                    "full-sync-done",
                    json!({ "workspaceId": workspace_id.to_string() }),
                );
            }
            AppEvent::NavigateTo { target, route } => {
                // Precision target-routing for navigation. The only case that
                // still benefits from `emit_to` because the user explicitly
                // said "open route X in window Y".
                let _ = self.app.emit_to(target.as_str(), "navigate", route);
            }
        }
    }
}
