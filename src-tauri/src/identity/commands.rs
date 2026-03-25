use tauri::State;

use super::{DeviceInfo, IdentityState};
use crate::config::GlobalConfigState;
use crate::error::{AppError, AppResult};

#[tauri::command]
pub fn get_device_info(state: State<'_, IdentityState>) -> AppResult<DeviceInfo> {
    let info = state
        .device_info
        .read()
        .map_err(|e| AppError::Identity(format!("lock error: {e}")))?;
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
            .map_err(|e| AppError::Identity(format!("lock error: {e}")))?;
        info.device_name = name;
    }

    // 更新内存中的配置并持久化到磁盘
    let mut config = config_state.0.write().await;
    let info = state
        .device_info
        .read()
        .map_err(|e| AppError::Identity(format!("lock error: {e}")))?;
    config.device_name = info.device_name.clone();
    crate::config::save_config(&config)?;

    Ok(())
}
