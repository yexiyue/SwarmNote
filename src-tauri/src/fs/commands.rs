use tauri::State;

use crate::db::state::WorkspaceState;
use crate::error::{AppError, AppResult};

use super::FileTreeNode;

/// Get the workspace path from state, or error.
async fn workspace_path(ws_state: &WorkspaceState) -> Result<String, AppError> {
    ws_state
        .0
        .read()
        .await
        .as_ref()
        .map(|ws| ws.path.clone())
        .ok_or(AppError::NoWorkspaceOpen)
}

#[tauri::command]
pub async fn scan_workspace_tree(
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<Vec<FileTreeNode>> {
    let path = workspace_path(&ws_state).await?;
    let ws_path = std::path::PathBuf::from(&path);

    // Run blocking FS scan on a background thread
    tokio::task::spawn_blocking(move || super::scan::scan_workspace_tree(&ws_path))
        .await
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}

#[tauri::command]
pub async fn fs_create_file(
    parent_rel: String,
    name: String,
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<String> {
    let path = workspace_path(&ws_state).await?;
    let ws_path = std::path::PathBuf::from(&path);
    tokio::task::spawn_blocking(move || super::crud::create_file(&ws_path, &parent_rel, &name))
        .await
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}

#[tauri::command]
pub async fn fs_create_dir(
    parent_rel: String,
    name: String,
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<String> {
    let path = workspace_path(&ws_state).await?;
    let ws_path = std::path::PathBuf::from(&path);
    tokio::task::spawn_blocking(move || super::crud::create_dir(&ws_path, &parent_rel, &name))
        .await
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}

#[tauri::command]
pub async fn fs_delete_file(
    rel_path: String,
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<()> {
    let path = workspace_path(&ws_state).await?;
    let ws_path = std::path::PathBuf::from(&path);
    tokio::task::spawn_blocking(move || super::crud::delete_file(&ws_path, &rel_path))
        .await
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}

#[tauri::command]
pub async fn fs_delete_dir(rel_path: String, ws_state: State<'_, WorkspaceState>) -> AppResult<()> {
    let path = workspace_path(&ws_state).await?;
    let ws_path = std::path::PathBuf::from(&path);
    tokio::task::spawn_blocking(move || super::crud::delete_dir(&ws_path, &rel_path))
        .await
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}

#[tauri::command]
pub async fn fs_rename(
    rel_path: String,
    new_name: String,
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<String> {
    let path = workspace_path(&ws_state).await?;
    let ws_path = std::path::PathBuf::from(&path);
    tokio::task::spawn_blocking(move || super::crud::rename(&ws_path, &rel_path, &new_name))
        .await
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}
