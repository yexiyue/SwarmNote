//! P2P 网络层：节点启停、事件循环、DHT 在线宣告。

pub mod commands;
pub mod config;
pub mod dht_key;
pub mod event_loop;
pub mod online;

use std::sync::Arc;

use sea_orm::DatabaseConnection;
use serde::Serialize;
use swarm_p2p_core::libp2p::PeerId;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;
use tracing::warn;

use crate::device::DeviceManager;
use crate::pairing::PairingManager;
use crate::sync::SyncManager;

use self::online::{AppNetClient, OnlineAnnouncer};

/// P2P 节点状态——Rust/前端共用的 single source of truth
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum NodeStatus {
    Stopped,
    Running,
    #[expect(dead_code)]
    Error {
        message: String,
    },
}

/// 网络管理器：持有 P2P 节点所有运行时状态
pub struct NetManager {
    pub client: AppNetClient,
    pub device_manager: Arc<DeviceManager>,
    pub online_announcer: Arc<OnlineAnnouncer>,
    pub pairing_manager: Arc<PairingManager>,
    pub sync_manager: Arc<SyncManager>,
    cancel_token: CancellationToken,
}

impl NetManager {
    pub fn new(
        app: AppHandle,
        client: AppNetClient,
        peer_id: PeerId,
        db: DatabaseConnection,
        device_name: Option<String>,
    ) -> Self {
        let paired_devices = Arc::new(dashmap::DashMap::new());
        let device_manager = Arc::new(DeviceManager::new(paired_devices.clone()));
        let online_announcer = Arc::new(OnlineAnnouncer::new(client.clone(), peer_id));
        let pairing_manager = Arc::new(PairingManager::new(
            client.clone(),
            peer_id,
            db,
            paired_devices,
            device_name,
        ));
        let sync_manager = Arc::new(SyncManager::new(app, client.clone()));
        let cancel_token = CancellationToken::new();

        Self {
            client,
            device_manager,
            online_announcer,
            pairing_manager,
            sync_manager,
            cancel_token,
        }
    }

    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    /// 优雅关闭：通知 DHT 下线 → 取消后台任务
    pub async fn shutdown(&self) {
        // 尝试 announce_offline，但不阻塞关闭流程
        if let Err(e) = self.online_announcer.announce_offline().await {
            warn!("Failed to announce offline: {e}");
        }
        self.cancel_token.cancel();
    }
}

/// NetManager 的 Tauri State 类型（newtype 以支持便捷方法）
pub struct NetManagerState(tokio::sync::Mutex<Option<NetManager>>);

impl NetManagerState {
    pub fn new() -> Self {
        Self(tokio::sync::Mutex::new(None))
    }

    pub async fn lock(&self) -> tokio::sync::MutexGuard<'_, Option<NetManager>> {
        self.0.lock().await
    }

    /// 查询当前节点状态
    pub async fn status(&self) -> NodeStatus {
        if self.0.lock().await.is_some() {
            NodeStatus::Running
        } else {
            NodeStatus::Stopped
        }
    }

    /// 获取 PairingManager。节点未运行时返回错误。
    pub async fn pairing(&self) -> crate::error::AppResult<Arc<crate::pairing::PairingManager>> {
        let guard = self.0.lock().await;
        let manager = guard
            .as_ref()
            .ok_or_else(crate::error::AppError::node_not_running)?;
        Ok(manager.pairing_manager.clone())
    }

    /// 获取 NetClient。节点未运行时返回错误。
    pub async fn client(&self) -> crate::error::AppResult<online::AppNetClient> {
        let guard = self.0.lock().await;
        let manager = guard
            .as_ref()
            .ok_or_else(crate::error::AppError::node_not_running)?;
        Ok(manager.client.clone())
    }

    /// 获取 DeviceManager。节点未运行时返回错误。
    pub async fn devices(&self) -> crate::error::AppResult<Arc<DeviceManager>> {
        let guard = self.0.lock().await;
        let manager = guard
            .as_ref()
            .ok_or_else(crate::error::AppError::node_not_running)?;
        Ok(manager.device_manager.clone())
    }

    /// 获取 SyncManager。节点未运行时返回错误。
    pub async fn sync(&self) -> crate::error::AppResult<Arc<SyncManager>> {
        let guard = self.0.lock().await;
        let manager = guard
            .as_ref()
            .ok_or_else(crate::error::AppError::node_not_running)?;
        Ok(manager.sync_manager.clone())
    }
}
