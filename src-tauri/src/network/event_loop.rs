use std::sync::Arc;

use swarm_p2p_core::event::NodeEvent;
use swarm_p2p_core::EventReceiver;
use tauri::{AppHandle, Emitter, Manager};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::device::DeviceManager;
use crate::pairing::PairingManager;
use crate::protocol::AppRequest;

/// Tauri 事件名常量
pub mod events {
    pub const PEER_CONNECTED: &str = "peer-connected";
    pub const PEER_DISCONNECTED: &str = "peer-disconnected";
    pub const NETWORK_STATUS_CHANGED: &str = "network-status-changed";
    pub const PAIRING_REQUEST_RECEIVED: &str = "pairing-request-received";
    pub const PAIRED_DEVICE_ADDED: &str = "paired-device-added";
    pub const PAIRED_DEVICE_REMOVED: &str = "paired-device-removed";
    #[expect(dead_code)]
    pub const NEARBY_DEVICES_CHANGED: &str = "nearby-devices-changed";
}

/// 启动事件循环，持续读取 NodeEvent 并分发到 DeviceManager + Tauri 事件
pub fn spawn_event_loop(
    mut receiver: EventReceiver<AppRequest>,
    app: AppHandle,
    device_manager: Arc<DeviceManager>,
    pairing_manager: Arc<PairingManager>,
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
                        Some(event) => handle_event(event, &app, &device_manager, &pairing_manager).await,
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
    device_manager: &DeviceManager,
    pairing_manager: &PairingManager,
) {
    match event {
        NodeEvent::Listening { addr } => {
            info!("Listening on {addr}");
        }

        NodeEvent::PeersDiscovered { peers } => {
            info!("Discovered {} peer(s)", peers.len());
            device_manager.add_peers(peers);
        }

        NodeEvent::PeerConnected { peer_id } => {
            info!("Peer connected: {peer_id}");
            device_manager.set_connected(&peer_id);
            if let Some(peer_info) = device_manager.get_peer(&peer_id) {
                let _ = app.emit(events::PEER_CONNECTED, &peer_info);
            }
            #[cfg(desktop)]
            if let Some(tray) = app.try_state::<crate::tray::TrayManagerState>() {
                let count = device_manager.connected_count();
                tray.lock()
                    .await
                    .set_status(crate::tray::NodeStatus::Running { peer_count: count });
            }
        }

        NodeEvent::PeerDisconnected { peer_id } => {
            info!("Peer disconnected: {peer_id}");
            device_manager.set_disconnected(&peer_id);
            let _ = app.emit(events::PEER_DISCONNECTED, peer_id.to_string());
            #[cfg(desktop)]
            if let Some(tray) = app.try_state::<crate::tray::TrayManagerState>() {
                let count = device_manager.connected_count();
                tray.lock()
                    .await
                    .set_status(crate::tray::NodeStatus::Running { peer_count: count });
            }
        }

        NodeEvent::IdentifyReceived {
            peer_id,
            agent_version,
            ..
        } => {
            let is_swarmnote = device_manager.set_agent_version(&peer_id, &agent_version);
            if is_swarmnote {
                info!("Identified SwarmNote peer: {peer_id} ({agent_version})");
                if let Some(peer_info) = device_manager.get_peer(&peer_id) {
                    let _ = app.emit(events::PEER_CONNECTED, &peer_info);
                }
            }
        }

        NodeEvent::PingSuccess { peer_id, rtt_ms } => {
            device_manager.update_rtt(&peer_id, rtt_ms);
        }

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

        NodeEvent::HolePunchSucceeded { peer_id } => {
            info!("Hole punch succeeded with {peer_id}");
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

        NodeEvent::InboundRequest {
            peer_id,
            pending_id,
            request,
        } => {
            match &request {
                AppRequest::Pairing(ref pairing_req) => {
                    info!("Received pairing request from {peer_id} (pending_id={pending_id})");
                    pairing_manager.cache_inbound_request(peer_id, pending_id, pairing_req);

                    // 构建 payload
                    let expires_at = chrono::Utc::now().timestamp_millis() + 90_000; // 90s deadline
                    let payload = serde_json::json!({
                        "pendingId": pending_id,
                        "peerId": peer_id.to_string(),
                        "osInfo": pairing_req.os_info,
                        "method": pairing_req.method,
                        "expiresAt": expires_at,
                    });

                    // 定向 emit 或广播 + 通知
                    emit_to_focused_or_all(app, events::PAIRING_REQUEST_RECEIVED, &payload);
                    notify_if_unfocused(
                        app,
                        "配对请求",
                        &format!("{} 请求与您配对", pairing_req.os_info.hostname),
                    );
                }
                AppRequest::Sync(_) => {
                    warn!("Received sync request from {peer_id} (pending_id={pending_id}), but sync handler not yet implemented");
                }
            }
        }

        NodeEvent::GossipMessage { source, topic, .. } => {
            info!(
                "GossipSub message from {source:?} on topic {topic} (handler not yet implemented)"
            );
        }

        NodeEvent::GossipSubscribed { peer_id, topic } => {
            info!("Peer {peer_id} subscribed to topic {topic}");
        }

        NodeEvent::GossipUnsubscribed { peer_id, topic } => {
            info!("Peer {peer_id} unsubscribed from topic {topic}");
        }
    }
}

/// 向聚焦窗口定向 emit，无聚焦窗口时广播给所有窗口
fn emit_to_focused_or_all<S: serde::Serialize + Clone>(app: &AppHandle, event: &str, payload: &S) {
    use tauri::Manager;
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
    // 无聚焦窗口，广播
    let _ = app.emit(event, payload.clone());
}

/// 当所有窗口都不在前台时发送系统通知
fn notify_if_unfocused(app: &AppHandle, title: &str, body: &str) {
    use tauri::Manager;
    let any_focused = app
        .webview_windows()
        .values()
        .any(|w| w.is_focused().unwrap_or(false));

    if !any_focused {
        use tauri_plugin_notification::NotificationExt;
        let _ = app.notification().builder().title(title).body(body).show();
    }
}
