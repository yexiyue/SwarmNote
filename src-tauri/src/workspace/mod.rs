pub mod commands;
pub mod db;
pub mod state;

use std::path::PathBuf;

use entity::workspace::workspaces;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use tauri::Manager;
use tokio::sync::RwLock;

use crate::config::GlobalConfigState;
use crate::error::AppError;
use commands::WorkspaceInfo;
use state::{DbState, WorkspaceState};

/// 初始化数据库层：devices.db + 可选的工作区自动恢复。
///
/// 通过 `tokio::join!` 并发执行两个初始化操作以减少启动延迟。
pub fn init(app: &tauri::AppHandle) -> Result<(), AppError> {
    let (devices_result, (workspace_db, workspace_info)) = tauri::async_runtime::block_on(async {
        tokio::join!(db::init_devices_db(), try_auto_restore_workspace(app))
    });
    let devices_db = devices_result?;

    app.manage(DbState {
        devices_db,
        workspace_db: RwLock::new(workspace_db),
    });
    app.manage(WorkspaceState(RwLock::new(workspace_info)));

    Ok(())
}

/// 尝试从全局配置自动恢复上次打开的工作区。
///
/// 任何失败均返回 `(None, None)` —— 不会 panic。
async fn try_auto_restore_workspace(
    app: &tauri::AppHandle,
) -> (Option<DatabaseConnection>, Option<WorkspaceInfo>) {
    let config_state = match app.try_state::<GlobalConfigState>() {
        Some(s) => s,
        None => return (None, None),
    };

    let last_path = {
        let config = config_state.0.read().await;
        config.last_workspace_path.clone()
    };

    let path = match last_path {
        Some(p) if !p.is_empty() => p,
        _ => return (None, None),
    };

    let ws_path = PathBuf::from(&path);

    // 跳过 TOCTOU 检查 —— 让 init_workspace_db 直接处理缺失文件
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

    // 如果目录被重命名则更新数据库中的名称（与 open_workspace 逻辑相同）
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
