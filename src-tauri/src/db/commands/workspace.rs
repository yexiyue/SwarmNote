use std::path::{Path, PathBuf};

use chrono::Utc;
use entity::workspace::workspaces;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use super::peer_id;
use crate::db::init_workspace_db;
use crate::db::state::{DbState, WorkspaceState};
use crate::error::{AppError, AppResult};
use crate::identity::{GlobalConfigState, IdentityState};

/// Workspace info returned to the frontend, combining db record + runtime path.
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub created_by: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl WorkspaceInfo {
    pub fn from_model(model: &workspaces::Model, path: &str) -> Self {
        Self {
            id: model.id.clone(),
            name: model.name.clone(),
            path: path.to_owned(),
            created_by: model.created_by.clone(),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

/// Extract workspace name from the last path component.
pub(crate) fn workspace_name_from_path(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Workspace".to_owned())
}

/// Idempotent workspace open/create command.
///
/// - If `.swarmnote/workspace.db` exists: open and return existing info.
/// - Otherwise: create `.swarmnote/`, init db, insert workspace record.
/// - Updates global config with `last_workspace_path` on success.
#[tauri::command]
pub async fn open_workspace(
    path: String,
    db_state: State<'_, DbState>,
    identity: State<'_, IdentityState>,
    config_state: State<'_, GlobalConfigState>,
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<WorkspaceInfo> {
    let ws_path = PathBuf::from(&path);

    if !ws_path.is_dir() {
        return Err(AppError::InvalidPath(format!(
            "Directory does not exist: {path}"
        )));
    }

    let conn = init_workspace_db(&ws_path).await?;
    let dir_name = workspace_name_from_path(&ws_path);

    let workspace = match workspaces::Entity::find().one(&conn).await? {
        Some(mut ws) => {
            // Update name if directory was renamed
            if ws.name != dir_name {
                let mut active: workspaces::ActiveModel = ws.clone().into();
                active.name = Set(dir_name.clone());
                active.updated_at = Set(Utc::now().timestamp());
                ws = active.update(&conn).await?;
            }
            ws
        }
        None => {
            let now = Utc::now().timestamp();
            #[allow(clippy::needless_update)]
            let model = workspaces::ActiveModel {
                id: Set(Uuid::now_v7().to_string()),
                name: Set(dir_name.clone()),
                created_by: Set(peer_id(&identity)?),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };
            model.insert(&conn).await?
        }
    };

    *db_state.workspace_db.write().await = Some(conn);

    let info = WorkspaceInfo::from_model(&workspace, &path);

    // Store in workspace state
    *ws_state.0.write().await = Some(info.clone());

    // Persist to global config
    {
        let mut config = config_state.0.write().await;
        if let Err(e) =
            crate::identity::config::update_last_workspace(&mut config, &path, &dir_name)
        {
            log::warn!("Failed to update global config: {e}");
        }
    }

    Ok(info)
}

#[tauri::command]
pub async fn get_workspace_info(
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<Option<WorkspaceInfo>> {
    Ok(ws_state.0.read().await.clone())
}

#[tauri::command]
pub async fn get_recent_workspaces(
    config_state: State<'_, GlobalConfigState>,
) -> AppResult<Vec<crate::identity::config::RecentWorkspace>> {
    let config = config_state.0.read().await;
    Ok(config.recent_workspaces.clone())
}
