//! P2P 事件循环：接收 NodeEvent 并分发到 DeviceManager、Tauri 事件、系统托盘。

use std::sync::Arc;

use sea_orm::{EntityTrait, PaginatorTrait};
use swarm_p2p_core::event::NodeEvent;
use swarm_p2p_core::libp2p::PeerId;
use swarm_p2p_core::EventReceiver;
use tauri::{AppHandle, Emitter, Manager};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::device::{DeviceFilter, DeviceManager};
use crate::network::online::AppNetClient;
use crate::pairing::PairingManager;
use crate::protocol::{
    AppRequest, AppResponse, WorkspaceMeta, WorkspaceRequest, WorkspaceResponse,
};
use crate::sync::{decode_ws_gossip, parse_sync_topic, parse_ws_topic, SyncManager};
use crate::workspace::state::{DbState, WorkspaceState};

/// Tauri 事件名常量
pub mod events {
    pub const DEVICES_CHANGED: &str = "devices-changed";
    pub const NETWORK_STATUS_CHANGED: &str = "network-status-changed";
    pub const PAIRING_REQUEST_RECEIVED: &str = "pairing-request-received";
    pub const PAIRED_DEVICE_ADDED: &str = "paired-device-added";
    pub const PAIRED_DEVICE_REMOVED: &str = "paired-device-removed";
}

/// 启动事件循环，持续读取 NodeEvent 并分发到 DeviceManager + Tauri 事件
pub fn spawn_event_loop(
    mut receiver: EventReceiver<AppRequest>,
    app: AppHandle,
    client: AppNetClient,
    device_manager: Arc<DeviceManager>,
    pairing_manager: Arc<PairingManager>,
    sync_manager: Arc<SyncManager>,
    cancel_token: CancellationToken,
) {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    info!("Event loop cancelled");
                    break;
                }
                event = receiver.recv() => {
                    match event {
                        Some(event) => handle_event(event, &app, &client, &device_manager, &pairing_manager, &sync_manager).await,
                        None => {
                            info!("Event receiver closed, exiting event loop");
                            break;
                        }
                    }
                }
            }
        }
    });
}

async fn handle_event(
    event: NodeEvent<AppRequest>,
    app: &AppHandle,
    client: &AppNetClient,
    device_manager: &DeviceManager,
    pairing_manager: &PairingManager,
    sync_manager: &Arc<SyncManager>,
) {
    // 统一更新设备状态
    device_manager.handle_event(&event);

    let emit_devices = || {
        let devices = device_manager.get_devices(DeviceFilter::All);
        let _ = app.emit(events::DEVICES_CHANGED, &devices);
    };

    match event {
        // ── 设备发现与连接 ──
        NodeEvent::Listening { addr } => {
            info!("Listening on {addr}");
        }
        NodeEvent::PeersDiscovered { ref peers } => {
            info!("Discovered {} peer(s)", peers.len());
            emit_devices();
        }
        NodeEvent::PeerConnected { ref peer_id } => {
            info!("Peer connected: {peer_id}");
            emit_devices();
            #[cfg(desktop)]
            update_tray_peer_count(app, device_manager).await;
        }
        NodeEvent::PeerDisconnected { ref peer_id } => {
            info!("Peer disconnected: {peer_id}");
            emit_devices();
            #[cfg(desktop)]
            update_tray_peer_count(app, device_manager).await;
        }
        NodeEvent::IdentifyReceived {
            ref peer_id,
            ref agent_version,
            ..
        } => {
            info!("Identified peer: {peer_id} ({agent_version})");
            emit_devices();

            // If the identified peer is a paired device, trigger full sync
            if device_manager.is_paired(peer_id) {
                sync_manager.on_paired_peer_connected(*peer_id).await;
            }
        }
        NodeEvent::PingSuccess { .. } => {
            emit_devices();
        }
        NodeEvent::HolePunchSucceeded { ref peer_id } => {
            info!("Hole punch succeeded with {peer_id}");
            emit_devices();
        }

        // ── 网络状态 ──
        NodeEvent::NatStatusChanged {
            status,
            public_addr,
        } => {
            info!("NAT status changed: {status:?}, public_addr: {public_addr:?}");
            let payload = serde_json::json!({
                "natStatus": format!("{status:?}"),
                "publicAddr": public_addr.map(|a| a.to_string()),
            });
            let _ = app.emit(events::NETWORK_STATUS_CHANGED, payload);
        }
        NodeEvent::HolePunchFailed { peer_id, error } => {
            warn!("Hole punch failed with {peer_id}: {error}");
        }
        NodeEvent::RelayReservationAccepted {
            relay_peer_id,
            renewal,
        } => {
            info!(
                "Relay reservation {} by {relay_peer_id}",
                if renewal { "renewed" } else { "accepted" }
            );
        }

        // ── 入站请求 ──
        NodeEvent::InboundRequest {
            peer_id,
            pending_id,
            request,
        } => {
            handle_inbound_request(
                app,
                client,
                pairing_manager,
                sync_manager,
                peer_id,
                pending_id,
                request,
            )
            .await;
        }

        // ── GossipSub ──
        NodeEvent::GossipMessage {
            source,
            topic,
            data,
        } => {
            if let Some(ws_uuid) = parse_ws_topic(&topic) {
                // Workspace-level topic: decode doc_uuid from payload
                if let Some((doc_uuid, update)) = decode_ws_gossip(&data) {
                    sync_manager
                        .handle_ws_gossip_update(source, ws_uuid, doc_uuid, update.to_vec())
                        .await;
                } else {
                    warn!("Invalid workspace GossipSub payload on {topic}");
                }
            } else if let Some(doc_uuid) = parse_sync_topic(&topic) {
                // Legacy per-doc topic (backwards compat during transition)
                let ydoc_mgr = sync_manager.app.state::<crate::yjs::manager::YDocManager>();
                if let Some(Err(e)) = ydoc_mgr
                    .apply_sync_update(&sync_manager.app, &doc_uuid, &data)
                    .await
                {
                    warn!("Failed to apply legacy gossip update for {doc_uuid}: {e}");
                }
            } else {
                info!("GossipSub message on unknown topic: {topic}");
            }
        }
        NodeEvent::GossipSubscribed { peer_id, topic } => {
            info!("Peer {peer_id} subscribed to topic {topic}");
        }
        NodeEvent::GossipUnsubscribed { peer_id, topic } => {
            info!("Peer {peer_id} unsubscribed from topic {topic}");
        }
    }
}

