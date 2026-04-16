//! `AppCore` — device-level singleton. Holds identity, (future) P2P node,
//! paired devices, pairing manager, event bus, and the workspace registry.
//!
//! **Current state (PR #1):** only `identity` + `config` + `keychain` +
//! `event_bus` are wired. Network / device / pairing / ctrl dispatcher are
//! populated in PR #3; the `workspaces` registry is populated in PR #2.

use std::path::PathBuf;
use std::sync::Arc;

use crate::config::{load_or_create_config, GlobalConfigState};
use crate::error::AppResult;
use crate::events::EventBus;
use crate::identity::IdentityManager;
use crate::keychain::KeychainProvider;

/// Device-level core. Construct once per process via [`AppCore::new`].
///
/// Future fields (populated in subsequent PRs):
/// - `devices_db: DatabaseConnection` (paired device table)
/// - `device_manager: Arc<DeviceManager>`
/// - `pairing_manager: Arc<PairingManager>`
/// - `net: Arc<tokio::sync::Mutex<Option<NetManager>>>` (P2P node handle)
/// - `ctrl_dispatcher: Arc<CtrlMessageDispatcher>`
/// - `workspaces: tokio::sync::Mutex<HashMap<Uuid, Weak<WorkspaceCore>>>`
pub struct AppCore {
    pub identity: Arc<IdentityManager>,
    pub config: Arc<GlobalConfigState>,
    pub event_bus: Arc<dyn EventBus>,
    pub keychain: Arc<dyn KeychainProvider>,
}

impl AppCore {
    /// Bootstrap the device-level core.
    ///
    /// - `keychain`: host-provided secret storage (desktop: keyring, mobile:
    ///   Android Keystore / iOS Keychain).
    /// - `event_bus`: host-provided event sink.
    /// - `app_data_dir`: absolute path where the config file and `devices.db`
    ///   should live (desktop: `~/.swarmnote/`; mobile: documentDirectory).
    pub async fn new(
        keychain: Arc<dyn KeychainProvider>,
        event_bus: Arc<dyn EventBus>,
        app_data_dir: PathBuf,
    ) -> AppResult<Arc<Self>> {
        // 1. Load (or create default) global config.
        let config_data = load_or_create_config(&app_data_dir)?;
        let config_state = Arc::new(GlobalConfigState::new(
            config_data.clone(),
            app_data_dir.join("config.json"),
        ));

        // 2. Initialize device identity using the host-provided keychain.
        let identity = Arc::new(IdentityManager::new(keychain.clone(), &config_data).await?);

        Ok(Arc::new(Self {
            identity,
            config: config_state,
            event_bus,
            keychain,
        }))
    }
}
