use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use entity::workspace::workspaces;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use serde::Serialize;
use tauri::{Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

use super::db::init_workspace_db;
use super::state::{DbState, WorkspaceState};
use crate::config::GlobalConfigState;
use crate::error::{AppError, AppResult};
use crate::identity::IdentityState;

/// 返回给前端的工作区信息，组合数据库记录 + 运行时路径。
#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceInfo {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WorkspaceInfo {
    pub fn from_model(model: &workspaces::Model, path: &str) -> Self {
        Self {
            id: model.id,
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
///
/// Ensures `workspace.json` exists (source of truth for workspace UUID)
/// and that the `workspaces` table uses the same UUID.
async fn ensure_workspace(
    ws_path: &Path,
    identity: &IdentityState,
) -> AppResult<(DatabaseConnection, workspaces::Model)> {
    let conn = init_workspace_db(ws_path).await?;
    let dir_name = workspace_name_from_path(ws_path);

    let existing_ws = workspaces::Entity::find().one(&conn).await?;

    // Resolve workspace UUID from workspace.json (source of truth).
    // Falls back to existing DB UUID for upgrades, or generates new.
    let ws_uuid =
        super::identity::ensure_identity(ws_path, existing_ws.as_ref().map(|ws| ws.id), &dir_name)
            .await?;

    let workspace = match existing_ws {
        Some(mut ws) => {
            let needs_update = ws.name != dir_name || ws.id != ws_uuid;
            if needs_update {
                let mut active: workspaces::ActiveModel = ws.clone().into();
                active.id = Set(ws_uuid);
                active.name = Set(dir_name);
                ws = active.update(&conn).await?;
            }
            ws
        }
        None => {
            let model = workspaces::ActiveModel {
                id: Set(ws_uuid),
                name: Set(dir_name),
                created_by: Set(identity.peer_id()?),
                ..Default::default()
            };
            model.insert(&conn).await?
        }
    };

    Ok((conn, workspace))
}

/// 将工作区绑定到指定窗口的 per-window 状态，并更新全局配置和 watcher。
#[expect(clippy::too_many_arguments)]
async fn bind_workspace_to_window(
    label: &str,
    path: &str,
    conn: DatabaseConnection,
    workspace: &workspaces::Model,
    identity: &IdentityState,
    db_state: &DbState,
    ws_state: &WorkspaceState,
    config_state: &GlobalConfigState,
    watcher_state: &crate::fs::watcher::FsWatcherState,
    app_handle: &tauri::AppHandle,
) -> WorkspaceInfo {
    // Reconcile scanned files with DB before starting the watcher,
    // so every .md file has a stable UUID from the start.
    let ws_path = Path::new(path);
    if let Ok(tree) = crate::fs::scan::scan_workspace_tree(ws_path) {
        if let Ok(peer_id) = identity.peer_id() {
            if let Err(e) =
                crate::fs::scan::reconcile_with_db(&conn, workspace.id, &peer_id, &tree).await
            {
                log::warn!("Reconcile failed for '{path}': {e}");
            }
        }
    }

    db_state
        .insert_workspace_db(label, workspace.id, conn)
        .await;

    let info = WorkspaceInfo::from_model(workspace, path);
    ws_state.bind(label, info.clone()).await;

    let dir_name = workspace_name_from_path(ws_path);
    {
        let mut config = config_state.write().await;
        if let Err(e) = crate::config::update_last_workspace(&mut config, path, &dir_name) {
            log::warn!("Failed to update global config: {e}");
        }
    }

    if let Err(e) = crate::fs::watcher::start_watching(app_handle, label, ws_path, watcher_state) {
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
                super::cleanup_window(&a, &lbl, &db, &ws, &watcher).await;
            });
        }
    });
}

// ── Tauri commands ──

/// 幂等的工作区打开/创建命令（per-window）。
#[expect(clippy::too_many_arguments)]
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
        &identity,
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
    Ok(ws_state.get_by_label(window.label()).await)
}

#[tauri::command]
pub async fn get_recent_workspaces(
    config_state: State<'_, GlobalConfigState>,
) -> AppResult<Vec<crate::config::RecentWorkspace>> {
    let config = config_state.read().await;
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
    if let Some(existing_label) = ws_state.find_label_by_path(&path).await {
        if let Some(win) = app.get_webview_window(&existing_label) {
            let _ = win.set_focus();
        }
        return Ok(());
    }

    if !ws_path.is_dir() {
        return Err(AppError::InvalidPath(format!(
            "Directory does not exist: {path}"
        )));
    }

    // 先完成 DB 初始化和状态绑定，再创建窗口，消除竞态
    let (conn, workspace) = ensure_workspace(&ws_path, &identity).await?;
    let info = bind_workspace_to_window(
        &label,
        &path,
        conn,
        &workspace,
        &identity,
        &db_state,
        &ws_state,
        &config_state,
        &watcher_state,
        &app,
    )
    .await;

    // 状态就绪后再创建窗口
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

    // 推送工作区信息给新窗口，前端可通过事件直接获取而无需轮询
    if let Err(e) = new_window.emit("workspace:ready", &info) {
        log::warn!("Failed to emit workspace:ready to window '{label}': {e}");
    }

    bind_window_cleanup(&new_window, &app, &label);

    Ok(())
}

/// 根据工作区路径生成稳定的窗口 label。
fn workspace_window_label(path: &str) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("ws-{:016x}", hasher.finish())
}

/// 打开或聚焦设置窗口，支持路由导航。
#[tauri::command]
pub async fn open_settings_window(app: tauri::AppHandle, route: Option<String>) -> AppResult<()> {
    let target_route = format!(
        "/settings/{}",
        route.unwrap_or_else(|| "general".to_string())
    );

    if let Some(win) = app.get_webview_window("settings") {
        let _ = win.set_focus();
        let _ = win.emit("navigate", &target_route);
        return Ok(());
    }

    let _settings_window =
        WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App(target_route.into()))
            .title("SwarmNote 设置")
            .inner_size(720.0, 520.0)
            .resizable(false)
            .decorations(cfg!(target_os = "macos"))
            .build()
            .map_err(|e| AppError::Window(format!("Failed to create settings window: {e}")))?;

    #[cfg(target_os = "macos")]
    {
        use tauri::TitleBarStyle;
        let _ = _settings_window.set_title_bar_style(TitleBarStyle::Overlay);
    }

    Ok(())
}
