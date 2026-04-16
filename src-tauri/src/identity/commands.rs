use std::sync::Arc;

use swarmnote_core::AppCore;
use tauri::{Manager, State};
use tracing::info;

use crate::config::GlobalConfigState;
use crate::error::AppResult;
use crate::identity::{IdentityError, IdentityState};
use crate::network::commands::do_start_p2p_node;
use crate::network::NetManagerState;
use crate::workspace::state::DbState;
use swarmnote_core::DeviceInfo;

/// Return current device info.
///
/// PR #1: canonical source is `AppCore.identity`. The legacy `IdentityState`
/// (still registered for the P2P layer) is kept in sync by `set_device_name`
/// below; it will be removed in PR #3 when the network module moves to core.
#[tauri::command]
pub fn get_device_info(core: State<'_, Arc<AppCore>>) -> AppResult<DeviceInfo> {
    Ok(core.identity.device_info()?)
}

#[tauri::command]
pub async fn set_device_name(
    name: String,
    app: tauri::AppHandle,
    state: State<'_, IdentityState>,
    config_state: State<'_, GlobalConfigState>,
    core: State<'_, Arc<AppCore>>,
) -> AppResult<()> {
    // Update legacy `IdentityState.device_info` (still used by the P2P layer
    // in src-tauri/src/network/). Removed in PR #3 once network moves.
    {
        let mut info = state
            .device_info
            .write()
            .map_err(|e| IdentityError::Config(format!("lock error: {e}")))?;
        info.device_name = name.clone();
    }

    // Update the core-side canonical snapshot so `get_device_info` returns
    // fresh data immediately.
    core.identity.set_device_name(name.clone())?;

    // Persist to the on-disk config (single source of truth). We keep writing
    // through the legacy `GlobalConfigState` for now — `AppCore.config` will
    // take over when the remaining consumers migrate.
    let mut config = config_state.write().await;
    config.device_name = name.clone();
    crate::config::save_config(&config)?;
    drop(config);

    // Also mirror into AppCore.config so other core-side consumers see the
    // new name without a reload.
    {
        let mut core_cfg = core.config.write().await;
        core_cfg.device_name = name.clone();
    }

    let keypair = state.keypair.clone();

    // Restart P2P node if running so the new name propagates via Identify.
    let net_state = app.state::<NetManagerState>();
    let was_running = {
        let mut guard = net_state.lock().await;
        if let Some(manager) = guard.take() {
            manager.shutdown().await;
            true
        } else {
            false
        }
    };

    if was_running {
        info!("Restarting P2P node to propagate new device name: {name}");
        let db_state = app.state::<DbState>();
        let _ = do_start_p2p_node(&app, &net_state, keypair, db_state.devices_db.clone()).await;
    }

    Ok(())
}
