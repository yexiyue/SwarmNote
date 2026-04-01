use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::device::{Device, DeviceFilter, DeviceListResult};
use crate::error::AppResult;
use crate::network::event_loop::events;
use crate::network::NetManagerState;
use crate::protocol::{OsInfo, PairingMethod};

use super::code::PairingCodeInfo;
use super::manager::PairedDeviceInfo;

/// `get_device_by_code` 的类型化返回值。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceByCodeResult {
    pub peer_id: String,
    pub os_info: OsInfo,
}

#[tauri::command]
pub async fn generate_pairing_code(
    net_state: State<'_, NetManagerState>,
    expires_in_secs: Option<u64>,
) -> AppResult<PairingCodeInfo> {
    net_state
        .pairing()
        .await?
        .generate_code(expires_in_secs.unwrap_or(300))
        .await
}

#[tauri::command]
pub async fn get_device_by_code(
    net_state: State<'_, NetManagerState>,
    code: String,
) -> AppResult<DeviceByCodeResult> {
    let (peer_id, record) = net_state.pairing().await?.get_device_by_code(&code).await?;
    Ok(DeviceByCodeResult {
        peer_id,
        os_info: record.os_info,
    })
}

#[tauri::command]
pub async fn request_pairing(
    app: AppHandle,
    net_state: State<'_, NetManagerState>,
    peer_id: String,
    method: PairingMethod,
    remote_os_info: Option<OsInfo>,
) -> AppResult<crate::protocol::PairingResponse> {
    let resp = net_state
        .pairing()
        .await?
        .request_pairing(&peer_id, method, remote_os_info)
        .await?;

    if matches!(resp, crate::protocol::PairingResponse::Success) {
        let _ = app.emit(events::PAIRED_DEVICE_ADDED, ());
        if let Ok(dm) = net_state.devices().await {
            let devices = dm.get_devices(DeviceFilter::All);
            let _ = app.emit(events::DEVICES_CHANGED, &devices);
        }
    }

    Ok(resp)
}

#[tauri::command]
pub async fn respond_pairing_request(
    app: AppHandle,
    net_state: State<'_, NetManagerState>,
    pending_id: u64,
    accept: bool,
) -> AppResult<()> {
    let result = net_state
        .pairing()
        .await?
        .handle_pairing_request(pending_id, accept)
        .await?;
    if let Some(info) = result {
        let _ = app.emit(events::PAIRED_DEVICE_ADDED, &info);
        // 配对成功后 emit 完整设备列表，确保前端即时更新
        if let Ok(dm) = net_state.devices().await {
            let devices = dm.get_devices(DeviceFilter::All);
            let _ = app.emit(events::DEVICES_CHANGED, &devices);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn get_paired_devices(
    net_state: State<'_, NetManagerState>,
) -> AppResult<Vec<PairedDeviceInfo>> {
    match net_state.pairing().await {
        Ok(pairing) => Ok(pairing.get_paired_devices()),
        Err(_) => Ok(Vec::new()),
    }
}

#[tauri::command]
pub async fn unpair_device(
    app: AppHandle,
    net_state: State<'_, NetManagerState>,
    peer_id: String,
) -> AppResult<()> {
    net_state.pairing().await?.unpair(&peer_id).await?;
    let _ = app.emit(events::PAIRED_DEVICE_REMOVED, &peer_id);
    // 取消配对后 emit 完整设备列表
    if let Ok(dm) = net_state.devices().await {
        let devices = dm.get_devices(DeviceFilter::All);
        let _ = app.emit(events::DEVICES_CHANGED, &devices);
    }
    Ok(())
}

#[tauri::command]
pub async fn list_devices(
    net_state: State<'_, NetManagerState>,
    filter: Option<DeviceFilter>,
) -> AppResult<DeviceListResult> {
    let dm = net_state.devices().await?;
    let devices = dm.get_devices(filter.unwrap_or_default());
    let total = devices.len();
    Ok(DeviceListResult { devices, total })
}

#[tauri::command]
pub async fn get_nearby_devices(net_state: State<'_, NetManagerState>) -> AppResult<Vec<Device>> {
    let dm = match net_state.devices().await {
        Ok(dm) => dm,
        Err(_) => return Ok(Vec::new()),
    };

    let nearby: Vec<Device> = dm
        .get_devices(DeviceFilter::Connected)
        .into_iter()
        .filter(|d| !d.is_paired)
        .collect();
    Ok(nearby)
}
