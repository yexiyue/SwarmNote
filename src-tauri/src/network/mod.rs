pub mod commands;
pub mod config;
pub mod dht_key;
pub mod event_loop;
pub mod online;

use std::sync::Arc;

use swarm_p2p_core::libp2p::PeerId;
use tokio_util::sync::CancellationToken;
use tracing::warn;

use crate::device::DeviceManager;

use self::online::{AppNetClient, OnlineAnnouncer};

/// 网络管理器：持有 P2P 节点所有运行时状态
pub struct NetManager {
    /// 供 #26 pairing / #28 sync 使用
    #[allow(dead_code)]
    pub client: AppNetClient,
    pub device_manager: Arc<DeviceManager>,
    pub online_announcer: Arc<OnlineAnnouncer>,
    cancel_token: CancellationToken,
}

impl NetManager {
    pub fn new(client: AppNetClient, peer_id: PeerId) -> Self {
        let device_manager = Arc::new(DeviceManager::new());
        let online_announcer = Arc::new(OnlineAnnouncer::new(client.clone(), peer_id));
        let cancel_token = CancellationToken::new();

        Self {
            client,
            device_manager,
            online_announcer,
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

/// NetManager 的 Tauri State 类型
pub type NetManagerState = tokio::sync::Mutex<Option<NetManager>>;
