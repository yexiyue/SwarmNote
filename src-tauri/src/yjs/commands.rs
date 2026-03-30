use tauri::{Manager, State};
use uuid::Uuid;

use super::manager::{OpenDocResult, YDocManager};
use crate::error::{AppError, AppResult};

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
    asset_url_prefix: String,
    ydoc_mgr: State<'_, YDocManager>,
) -> AppResult<OpenDocResult> {
    ydoc_mgr
        .open_doc(
            window.app_handle(),
            window.label(),
            &rel_path,
            workspace_id,
            &asset_url_prefix,
        )
        .await
}

#[tauri::command]
pub async fn apply_ydoc_update(
    window: tauri::Window,
    doc_uuid: String,
    update: Vec<u8>,
    ydoc_mgr: State<'_, YDocManager>,
) -> AppResult<()> {
    let uuid = parse_doc_uuid(&doc_uuid)?;
    ydoc_mgr.apply_update(window.label(), uuid, &update).await
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
