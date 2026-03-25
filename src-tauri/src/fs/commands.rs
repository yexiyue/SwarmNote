use tauri::State;

use crate::error::{AppError, AppResult};
use crate::workspace::state::WorkspaceState;

use super::FileTreeNode;

/// 从状态中获取工作区路径，不存在则返回错误。
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

    // 在后台线程执行阻塞式文件系统扫描
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
