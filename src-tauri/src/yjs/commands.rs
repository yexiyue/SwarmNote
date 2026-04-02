use tauri::{Manager, State};
use uuid::Uuid;

use super::manager::{OpenDocResult, YDocManager};
use crate::error::{AppError, AppResult};
use crate::network::NetManagerState;

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

    // Notify SyncManager to subscribe GossipSub topic (best-effort)
    if let Some(net_state) = window.try_state::<NetManagerState>() {
        if let Ok(sync_mgr) = net_state.sync().await {
            sync_mgr.notify_doc_opened(result.doc_uuid).await;
        }
    }

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

    // Broadcast local edit to GossipSub (best-effort, non-blocking).
    // Safe from loops: this command is only invoked by the frontend for LOCAL edits.
    // Remote updates arrive via GossipSub → apply_sync_update (different path, no re-invoke).
    if let Some(net_state) = window.try_state::<NetManagerState>() {
        if let Ok(sync_mgr) = net_state.sync().await {
            let topic = format!("swarmnote/doc/{uuid}");
            if let Err(e) = sync_mgr.client.publish(&topic, update).await {
                tracing::warn!("Failed to publish update to GossipSub: {e}");
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

    // Notify SyncManager to unsubscribe GossipSub topic (best-effort)
    if let Some(net_state) = window.try_state::<NetManagerState>() {
        if let Ok(sync_mgr) = net_state.sync().await {
            sync_mgr.notify_doc_closed(uuid).await;
        }
    }

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
