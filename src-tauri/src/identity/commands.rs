use tauri::{Manager, State};
use tracing::info;

use super::{DeviceInfo, IdentityError, IdentityState};
use crate::config::GlobalConfigState;
use crate::error::AppResult;
use crate::network::commands::do_start_p2p_node;
use crate::network::NetManagerState;
use crate::workspace::state::DbState;

#[tauri::command]
pub fn get_device_info(state: State<'_, IdentityState>) -> AppResult<DeviceInfo> {
    let info = state
        .device_info
        .read()
        .map_err(|e| IdentityError::Config(format!("lock error: {e}")))?;
    Ok(info.clone())
}

#[tauri::command]
pub async fn set_device_name(
    name: String,
    app: tauri::AppHandle,
    state: State<'_, IdentityState>,
    config_state: State<'_, GlobalConfigState>,
) -> AppResult<()> {
    // Update device info in memory
    {
        let mut info = state
            .device_info
            .write()
            .map_err(|e| IdentityError::Config(format!("lock error: {e}")))?;
        info.device_name = name.clone();
    }

    // Persist to config
    let mut config = config_state.write().await;
    config.device_name = name.clone();
    crate::config::save_config(&config)?;
    drop(config);

    // Clone before async (State refs are not Send)
    let keypair = state.keypair.clone();

    // Restart P2P node if running so the new name propagates via Identify
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
