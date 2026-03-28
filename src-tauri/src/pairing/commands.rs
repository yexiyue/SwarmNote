use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::device::PeerInfo;
use crate::error::{AppError, AppResult};
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
    let pairing = {
        let guard = net_state.lock().await;
        let manager = guard
            .as_ref()
            .ok_or(AppError::Network("P2P node is not running".to_string()))?;
        manager.pairing_manager.clone()
    };
    pairing.generate_code(expires_in_secs.unwrap_or(300)).await
}

#[tauri::command]
pub async fn get_device_by_code(
    net_state: State<'_, NetManagerState>,
    code: String,
) -> AppResult<DeviceByCodeResult> {
    let pairing = {
        let guard = net_state.lock().await;
        let manager = guard
            .as_ref()
            .ok_or(AppError::Network("P2P node is not running".to_string()))?;
        manager.pairing_manager.clone()
    };
    let (peer_id, record) = pairing.get_device_by_code(&code).await?;
    Ok(DeviceByCodeResult {
        peer_id,
        os_info: record.os_info,
    })
}

#[tauri::command]
pub async fn request_pairing(
    net_state: State<'_, NetManagerState>,
    peer_id: String,
    method: PairingMethod,
    remote_os_info: Option<OsInfo>,
) -> AppResult<crate::protocol::PairingResponse> {
    let pairing = {
        let guard = net_state.lock().await;
        let manager = guard
            .as_ref()
            .ok_or(AppError::Network("P2P node is not running".to_string()))?;
        manager.pairing_manager.clone()
    };
    pairing
        .request_pairing(&peer_id, method, remote_os_info)
        .await
}

#[tauri::command]
pub async fn respond_pairing_request(
    app: AppHandle,
    net_state: State<'_, NetManagerState>,
    pending_id: u64,
    accept: bool,
) -> AppResult<()> {
    let pairing = {
        let guard = net_state.lock().await;
        let manager = guard
            .as_ref()
            .ok_or(AppError::Network("P2P node is not running".to_string()))?;
        manager.pairing_manager.clone()
    };
    let result = pairing.handle_pairing_request(pending_id, accept).await?;
    if let Some(info) = result {
        let _ = app.emit(events::PAIRED_DEVICE_ADDED, &info);
    }
    Ok(())
}

#[tauri::command]
pub async fn get_paired_devices(
    net_state: State<'_, NetManagerState>,
) -> AppResult<Vec<PairedDeviceInfo>> {
    let pairing = {
        let guard = net_state.lock().await;
        match guard.as_ref() {
            Some(manager) => manager.pairing_manager.clone(),
            None => return Ok(Vec::new()),
        }
    };
    Ok(pairing.get_paired_devices())
}

#[tauri::command]
pub async fn unpair_device(
    app: AppHandle,
    net_state: State<'_, NetManagerState>,
    peer_id: String,
) -> AppResult<()> {
    let pairing = {
        let guard = net_state.lock().await;
        let manager = guard
            .as_ref()
            .ok_or(AppError::Network("P2P node is not running".to_string()))?;
        manager.pairing_manager.clone()
    };
    pairing.unpair(&peer_id).await?;
    let _ = app.emit(events::PAIRED_DEVICE_REMOVED, &peer_id);
    Ok(())
}

#[tauri::command]
pub async fn get_nearby_devices(net_state: State<'_, NetManagerState>) -> AppResult<Vec<PeerInfo>> {
    let (device_manager, pairing) = {
        let guard = net_state.lock().await;
        match guard.as_ref() {
            Some(manager) => (
                manager.device_manager.clone(),
                manager.pairing_manager.clone(),
            ),
            None => return Ok(Vec::new()),
        }
    };
    let all_peers = device_manager.get_connected_peers();
    let nearby: Vec<_> = all_peers
        .into_iter()
        .filter(|p| {
            let peer_id: Result<swarm_p2p_core::libp2p::PeerId, _> = p.peer_id.parse();
            match peer_id {
                Ok(pid) => !pairing.is_paired(&pid),
                Err(_) => true,
            }
        })
        .collect();
    Ok(nearby)
}
