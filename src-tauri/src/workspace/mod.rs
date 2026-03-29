//! 工作区管理：数据库初始化、自动恢复、per-window 资源生命周期。

pub mod commands;
pub mod db;
pub mod state;

use std::collections::HashMap;
use std::path::PathBuf;

use entity::workspace::workspaces;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use tauri::Manager;

use crate::config::GlobalConfigState;
use crate::error::AppError;
use commands::WorkspaceInfo;
use state::{DbState, WorkspaceState};

/// 初始化数据库层：devices.db + 可选的工作区自动恢复（以 "main" 为 key）。
pub fn init(app: &tauri::AppHandle) -> Result<(), AppError> {
    let (devices_result, (workspace_db, workspace_info)) = tauri::async_runtime::block_on(async {
        tokio::join!(db::init_devices_db(), try_auto_restore_workspace(app))
    });
    let devices_db = devices_result?;

    let mut workspace_dbs = HashMap::new();
    let mut workspace_infos = HashMap::new();

    if let Some(conn) = workspace_db {
        workspace_dbs.insert("main".to_owned(), conn);
    }
    if let Some(info) = workspace_info {
        workspace_infos.insert("main".to_owned(), info);
    }

    app.manage(DbState::new(devices_db, workspace_dbs));
    app.manage(WorkspaceState::new(workspace_infos));

    Ok(())
}

/// 清理指定窗口的所有 per-window 资源。
pub async fn cleanup_window(
    app: &tauri::AppHandle,
    label: &str,
    db_state: &DbState,
    ws_state: &WorkspaceState,
    watcher_state: &crate::fs::watcher::FsWatcherState,
) {
    // Flush and close all Y.Doc instances for this window
    let ydoc_mgr = app.state::<crate::yjs::manager::YDocManager>();
    ydoc_mgr.close_all_for_window(app, label).await;

    crate::fs::watcher::stop_watching(label, watcher_state);

    if ws_state.remove(label).await {
        tracing::info!("Cleaned up WorkspaceInfo for window '{label}'");
    }
    if db_state.remove_workspace_db(label).await {
        tracing::info!("Cleaned up workspace DB for window '{label}'");
    }
}

/// 尝试从全局配置自动恢复上次打开的工作区。
async fn try_auto_restore_workspace(
    app: &tauri::AppHandle,
) -> (Option<DatabaseConnection>, Option<WorkspaceInfo>) {
    let config_state = match app.try_state::<GlobalConfigState>() {
        Some(s) => s,
        None => return (None, None),
    };

    let last_path = {
        let config = config_state.read().await;
        config.last_workspace_path.clone()
    };

    let path = match last_path {
        Some(p) if !p.is_empty() => p,
        _ => return (None, None),
    };

    let ws_path = PathBuf::from(&path);

    let conn = match db::init_workspace_db(&ws_path).await {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to open workspace db at {path}: {e}");
            return (None, None);
        }
    };

    let ws = match workspaces::Entity::find().one(&conn).await {
        Ok(Some(ws)) => ws,
        Ok(None) => {
            log::warn!("Workspace db exists but has no workspace record: {path}");
            return (None, None);
        }
        Err(e) => {
            log::warn!("Failed to query workspace record: {e}");
            return (None, None);
        }
    };

    let dir_name = commands::workspace_name_from_path(&ws_path);

    let ws = if ws.name != dir_name {
        let mut active: workspaces::ActiveModel = ws.into();
        active.name = Set(dir_name.clone());
        active.updated_at = Set(chrono::Utc::now().timestamp());
        match active.update(&conn).await {
            Ok(updated) => updated,
            Err(e) => {
                log::warn!("Failed to update workspace name: {e}");
                return (Some(conn), None);
            }
        }
    } else {
        ws
    };

    let info = WorkspaceInfo::from_model(&ws, &path);
    log::info!("Auto-restored workspace: {dir_name} ({path})");
    (Some(conn), Some(info))
}
