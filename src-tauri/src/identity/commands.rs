use tauri::State;

use super::{DeviceInfo, IdentityState};

#[tauri::command]
pub fn get_device_info(state: State<'_, IdentityState>) -> Result<DeviceInfo, String> {
    let info = state
        .device_info
        .read()
        .map_err(|e| format!("lock error: {e}"))?;
    Ok(info.clone())
}

#[tauri::command]
pub fn set_device_name(name: String, state: State<'_, IdentityState>) -> Result<(), String> {
    let mut info = state
        .device_info
        .write()
        .map_err(|e| format!("lock error: {e}"))?;

    info.device_name = name;

    let config = super::config::DeviceConfig {
        device_name: info.device_name.clone(),
        created_at: info.created_at.clone(),
    };
    super::config::save_config(&config).map_err(|e| e.to_string())?;

    Ok(())
}
