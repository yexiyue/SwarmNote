use tauri::{AppHandle, State};
use tracing::info;

use crate::device::PeerInfo;
use crate::error::AppResult;
use crate::identity::IdentityState;
use crate::protocol::{AppRequest, AppResponse, OsInfo};
use crate::workspace::state::DbState;

use super::config::create_node_config;
use super::event_loop::spawn_event_loop;
use super::online::AppNetClient;
use super::{NetManager, NetManagerState};

/// 启动 P2P 节点
#[tauri::command]
pub async fn start_p2p_node(
    app: AppHandle,
    identity: State<'_, IdentityState>,
    net_state: State<'_, NetManagerState>,
    db_state: State<'_, DbState>,
) -> AppResult<()> {
    let mut guard = net_state.lock().await;
    if guard.is_some() {
        return Err(crate::error::AppError::Network(
            "P2P node is already running".to_string(),
        ));
    }

    // 从 IdentityState 克隆 keypair
    let keypair = identity.keypair.clone();
    let peer_id = keypair.public().to_peer_id();

    // 构建 agent_version
    let os_info = OsInfo::default();
    let agent_version = os_info.to_agent_version(env!("CARGO_PKG_VERSION"));

    // 创建节点配置
    let config = create_node_config(agent_version);

    // 启动 P2P 节点
    let (client, receiver): (AppNetClient, _) =
        swarm_p2p_core::start::<AppRequest, AppResponse>(keypair, config)
            .map_err(|e| crate::error::AppError::Network(format!("Failed to start P2P: {e}")))?;

    // 创建 NetManager（传入 devices_db 供 PairingManager 使用）
    let net_manager = NetManager::new(client.clone(), peer_id, db_state.devices_db.clone());
    let cancel_token = net_manager.cancel_token();

    // 启动事件循环
    spawn_event_loop(
        receiver,
        app.clone(),
        net_manager.device_manager.clone(),
        net_manager.pairing_manager.clone(),
        cancel_token.clone(),
    );

    // 在线宣告 + DHT bootstrap + 已配对设备重连（后台任务）
    let announcer = net_manager.online_announcer.clone();
    let bootstrap_client = client.clone();
    let pairing_for_bootstrap = net_manager.pairing_manager.clone();

    tokio::spawn(async move {
        // 先 announce online
        if let Err(e) = announcer.announce_online().await {
            tracing::warn!("Failed to announce online: {e}");
        }

        // DHT bootstrap
        match bootstrap_client.bootstrap().await {
            Ok(_) => info!("DHT bootstrap completed"),
            Err(e) => tracing::warn!("DHT bootstrap failed: {e}"),
        }

        // 从 SQLite 读取已配对设备并 check_paired_online
        if let Err(e) = pairing_for_bootstrap.load_paired_devices().await {
            tracing::warn!("Failed to load paired devices: {e}");
        }
        let paired_peer_ids = pairing_for_bootstrap.get_paired_peer_ids();
        announcer.check_paired_online(paired_peer_ids).await;
    });

    // 启动周期续期
    net_manager
        .online_announcer
        .clone()
        .spawn_renewal_task(cancel_token);

    *guard = Some(net_manager);

    info!("P2P node started, PeerId: {peer_id}");
    Ok(())
}

/// 停止 P2P 节点
#[tauri::command]
pub async fn stop_p2p_node(net_state: State<'_, NetManagerState>) -> AppResult<()> {
    let mut guard = net_state.lock().await;
    if let Some(manager) = guard.take() {
        manager.shutdown().await;
        info!("P2P node stopped");
    }
    Ok(())
}

/// 获取已连接的 peers 列表
#[tauri::command]
pub async fn get_connected_peers(
    net_state: State<'_, NetManagerState>,
) -> AppResult<Vec<PeerInfo>> {
    let guard = net_state.lock().await;
    match guard.as_ref() {
        Some(manager) => Ok(manager.device_manager.get_connected_peers()),
        None => Ok(Vec::new()),
    }
}
