use tauri::ipc::Channel;
use tauri::{Manager, State};
use uuid::Uuid;

use super::doc_state::{HydrateProgress, HydrateResult};
use super::manager::{OpenDocResult, YDocManager};
use crate::error::{AppError, AppResult};
use crate::network::NetManagerState;
use crate::workspace::state::WorkspaceState;

fn parse_doc_uuid(doc_uuid: &str) -> AppResult<Uuid> {
    doc_uuid
        .parse()
        .map_err(|e| AppError::Yjs(format!("invalid doc_uuid: {e}")))
}

#[tauri::command]
pub async fn open_ydoc(
    window: tauri::Window,
    rel_path: String,
    workspace_id: Uuid,
    ydoc_mgr: State<'_, YDocManager>,
) -> AppResult<OpenDocResult> {
    let result = ydoc_mgr
        .open_doc(window.app_handle(), window.label(), &rel_path, workspace_id)
        .await?;

    Ok(result)
}

#[tauri::command]
pub async fn apply_ydoc_update(
    window: tauri::Window,
    doc_uuid: String,
    update: Vec<u8>,
    ydoc_mgr: State<'_, YDocManager>,
) -> AppResult<()> {
    let uuid = parse_doc_uuid(&doc_uuid)?;
    ydoc_mgr.apply_update(window.label(), uuid, &update).await?;

    // Broadcast local edit to workspace GossipSub topic (best-effort, non-blocking).
    // Safe from loops: this command is only invoked by the frontend for LOCAL edits.
    // Remote updates arrive via GossipSub → handle_ws_gossip_update (different path).
    if let Some(net_state) = window.try_state::<NetManagerState>() {
        if let Ok(sync_mgr) = net_state.sync().await {
            if let Some(ws_uuid) = ydoc_mgr.workspace_uuid_for_doc(&uuid) {
                sync_mgr.publish_doc_update(ws_uuid, uuid, update).await;
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn close_ydoc(
    window: tauri::Window,
    doc_uuid: String,
    ydoc_mgr: State<'_, YDocManager>,
) -> AppResult<()> {
    let uuid = parse_doc_uuid(&doc_uuid)?;

    ydoc_mgr
        .close_doc(window.app_handle(), window.label(), uuid)
        .await
}

#[tauri::command]
pub async fn rename_ydoc(
    window: tauri::Window,
    doc_uuid: String,
    new_rel_path: String,
    ydoc_mgr: State<'_, YDocManager>,
) -> AppResult<()> {
    let uuid = parse_doc_uuid(&doc_uuid)?;
    ydoc_mgr.rename_doc(window.label(), uuid, &new_rel_path);
    Ok(())
}

#[tauri::command]
pub async fn reload_ydoc_confirmed(
    window: tauri::Window,
    doc_uuid: String,
    ydoc_mgr: State<'_, YDocManager>,
) -> AppResult<()> {
    let uuid = parse_doc_uuid(&doc_uuid)?;
    ydoc_mgr
        .reload_confirmed(window.app_handle(), window.label(), uuid)
        .await
}

#[tauri::command]
pub async fn hydrate_workspace(
    app: tauri::AppHandle,
    workspace_uuid: Uuid,
    on_progress: Channel<HydrateProgress>,
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<HydrateResult> {
    let ws_info = ws_state
        .get(&workspace_uuid)
        .await
        .ok_or(AppError::NoWorkspaceOpen)?;

    super::doc_state::hydrate_workspace(&app, workspace_uuid, &ws_info.path, &on_progress).await
}
