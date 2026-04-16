//! 设备身份管理：Ed25519 密钥对、PeerId、设备信息。

pub mod commands;
pub mod keychain;

use std::sync::RwLock;
use swarm_p2p_core::libp2p::identity::Keypair;
use tauri::Manager;

// Canonical DeviceInfo lives in `swarmnote_core`. Re-exported here so existing
// `crate::identity::DeviceInfo` imports keep resolving to the same nominal
// type used by `AppCore`.
pub use swarmnote_core::DeviceInfo;

/// 身份操作相关的错误类型。
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("keychain error: {0}")]
    Keychain(String),
    #[error("keypair decode error: {0}")]
    KeypairDecode(String),
    #[error("keypair encode error: {0}")]
    KeypairEncode(String),
    #[error("config error: {0}")]
    Config(String),
}

/// 运行时身份状态，存储在 Tauri State 中。
pub struct IdentityState {
    pub keypair: Keypair,
    pub device_info: RwLock<DeviceInfo>,
}

impl IdentityState {
    /// 获取当前设备的 PeerId 字符串。
    pub fn peer_id(&self) -> crate::error::AppResult<String> {
        let info = self
            .device_info
            .read()
            .map_err(|e| IdentityError::Config(format!("lock error: {e}")))?;
        Ok(info.peer_id.clone())
    }
}

/// 在 Tauri 启动阶段初始化设备身份。
///
/// 1. 从系统钥匙串加载或生成 Ed25519 密钥对
/// 2. 从公钥派生 PeerId
/// 3. 加载或创建设备配置（device_name、created_at）
/// 4. 将 IdentityState 和 GlobalConfigState 注册到 Tauri State
pub fn init(app: &tauri::AppHandle) -> Result<(), crate::error::AppError> {
    let keypair = keychain::load_or_generate_keypair()?;
    let peer_id = keypair.public().to_peer_id().to_string();
    let config = crate::config::load_or_create_config()?;

    let device_info = DeviceInfo {
        peer_id,
        device_name: config.device_name.clone(),
        hostname: hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_default(),
        os: std::env::consts::OS.to_string(),
        platform: std::env::consts::FAMILY.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        created_at: config.created_at.clone(),
    };

    log::info!(
        "Device identity initialized: PeerId={}",
        device_info.peer_id
    );

    app.manage(IdentityState {
        keypair,
        device_info: RwLock::new(device_info),
    });

    let config_path = crate::config::swarmnote_global_dir()?.join("config.json");
    app.manage(crate::config::GlobalConfigState::new(config, config_path));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keypair_generation_produces_valid_peer_id() {
        let keypair = Keypair::generate_ed25519();
        let peer_id = keypair.public().to_peer_id().to_string();

        // libp2p PeerId 以 "12D3KooW" 开头
        assert!(
            peer_id.starts_with("12D3KooW"),
            "PeerId should start with 12D3KooW, got: {peer_id}"
        );
    }

    #[test]
    fn same_keypair_produces_same_peer_id() {
        let keypair = Keypair::generate_ed25519();
        let pid1 = keypair.public().to_peer_id().to_string();
        let pid2 = keypair.public().to_peer_id().to_string();
        assert_eq!(pid1, pid2);
    }

    #[test]
    fn keypair_protobuf_roundtrip() {
        let keypair = Keypair::generate_ed25519();
        let bytes = keypair.to_protobuf_encoding().unwrap();
        let restored = Keypair::from_protobuf_encoding(&bytes).unwrap();

        assert_eq!(
            keypair.public().to_peer_id(),
            restored.public().to_peer_id(),
            "Roundtripped keypair should produce the same PeerId"
        );
    }

    #[test]
    fn device_info_serialization() {
        let info = DeviceInfo {
            peer_id: "12D3KooWTest".to_string(),
            device_name: "Test Device".to_string(),
            hostname: "DESKTOP-TEST".to_string(),
            os: "windows".to_string(),
            platform: "windows".to_string(),
            arch: "x86_64".to_string(),
            created_at: "2026-03-21T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        let restored: DeviceInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info.peer_id, restored.peer_id);
        assert_eq!(info.device_name, restored.device_name);
    }
}
