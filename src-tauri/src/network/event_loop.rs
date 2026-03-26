use std::sync::Arc;

use swarm_p2p_core::event::NodeEvent;
use swarm_p2p_core::EventReceiver;
use tauri::{AppHandle, Emitter};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::device::DeviceManager;
use crate::protocol::AppRequest;

/// Tauri 事件名常量
pub mod events {
    pub const PEER_CONNECTED: &str = "peer-connected";
    pub const PEER_DISCONNECTED: &str = "peer-disconnected";
    pub const NETWORK_STATUS_CHANGED: &str = "network-status-changed";
}

/// 启动事件循环，持续读取 NodeEvent 并分发到 DeviceManager + Tauri 事件
pub fn spawn_event_loop(
    mut receiver: EventReceiver<AppRequest>,
    app: AppHandle,
    device_manager: Arc<DeviceManager>,
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
                        Some(event) => handle_event(event, &app, &device_manager),
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

fn handle_event(event: NodeEvent<AppRequest>, app: &AppHandle, device_manager: &DeviceManager) {
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
        }

        NodeEvent::PeerDisconnected { peer_id } => {
            info!("Peer disconnected: {peer_id}");
            device_manager.set_disconnected(&peer_id);
            let _ = app.emit(events::PEER_DISCONNECTED, peer_id.to_string());
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
            // 暂不处理请求，留给 #26 (pairing) 和 #28 (sync)
            match &request {
                AppRequest::Pairing(_) => {
                    warn!("Received pairing request from {peer_id} (pending_id={pending_id}), but pairing handler not yet implemented");
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
