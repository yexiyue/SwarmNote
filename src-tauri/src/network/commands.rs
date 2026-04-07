use sea_orm::DatabaseConnection;
use swarm_p2p_core::libp2p::identity::Keypair;
use tauri::{AppHandle, Emitter, Manager, State};
use tracing::info;

use crate::device::{Device, DeviceFilter};
use crate::error::{AppError, AppResult};
use crate::identity::IdentityState;
use crate::protocol::{AppRequest, AppResponse, OsInfo};
use crate::workspace::state::DbState;

use super::config::create_node_config;
use super::event_loop::spawn_event_loop;
use super::online::AppNetClient;
use super::{NetManager, NetManagerState};

/// 启动 P2P 节点的核心逻辑（供 Tauri command 和 setup 自动启动共用）
pub async fn do_start_p2p_node(
    app: &AppHandle,
    net_state: &NetManagerState,
    keypair: Keypair,
    db: DatabaseConnection,
) -> AppResult<()> {
    let mut guard = net_state.lock().await;
    if guard.is_some() {
        return Err(AppError::Network("P2P node is already running".to_string()));
    }

    let peer_id = keypair.public().to_peer_id();

    // 读取用户自定义设备名称
    let identity_state = app.state::<IdentityState>();
    let device_name = identity_state
        .device_info
        .read()
        .map(|info| info.device_name.clone())
        .unwrap_or_default();

    // 构建 agent_version（PairingManager 也使用 device_name 构建一致的 OsInfo）
    let mut os_info = OsInfo::default();
    if device_name != os_info.hostname {
        os_info.name = Some(device_name.clone());
    }
    let agent_version = os_info.to_agent_version(env!("CARGO_PKG_VERSION"));
    let config = create_node_config(agent_version);

    // 启动 P2P 节点
    let (client, receiver): (AppNetClient, _) =
        swarm_p2p_core::start::<AppRequest, AppResponse>(keypair, config)
            .map_err(|e| AppError::Network(format!("Failed to start P2P: {e}")))?;

    let net_manager = NetManager::new(app.clone(), client.clone(), peer_id, db, os_info.name);
    let cancel_token = net_manager.cancel_token();

    // 启动事件循环
    spawn_event_loop(
        receiver,
        app.clone(),
        net_manager.client.clone(),
        net_manager.device_manager.clone(),
        net_manager.pairing_manager.clone(),
        net_manager.sync_manager.clone(),
        cancel_token.clone(),
    );

    // Start periodic SV compensation (60s interval, cancelled on node shutdown)
    net_manager
        .sync_manager
        .start_sv_compensation(cancel_token.clone());

    // 在线宣告 + DHT bootstrap + 已配对设备重连（后台任务）
    let announcer = net_manager.online_announcer.clone();
    let bootstrap_client = client.clone();
    let pairing_for_bootstrap = net_manager.pairing_manager.clone();

    tokio::spawn(async move {
        if let Err(e) = announcer.announce_online().await {
            tracing::warn!("Failed to announce online: {e}");
        }

        match bootstrap_client.bootstrap().await {
            Ok(_) => info!("DHT bootstrap completed"),
            Err(e) => tracing::warn!("DHT bootstrap failed: {e}"),
        }

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

    // 通知前端节点已启动
    let _ = app.emit("node-started", ());

    // 更新托盘状态
    #[cfg(desktop)]
    if let Some(tray) = app.try_state::<crate::tray::TrayManagerState>() {
        tray.lock()
            .await
            .set_status(crate::tray::TrayNodeStatus::Running { peer_count: 0 });
    }

    info!("P2P node started, PeerId: {peer_id}");
    Ok(())
}

/// 启动 P2P 节点（Tauri command）
#[tauri::command]
pub async fn start_p2p_node(
    app: AppHandle,
    identity: State<'_, IdentityState>,
    net_state: State<'_, NetManagerState>,
    db_state: State<'_, DbState>,
) -> AppResult<()> {
    do_start_p2p_node(
        &app,
        &net_state,
        identity.keypair.clone(),
        db_state.devices_db.clone(),
    )
    .await
}

/// 停止 P2P 节点
#[tauri::command]
pub async fn stop_p2p_node(app: AppHandle, net_state: State<'_, NetManagerState>) -> AppResult<()> {
    let mut guard = net_state.lock().await;
    if let Some(manager) = guard.take() {
        manager.shutdown().await;

        // 通知前端节点已停止
        let _ = app.emit("node-stopped", ());

        // 更新托盘状态
        #[cfg(desktop)]
        if let Some(tray) = app.try_state::<crate::tray::TrayManagerState>() {
            tray.lock()
                .await
                .set_status(crate::tray::TrayNodeStatus::Stopped);
        }

        info!("P2P node stopped");
    }
    Ok(())
}

/// 查询 P2P 节点当前运行状态
#[tauri::command]
pub async fn get_network_status(
    net_state: State<'_, NetManagerState>,
) -> AppResult<super::NodeStatus> {
    Ok(net_state.status().await)
}

/// 获取已连接的设备列表
#[tauri::command]
pub async fn get_connected_peers(net_state: State<'_, NetManagerState>) -> AppResult<Vec<Device>> {
    match net_state.devices().await {
        Ok(dm) => Ok(dm.get_devices(DeviceFilter::Connected)),
        Err(_) => Ok(Vec::new()),
    }
}
