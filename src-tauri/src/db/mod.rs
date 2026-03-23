pub mod commands;
pub mod state;

use std::path::{Path, PathBuf};

use entity::workspace::workspaces;
use migration::MigratorTrait;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use tauri::Manager;
use tokio::sync::RwLock;

use crate::error::AppError;
use crate::identity::GlobalConfigState;
use commands::WorkspaceInfo;
use migration::{DevicesMigrator, WorkspaceMigrator};
use state::{DbState, WorkspaceState};

fn swarmnote_global_dir() -> Result<PathBuf, AppError> {
    let home = directories::BaseDirs::new().ok_or(AppError::NoAppDataDir)?;
    Ok(home.home_dir().join(".swarmnote"))
}

async fn connect_sqlite(path: &Path) -> Result<DatabaseConnection, AppError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let url = format!("sqlite:{}?mode=rwc", path.display());
    Ok(Database::connect(&url).await?)
}

pub async fn init_devices_db() -> Result<DatabaseConnection, AppError> {
    let db_path = swarmnote_global_dir()?.join("devices.db");
    let db = connect_sqlite(&db_path).await?;
    DevicesMigrator::up(&db, None).await?;
    Ok(db)
}

pub async fn init_workspace_db(workspace_path: &Path) -> Result<DatabaseConnection, AppError> {
    let db_path = workspace_path.join(".swarmnote").join("workspace.db");
    let db = connect_sqlite(&db_path).await?;
    WorkspaceMigrator::up(&db, None).await?;
    Ok(db)
}

/// Initialize database layer: devices.db + optional workspace auto-restore.
///
/// Runs both init operations concurrently via `tokio::join!` to reduce startup latency.
pub fn init(app: &tauri::AppHandle) -> Result<(), AppError> {
    let (devices_result, (workspace_db, workspace_info)) = tauri::async_runtime::block_on(async {
        tokio::join!(init_devices_db(), try_auto_restore_workspace(app))
    });
    let devices_db = devices_result?;

    app.manage(DbState {
        devices_db,
        workspace_db: RwLock::new(workspace_db),
    });
    app.manage(WorkspaceState(RwLock::new(workspace_info)));

    Ok(())
}

/// Attempt to auto-restore the last workspace from global config.
///
/// Returns `(None, None)` on any failure — never panics.
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

    // Skip TOCTOU check — let init_workspace_db handle missing files directly
    let conn = match init_workspace_db(&ws_path).await {
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

    // Update name in db if directory was renamed (same logic as open_workspace)
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
