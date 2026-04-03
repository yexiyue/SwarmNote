use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::task::JoinSet;
use uuid::Uuid;

use crate::device::{Device, DeviceFilter, DeviceListResult, DeviceStatus};
use crate::error::AppResult;
use crate::network::event_loop::events;
use crate::network::NetManagerState;
use crate::protocol::{
    AppRequest, AppResponse, OsInfo, PairingMethod, WorkspaceRequest, WorkspaceResponse,
};
use crate::workspace::state::WorkspaceState;

use super::code::PairingCodeInfo;
use super::manager::PairedDeviceInfo;

/// `get_device_by_code` 的类型化返回值。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceByCodeResult {
    pub peer_id: String,
    pub os_info: OsInfo,
}

// ── 配对命令 ──

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
        emit_devices(&app, &net_state).await;
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
        emit_devices(&app, &net_state).await;
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
    emit_devices(&app, &net_state).await;
    Ok(())
}

// ── 设备查询命令 ──

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
    Ok(dm
        .get_devices(DeviceFilter::Connected)
        .into_iter()
        .filter(|d| !d.is_paired)
        .collect())
}

// ── 工作区列表交换 ──

/// 远程工作区信息（合并来源 peer 信息）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteWorkspaceInfo {
    pub uuid: Uuid,
    pub name: String,
    pub doc_count: u32,
    pub updated_at: i64,
    pub peer_id: String,
    pub peer_name: String,
    pub is_local: bool,
}

/// 并发查询所有已配对在线 peer 的工作区列表，标记 is_local。
#[tauri::command]
pub async fn get_remote_workspaces(
    app: AppHandle,
    net_state: State<'_, NetManagerState>,
) -> AppResult<Vec<RemoteWorkspaceInfo>> {
    let client = net_state.client().await?;
    let dm = net_state.devices().await?;

    let paired_online: Vec<Device> = dm
        .get_devices(DeviceFilter::Paired)
        .into_iter()
        .filter(|d| d.status == DeviceStatus::Online)
        .collect();

    if paired_online.is_empty() {
        return Ok(Vec::new());
    }

    // 并发向所有 peer 发送 ListWorkspaces，5s 超时
    let mut tasks = JoinSet::new();
    for device in &paired_online {
        let peer_id_str = device.peer_id.clone();
        let peer_name = device.hostname.clone();
        let client = client.clone();
        tasks.spawn(async move {
            let Ok(peer_id) = peer_id_str.parse::<swarm_p2p_core::libp2p::PeerId>() else {
                return (peer_id_str, peer_name, None);
            };
            let result = tokio::time::timeout(
                Duration::from_secs(5),
                client.send_request(
                    peer_id,
                    AppRequest::Workspace(WorkspaceRequest::ListWorkspaces),
                ),
            )
            .await;
            let workspaces = match result {
                Ok(Ok(AppResponse::Workspace(WorkspaceResponse::WorkspaceList { workspaces }))) => {
                    Some(workspaces)
                }
                _ => None,
            };
            (peer_id_str, peer_name, workspaces)
        });
    }

    // 收集结果
    let local_uuids: std::collections::HashSet<Uuid> = match app.try_state::<WorkspaceState>() {
        Some(ws_state) => ws_state
            .list_all()
            .await
            .into_iter()
            .map(|info| info.id)
            .collect(),
        None => std::collections::HashSet::new(),
    };

    let mut results = Vec::new();
    while let Some(Ok((peer_id, peer_name, Some(workspaces)))) = tasks.join_next().await {
        for ws in workspaces {
            results.push(RemoteWorkspaceInfo {
                is_local: local_uuids.contains(&ws.uuid),
                uuid: ws.uuid,
                name: ws.name,
                doc_count: ws.doc_count,
                updated_at: ws.updated_at,
                peer_id: peer_id.clone(),
                peer_name: peer_name.clone(),
            });
        }
    }

    Ok(results)
}

// ── Helpers ──

/// 配对变更后 emit 完整设备列表，确保前端即时更新。
async fn emit_devices(app: &AppHandle, net_state: &NetManagerState) {
    if let Ok(dm) = net_state.devices().await {
        let devices = dm.get_devices(DeviceFilter::All);
        let _ = app.emit(events::DEVICES_CHANGED, &devices);
    }
}
