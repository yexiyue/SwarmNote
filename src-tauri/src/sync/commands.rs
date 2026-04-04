use swarm_p2p_core::libp2p::PeerId;
use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::error::AppResult;
use crate::network::NetManagerState;

#[tauri::command]
pub async fn trigger_workspace_sync(
    _app: AppHandle,
    workspace_uuid: String,
    peer_id: String,
    net_state: State<'_, NetManagerState>,
) -> AppResult<()> {
    let sync_mgr = net_state.sync().await?;
    let uuid = Uuid::parse_str(&workspace_uuid)
        .map_err(|e| crate::error::AppError::Config(format!("Invalid UUID: {e}")))?;
    let pid: PeerId = peer_id
        .parse()
        .map_err(|e| crate::error::AppError::Config(format!("Invalid PeerId: {e}")))?;
    sync_mgr.spawn_full_sync(pid, uuid).await;
    Ok(())
}