// ── 入站请求处理 ──

async fn handle_inbound_request(
    app: &AppHandle,
    client: &AppNetClient,
    pairing_manager: &PairingManager,
    sync_manager: &SyncManager,
    peer_id: PeerId,
    pending_id: u64,
    request: AppRequest,
) {
    match &request {
        AppRequest::Pairing(pairing_req) => {
            info!("Received pairing request from {peer_id} (pending_id={pending_id})");
            pairing_manager.cache_inbound_request(peer_id, pending_id, pairing_req);

            let expires_at = chrono::Utc::now() + chrono::Duration::seconds(90);
            let payload = serde_json::json!({
                "pendingId": pending_id,
                "peerId": peer_id.to_string(),
                "osInfo": pairing_req.os_info,
                "method": pairing_req.method,
                "expiresAt": expires_at,
            });

            emit_to_focused_or_all(app, events::PAIRING_REQUEST_RECEIVED, &payload);
            notify_if_unfocused(
                app,
                "配对请求",
                &format!("{} 请求与您配对", pairing_req.os_info.hostname),
            );
        }

        AppRequest::Workspace(WorkspaceRequest::ListWorkspaces) => {
            info!("Received ListWorkspaces request from {peer_id}");
            let response = build_workspace_list(app).await;
            if let Err(e) = client
                .send_response(pending_id, AppResponse::Workspace(response))
                .await
            {
                warn!("Failed to send WorkspaceList response to {peer_id}: {e}");
            }
        }

        AppRequest::Sync(sync_req) => {
            sync_manager
                .handle_inbound_request(peer_id, pending_id, sync_req.clone())
                .await;
        }
    }
}

/// 从 Tauri managed state 构建当前已打开工作区的元数据列表。
async fn build_workspace_list(app: &AppHandle) -> WorkspaceResponse {
    let empty = || WorkspaceResponse::WorkspaceList { workspaces: vec![] };

    let Some(ws_state) = app.try_state::<WorkspaceState>() else {
        return empty();
    };
    let Some(db_state) = app.try_state::<DbState>() else {
        return empty();
    };

    let all_workspaces = ws_state.list_all().await;
    let mut metas = Vec::with_capacity(all_workspaces.len());

    for ws_info in &all_workspaces {
        let doc_count = match db_state.workspace_db(&ws_info.id).await {
            Ok(guard) => {
                use entity::workspace::documents;
                documents::Entity::find()
                    .count(guard.conn())
                    .await
                    .unwrap_or(0) as u32
            }
            Err(_) => 0,
        };

        metas.push(WorkspaceMeta {
            uuid: ws_info.id,
            name: ws_info.name.clone(),
            doc_count,
            updated_at: ws_info.updated_at.timestamp_millis(),
        });
    }

    WorkspaceResponse::WorkspaceList { workspaces: metas }
}

// ── 辅助函数 ──

/// 更新托盘的 peer 连接计数
#[cfg(desktop)]
async fn update_tray_peer_count(app: &AppHandle, device_manager: &DeviceManager) {
    if let Some(tray) = app.try_state::<crate::tray::TrayManagerState>() {
        let count = device_manager.connected_count();
        tray.lock()
            .await
            .set_status(crate::tray::TrayNodeStatus::Running { peer_count: count });
    }
}

/// 向聚焦窗口定向 emit，无聚焦窗口时广播给所有窗口
fn emit_to_focused_or_all<S: serde::Serialize + Clone>(app: &AppHandle, event: &str, payload: &S) {
    let focused = app
        .webview_windows()
        .values()
        .find(|w| w.is_focused().unwrap_or(false))
        .map(|w| w.label().to_string());

    if let Some(label) = focused {
        if let Some(win) = app.get_webview_window(&label) {
            let _ = win.emit(event, payload.clone());
            return;
        }
    }
    let _ = app.emit(event, payload.clone());
}

/// 当所有窗口都不在前台时发送系统通知
fn notify_if_unfocused(app: &AppHandle, title: &str, body: &str) {
    let any_focused = app
        .webview_windows()
        .values()
        .any(|w| w.is_focused().unwrap_or(false));

    if !any_focused {
        use tauri_plugin_notification::NotificationExt;
        let _ = app.notification().builder().title(title).body(body).show();
    }
}
