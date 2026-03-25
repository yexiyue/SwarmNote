use std::path::{Path, PathBuf};

use chrono::Utc;
use entity::workspace::workspaces;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use super::db::init_workspace_db;
use super::state::{DbState, WorkspaceState};
use crate::config::GlobalConfigState;
use crate::error::{AppError, AppResult};
use crate::identity::IdentityState;

fn peer_id(identity: &IdentityState) -> AppResult<String> {
    let info = identity
        .device_info
        .read()
        .map_err(|e| AppError::Identity(format!("lock error: {e}")))?;
    Ok(info.peer_id.clone())
}

/// 返回给前端的工作区信息，组合数据库记录 + 运行时路径。
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

/// 从路径的最后一个组件提取工作区名称。
pub(crate) fn workspace_name_from_path(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Workspace".to_owned())
}

/// 幂等的工作区打开/创建命令。
///
/// - 若 `.swarmnote/workspace.db` 已存在：打开并返回现有信息。
/// - 否则：创建 `.swarmnote/`，初始化数据库，插入工作区记录。
/// - 成功后更新全局配置的 `last_workspace_path`。
#[tauri::command]
pub async fn open_workspace(
    path: String,
    db_state: State<'_, DbState>,
    identity: State<'_, IdentityState>,
    config_state: State<'_, GlobalConfigState>,
    ws_state: State<'_, WorkspaceState>,
    watcher_state: State<'_, crate::fs::watcher::FsWatcherState>,
    app_handle: tauri::AppHandle,
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
            // 如果目录被重命名则更新名称
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

    // 存储到工作区状态
    *ws_state.0.write().await = Some(info.clone());

    // 持久化到全局配置
    {
        let mut config = config_state.0.write().await;
        if let Err(e) = crate::config::update_last_workspace(&mut config, &path, &dir_name) {
            log::warn!("Failed to update global config: {e}");
        }
    }

    // 为新工作区启动文件系统监听器
    if let Err(e) = crate::fs::watcher::start_watching(&app_handle, &ws_path, &watcher_state) {
        log::warn!("Failed to start fs watcher: {e}");
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
) -> AppResult<Vec<crate::config::RecentWorkspace>> {
    let config = config_state.0.read().await;
    Ok(config.recent_workspaces.clone())
}
