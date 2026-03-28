use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use swarm_p2p_core::libp2p::core::Multiaddr;
use swarm_p2p_core::libp2p::PeerId;

use crate::protocol::OsInfo;

/// 运行时 peer 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub hostname: String,
    pub os: String,
    pub platform: String,
    pub arch: String,
    pub is_connected: bool,
    pub rtt_ms: Option<u64>,
    pub connection_type: Option<String>,
    #[serde(skip)]
    pub addrs: Vec<Multiaddr>,
}

/// 管理运行时设备发现和连接状态
pub struct DeviceManager {
    peers: DashMap<PeerId, PeerInfo>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            peers: DashMap::new(),
        }
    }

    /// 添加发现的 peers（来自 mDNS / DHT），不更新连接状态
    pub fn add_peers(&self, peers: Vec<(PeerId, Multiaddr)>) {
        for (peer_id, addr) in peers {
            self.peers
                .entry(peer_id)
                .and_modify(|info| {
                    if !info.addrs.contains(&addr) {
                        info.addrs.push(addr.clone());
                    }
                })
                .or_insert_with(|| PeerInfo {
                    peer_id: peer_id.to_string(),
                    hostname: String::new(),
                    os: String::new(),
                    platform: String::new(),
                    arch: String::new(),
                    is_connected: false,
                    rtt_ms: None,
                    connection_type: None,
                    addrs: vec![addr],
                });
        }
    }

    /// 标记 peer 已连接
    pub fn set_connected(&self, peer_id: &PeerId) {
        if let Some(mut info) = self.peers.get_mut(peer_id) {
            info.is_connected = true;
        } else {
            self.peers.insert(
                *peer_id,
                PeerInfo {
                    peer_id: peer_id.to_string(),
                    hostname: String::new(),
                    os: String::new(),
                    platform: String::new(),
                    arch: String::new(),
                    is_connected: true,
                    rtt_ms: None,
                    connection_type: None,
                    addrs: Vec::new(),
                },
            );
        }
    }

    /// 标记 peer 已断开
    pub fn set_disconnected(&self, peer_id: &PeerId) {
        if let Some(mut info) = self.peers.get_mut(peer_id) {
            info.is_connected = false;
            info.rtt_ms = None;
        }
    }

    /// 设置 peer 的 agent_version，解析 OsInfo。
    /// 返回 true 如果是 SwarmNote 设备，false 否则（非 SwarmNote 设备会被移除）。
    pub fn set_agent_version(&self, peer_id: &PeerId, agent_version: &str) -> bool {
        match OsInfo::from_agent_version(agent_version) {
            Some(os_info) => {
                if let Some(mut info) = self.peers.get_mut(peer_id) {
                    info.hostname = os_info.hostname;
                    info.os = os_info.os;
                    info.platform = os_info.platform;
                    info.arch = os_info.arch;
                }
                true
            }
            None => {
                // 非 SwarmNote 设备，移除
                self.peers.remove(peer_id);
                false
            }
        }
    }

    /// 更新 RTT
    pub fn update_rtt(&self, peer_id: &PeerId, rtt_ms: u64) {
        if let Some(mut info) = self.peers.get_mut(peer_id) {
            info.rtt_ms = Some(rtt_ms);
        }
    }

    /// 获取所有已连接的 SwarmNote peers
    pub fn get_connected_peers(&self) -> Vec<PeerInfo> {
        self.peers
            .iter()
            .filter(|entry| entry.value().is_connected)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// 获取指定 peer 的信息
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<PeerInfo> {
        self.peers.get(peer_id).map(|entry| entry.value().clone())
    }

    /// 获取已连接的 peer 数量
    pub fn connected_count(&self) -> usize {
        self.peers
            .iter()
            .filter(|entry| entry.value().is_connected)
            .count()
    }
}
