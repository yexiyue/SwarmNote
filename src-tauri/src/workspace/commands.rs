use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use chrono::Utc;
use entity::workspace::workspaces;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde::Serialize;
use tauri::{Manager, State, WebviewUrl, WebviewWindowBuilder};
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

// ── 内部 helpers ──

/// 幂等地打开或创建工作区数据库记录，返回 DB 连接和工作区 model。
async fn ensure_workspace(
    ws_path: &Path,
    identity: &IdentityState,
) -> AppResult<(DatabaseConnection, workspaces::Model)> {
    let conn = init_workspace_db(ws_path).await?;
    let dir_name = workspace_name_from_path(ws_path);

    let workspace = match workspaces::Entity::find().one(&conn).await? {
        Some(mut ws) => {
            if ws.name != dir_name {
                let mut active: workspaces::ActiveModel = ws.clone().into();
                active.name = Set(dir_name);
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
                name: Set(dir_name),
                created_by: Set(peer_id(identity)?),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };
            model.insert(&conn).await?
        }
    };

    Ok((conn, workspace))
}

/// 将工作区绑定到指定窗口的 per-window 状态，并更新全局配置和 watcher。
#[allow(clippy::too_many_arguments)]
async fn bind_workspace_to_window(
    label: &str,
    path: &str,
    conn: DatabaseConnection,
    workspace: &workspaces::Model,
    db_state: &DbState,
    ws_state: &WorkspaceState,
    config_state: &GlobalConfigState,
    watcher_state: &crate::fs::watcher::FsWatcherState,
    app_handle: &tauri::AppHandle,
) -> WorkspaceInfo {
    db_state
        .workspace_dbs
        .write()
        .await
        .insert(label.to_owned(), conn);

    let info = WorkspaceInfo::from_model(workspace, path);
    ws_state
        .0
        .write()
        .await
        .insert(label.to_owned(), info.clone());

    let dir_name = workspace_name_from_path(Path::new(path));
    {
        let mut config = config_state.0.write().await;
        if let Err(e) = crate::config::update_last_workspace(&mut config, path, &dir_name) {
            log::warn!("Failed to update global config: {e}");
        }
    }

    if let Err(e) =
        crate::fs::watcher::start_watching(app_handle, label, Path::new(path), watcher_state)
    {
        log::warn!("Failed to start fs watcher for window '{label}': {e}");
    }

    info
}

/// 为窗口绑定关闭时的资源清理监听。
fn bind_window_cleanup(window: &tauri::WebviewWindow, app: &tauri::AppHandle, label: &str) {
    let cleanup_label = label.to_owned();
    let cleanup_app = app.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Destroyed = event {
            let lbl = cleanup_label.clone();
            let a = cleanup_app.clone();
            tauri::async_runtime::spawn(async move {
                let db = a.state::<DbState>();
                let ws = a.state::<WorkspaceState>();
                let watcher = a.state::<crate::fs::watcher::FsWatcherState>();
                super::cleanup_window(&lbl, &db, &ws, &watcher).await;
            });
        }
    });
}

// ── Tauri commands ──

/// 幂等的工作区打开/创建命令（per-window）。
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn open_workspace(
    window: tauri::Window,
    path: String,
    db_state: State<'_, DbState>,
    identity: State<'_, IdentityState>,
    config_state: State<'_, GlobalConfigState>,
    ws_state: State<'_, WorkspaceState>,
    watcher_state: State<'_, crate::fs::watcher::FsWatcherState>,
    app_handle: tauri::AppHandle,
) -> AppResult<WorkspaceInfo> {
    let label = window.label().to_owned();
    let ws_path = PathBuf::from(&path);

    if !ws_path.is_dir() {
        return Err(AppError::InvalidPath(format!(
            "Directory does not exist: {path}"
        )));
    }

    let (conn, workspace) = ensure_workspace(&ws_path, &identity).await?;

    let info = bind_workspace_to_window(
        &label,
        &path,
        conn,
        &workspace,
        &db_state,
        &ws_state,
        &config_state,
        &watcher_state,
        &app_handle,
    )
    .await;

    Ok(info)
}

/// 返回当前窗口对应的工作区信息。
#[tauri::command]
pub async fn get_workspace_info(
    window: tauri::Window,
    ws_state: State<'_, WorkspaceState>,
) -> AppResult<Option<WorkspaceInfo>> {
    let infos = ws_state.0.read().await;
    Ok(infos.get(window.label()).cloned())
}

#[tauri::command]
pub async fn get_recent_workspaces(
    config_state: State<'_, GlobalConfigState>,
) -> AppResult<Vec<crate::config::RecentWorkspace>> {
    let config = config_state.0.read().await;
    Ok(config.recent_workspaces.clone())
}

/// 为指定工作区路径打开新窗口，原子预绑定后端状态。
#[tauri::command]
pub async fn open_workspace_window(
    app: tauri::AppHandle,
    path: String,
    db_state: State<'_, DbState>,
    identity: State<'_, IdentityState>,
    config_state: State<'_, GlobalConfigState>,
    ws_state: State<'_, WorkspaceState>,
    watcher_state: State<'_, crate::fs::watcher::FsWatcherState>,
) -> AppResult<()> {
    let label = workspace_window_label(&path);
    let ws_path = PathBuf::from(&path);

    // 检查是否已有窗口打开了该路径（通过窗口 label）
    if let Some(existing) = app.get_webview_window(&label) {
        existing
            .set_focus()
            .map_err(|e| AppError::InvalidPath(format!("failed to focus window: {e}")))?;
        return Ok(());
    }

    // 检查 per-window 状态中是否已有该路径
    {
        let infos = ws_state.0.read().await;
        for (existing_label, info) in infos.iter() {
            if info.path == path {
                if let Some(win) = app.get_webview_window(existing_label) {
                    let _ = win.set_focus();
                }
                return Ok(());
            }
        }
    }

    if !ws_path.is_dir() {
        return Err(AppError::InvalidPath(format!(
            "Directory does not exist: {path}"
        )));
    }

    // 创建新窗口
    let new_window = WebviewWindowBuilder::new(&app, &label, WebviewUrl::App("index.html".into()))
        .title("SwarmNote")
        .inner_size(800.0, 600.0)
        .min_inner_size(800.0, 600.0)
        .decorations(cfg!(target_os = "macos"))
        .build()
        .map_err(|e| AppError::InvalidPath(format!("failed to create window: {e}")))?;

    #[cfg(target_os = "macos")]
    {
        use tauri::TitleBarStyle;
        let _ = new_window.set_title_bar_style(TitleBarStyle::Overlay);
    }

    // 原子预绑定
    let (conn, workspace) = ensure_workspace(&ws_path, &identity).await?;
    bind_workspace_to_window(
        &label,
        &path,
        conn,
        &workspace,
        &db_state,
        &ws_state,
        &config_state,
        &watcher_state,
        &app,
    )
    .await;

    bind_window_cleanup(&new_window, &app, &label);

    Ok(())
}

/// 根据工作区路径生成稳定的窗口 label。
fn workspace_window_label(path: &str) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("ws-{:016x}", hasher.finish())
}
