use serde::Serialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::workspace::state::WorkspaceState;

use super::FileTreeNode;

#[derive(Debug, Serialize)]
pub struct SaveDocumentResult {
    /// blake3 hash hex string
    pub file_hash: String,
}

/// 从 per-window 状态中获取指定窗口的工作区路径。
async fn workspace_path_for(ws_state: &WorkspaceState, label: &str) -> Result<String, AppError> {
    ws_state.workspace_path_for(label).await
}

#[tauri::command]
pub async fn load_document(
    window: tauri::Window,
    rel_path: String,
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<String> {
    let path = workspace_path_for(&ws_state, window.label()).await?;
    let ws_path = std::path::PathBuf::from(&path);
    tokio::task::spawn_blocking(move || {
        super::crud::validate_rel_path(&ws_path, &rel_path)?;
        let full_path = ws_path.join(&rel_path);
        match std::fs::read_to_string(&full_path) {
            Ok(content) => Ok(content),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
            Err(e) => Err(e.into()),
        }
    })
    .await
    .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}

#[tauri::command]
pub async fn save_document(
    window: tauri::Window,
    rel_path: String,
    content: String,
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<SaveDocumentResult> {
    let path = workspace_path_for(&ws_state, window.label()).await?;
    let ws_path = std::path::PathBuf::from(&path);
    tokio::task::spawn_blocking(move || {
        super::crud::validate_rel_path(&ws_path, &rel_path)?;
        let full_path = ws_path.join(&rel_path);
        std::fs::write(&full_path, &content)?;
        let hash = blake3::hash(content.as_bytes());
        Ok(SaveDocumentResult {
            file_hash: hash.to_hex().to_string(),
        })
    })
    .await
    .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}

#[tauri::command]
pub async fn save_media(
    window: tauri::Window,
    rel_path: String,
    file_name: String,
    data: Vec<u8>,
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<String> {
    let path = workspace_path_for(&ws_state, window.label()).await?;
    let ws_path = std::path::PathBuf::from(&path);
    tokio::task::spawn_blocking(move || {
        super::crud::validate_rel_path(&ws_path, &rel_path)?;
        // Resource dir: "notes/my-note.md" → "notes/my-note.assets/"
        let rel = std::path::Path::new(&rel_path);
        let resource_dir = ws_path.join(format!("{}.assets", rel.with_extension("").display()));
        std::fs::create_dir_all(&resource_dir)?;

        // Hash-based filename: "screenshot.png" → "screenshot-af3b9e2c.png"
        let hash = blake3::hash(&data);
        let short_hash = &hash.to_hex()[..8];
        let stem = std::path::Path::new(&file_name)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        let ext = std::path::Path::new(&file_name)
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        let unique_name = format!("{stem}-{short_hash}{ext}");
        let actual_path = resource_dir.join(&unique_name);

        // Content-addressed: same hash = same file, skip if exists
        if !actual_path.exists() {
            std::fs::write(&actual_path, &data)?;
        }

        // Return workspace-relative path (not absolute)
        let ws_relative =
            pathdiff::diff_paths(&actual_path, &ws_path).unwrap_or_else(|| actual_path.clone());
        Ok(ws_relative.to_string_lossy().replace('\\', "/"))
    })
    .await
    .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
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
