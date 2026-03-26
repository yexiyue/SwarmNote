use tauri::State;

use crate::error::{AppError, AppResult};
use crate::workspace::state::WorkspaceState;

use super::FileTreeNode;

/// 从 per-window 状态中获取指定窗口的工作区路径。
async fn workspace_path_for(ws_state: &WorkspaceState, label: &str) -> Result<String, AppError> {
    ws_state
        .0
        .read()
        .await
        .get(label)
        .map(|ws| ws.path.clone())
        .ok_or(AppError::NoWorkspaceOpen)
}

#[tauri::command]
pub async fn scan_workspace_tree(
    window: tauri::Window,
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<Vec<FileTreeNode>> {
    let path = workspace_path_for(&ws_state, window.label()).await?;
    let ws_path = std::path::PathBuf::from(&path);

    tokio::task::spawn_blocking(move || super::scan::scan_workspace_tree(&ws_path))
        .await
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}

#[tauri::command]
pub async fn fs_create_file(
    window: tauri::Window,
    parent_rel: String,
    name: String,
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<String> {
    let path = workspace_path_for(&ws_state, window.label()).await?;
    let ws_path = std::path::PathBuf::from(&path);
    tokio::task::spawn_blocking(move || super::crud::create_file(&ws_path, &parent_rel, &name))
        .await
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}

#[tauri::command]
pub async fn fs_create_dir(
    window: tauri::Window,
    parent_rel: String,
    name: String,
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<String> {
    let path = workspace_path_for(&ws_state, window.label()).await?;
    let ws_path = std::path::PathBuf::from(&path);
    tokio::task::spawn_blocking(move || super::crud::create_dir(&ws_path, &parent_rel, &name))
        .await
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}

#[tauri::command]
pub async fn fs_delete_file(
    window: tauri::Window,
    rel_path: String,
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<()> {
    let path = workspace_path_for(&ws_state, window.label()).await?;
    let ws_path = std::path::PathBuf::from(&path);
    tokio::task::spawn_blocking(move || super::crud::delete_file(&ws_path, &rel_path))
        .await
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}

#[tauri::command]
pub async fn fs_delete_dir(
    window: tauri::Window,
    rel_path: String,
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<()> {
    let path = workspace_path_for(&ws_state, window.label()).await?;
    let ws_path = std::path::PathBuf::from(&path);
    tokio::task::spawn_blocking(move || super::crud::delete_dir(&ws_path, &rel_path))
        .await
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}

#[tauri::command]
pub async fn fs_rename(
    window: tauri::Window,
    rel_path: String,
    new_name: String,
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<String> {
    let path = workspace_path_for(&ws_state, window.label()).await?;
    let ws_path = std::path::PathBuf::from(&path);
    tokio::task::spawn_blocking(move || super::crud::rename(&ws_path, &rel_path, &new_name))
        .await
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}
