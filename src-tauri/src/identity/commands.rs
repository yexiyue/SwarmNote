use tauri::State;

use super::{DeviceInfo, IdentityError, IdentityState};
use crate::config::GlobalConfigState;
use crate::error::AppResult;

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
    state: State<'_, IdentityState>,
    config_state: State<'_, GlobalConfigState>,
) -> AppResult<()> {
    {
        let mut info = state
            .device_info
            .write()
            .map_err(|e| IdentityError::Config(format!("lock error: {e}")))?;
        info.device_name = name;
    }

    let mut config = config_state.0.write().await;
    let info = state
        .device_info
        .read()
        .map_err(|e| IdentityError::Config(format!("lock error: {e}")))?;
    config.device_name = info.device_name.clone();
    crate::config::save_config(&config)?;

    Ok(())
}
