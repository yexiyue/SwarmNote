pub mod commands;
pub mod config;
pub mod keychain;

use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use swarm_p2p_core::libp2p::identity::Keypair;
use tauri::Manager;

/// Errors from identity operations.
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

impl From<IdentityError> for String {
    fn from(e: IdentityError) -> Self {
        e.to_string()
    }
}

/// Runtime identity state, stored in Tauri State.
pub struct IdentityState {
    /// Used by the P2P network layer (Phase 1).
    #[allow(dead_code)]
    pub keypair: Keypair,
    pub device_info: RwLock<DeviceInfo>,
}

/// Device info returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub peer_id: String,
    pub device_name: String,
    pub os: String,
    pub platform: String,
    pub arch: String,
    pub created_at: String,
}

/// Initialize device identity during Tauri setup.
///
/// 1. Load or generate Ed25519 keypair from system keychain
/// 2. Derive PeerId from public key
/// 3. Load or create device config (device_name, created_at)
/// 4. Register IdentityState in Tauri State
pub fn init(app: &tauri::AppHandle) -> Result<(), IdentityError> {
    let keypair = keychain::load_or_generate_keypair()?;
    let peer_id = keypair.public().to_peer_id().to_string();
    let config = config::load_or_create_config()?;

    let device_info = DeviceInfo {
        peer_id,
        device_name: config.device_name,
        os: std::env::consts::OS.to_string(),
        platform: std::env::consts::FAMILY.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        created_at: config.created_at,
    };

    log::info!(
        "Device identity initialized: PeerId={}",
        device_info.peer_id
    );

    app.manage(IdentityState {
        keypair,
        device_info: RwLock::new(device_info),
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keypair_generation_produces_valid_peer_id() {
        let keypair = Keypair::generate_ed25519();
        let peer_id = keypair.public().to_peer_id().to_string();

        // libp2p PeerId starts with "12D3KooW"
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
