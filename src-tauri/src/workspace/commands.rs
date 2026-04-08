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

/// 应用平台相关的窗口装饰配置。
/// macOS: Overlay 标题栏 + 隐藏标题 + 红绿灯定位；其他平台: 无装饰（自定义标题栏）。
fn with_platform_decorations<'a, R: tauri::Runtime, M: tauri::Manager<R>>(
    builder: WebviewWindowBuilder<'a, R, M>,
) -> WebviewWindowBuilder<'a, R, M> {
    #[cfg(target_os = "macos")]
    let builder = {
        use tauri::TitleBarStyle;
        builder
            .decorations(true)
            .title_bar_style(TitleBarStyle::Overlay)
            .hidden_title(true)
            .traffic_light_position(tauri::LogicalPosition::new(15.0, 16.0))
    };

    #[cfg(not(target_os = "macos"))]
    let builder = builder.decorations(false);

    builder
}

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

/// 结果类型：告诉前端 `open_workspace_window` 采取了哪种路径。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum OpenWorkspaceWindowResult {
    /// 工作区已绑定到调用方窗口（fullscreen picker 场景）。
    BoundToCaller { info: WorkspaceInfo },
    /// 已聚焦一个已存在的窗口（路径已打开）。
    FocusedExisting,
    /// 创建了新窗口。
    NewWindow,
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

    // 刷新托盘菜单（最近工作区列表已更新）
    #[cfg(desktop)]
    {
        let app = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            crate::tray::refresh_tray_menu(&app).await;
        });
    }

    // Subscribe to workspace GossipSub topic and notify peers.
    if let Some(net_state) = app_handle.try_state::<crate::network::NetManagerState>() {
        if let Ok(sync_mgr) = net_state.sync().await {
            sync_mgr.subscribe_workspace(info.id).await;
            sync_mgr.publish_workspace_opened(info.id).await;
        }
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

/// 关闭指定 label 的窗口（如果存在），忽略不存在的 label。
/// Debug 模式下 main 窗口仅隐藏（保持 MCP bridge 通信可用），release 模式销毁。
fn maybe_close_window(app: &tauri::AppHandle, label: &Option<String>) {
    if let Some(close_label) = label {
        if let Some(win) = app.get_webview_window(close_label) {
            #[cfg(debug_assertions)]
            if close_label == "main" {
                let _ = win.hide();
                return;
            }
            let _ = win.destroy();
        }
    }
}

/// 为指定工作区路径打开新窗口，原子预绑定后端状态。
///
/// 当传入 `bind_to_window` 且该窗口尚未绑定任何工作区时，后端会把工作区
/// 绑定到调用方窗口，而不是创建新窗口。`close_window` 可选指定成功后要
/// 关闭的窗口（如 workspace-manager）。
#[expect(clippy::too_many_arguments)]
#[tauri::command]
pub async fn open_workspace_window(
    app: tauri::AppHandle,
    path: String,
    bind_to_window: Option<String>,
    close_window: Option<String>,
    db_state: State<'_, DbState>,
    identity: State<'_, IdentityState>,
    config_state: State<'_, GlobalConfigState>,
    ws_state: State<'_, WorkspaceState>,
    watcher_state: State<'_, crate::fs::watcher::FsWatcherState>,
) -> AppResult<OpenWorkspaceWindowResult> {
    tracing::info!("open_workspace_window called: path={path}, bind_to_window={bind_to_window:?}");
    let ws_path = PathBuf::from(&path);

    // 优先级 1：如果该路径已在某个窗口中打开，直接 focus 那个窗口
    if let Some(existing_label) = ws_state.find_label_by_path(&path).await {
        tracing::info!(
            "open_workspace_window: FocusedExisting via find_label_by_path, label={existing_label}"
        );
        if let Some(win) = app.get_webview_window(&existing_label) {
            let _ = win.set_focus();
        }
        maybe_close_window(&app, &close_window);
        return Ok(OpenWorkspaceWindowResult::FocusedExisting);
    }
    let hashed_label = workspace_window_label(&path);
    if let Some(existing) = app.get_webview_window(&hashed_label) {
        tracing::info!("open_workspace_window: FocusedExisting via hashed_label={hashed_label}");
        existing
            .set_focus()
            .map_err(|e| AppError::InvalidPath(format!("failed to focus window: {e}")))?;
        maybe_close_window(&app, &close_window);
        return Ok(OpenWorkspaceWindowResult::FocusedExisting);
    }

    if !ws_path.is_dir() {
        return Err(AppError::InvalidPath(format!(
            "Directory does not exist: {path}"
        )));
    }

    let (conn, workspace) = ensure_workspace(&ws_path, &identity).await?;

    // 优先级 2：如果 caller 请求绑定到自己，且自己尚未绑定任何工作区 → 绑定到 caller
    if let Some(caller_label) = bind_to_window.as_deref() {
        let caller_has_ws = ws_state.get_by_label(caller_label).await.is_some();
        let caller_window = app.get_webview_window(caller_label);
        if !caller_has_ws && caller_window.is_some() {
            let info = bind_workspace_to_window(
                caller_label,
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

            if let Some(win) = caller_window {
                if let Err(e) = win.emit("workspace:ready", &info) {
                    log::warn!(
                        "Failed to emit workspace:ready to caller window '{caller_label}': {e}"
                    );
                }
                let _ = win.set_focus();
            }

            maybe_close_window(&app, &close_window);
            return Ok(OpenWorkspaceWindowResult::BoundToCaller { info });
        }
    }

    // 优先级 3：创建新窗口。使用 path 的 hash 作为稳定 label。
    let label = hashed_label;
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

    let new_window = with_platform_decorations(
        WebviewWindowBuilder::new(&app, &label, WebviewUrl::App("index.html".into()))
            .title("SwarmNote")
            .inner_size(800.0, 600.0)
            .min_inner_size(800.0, 600.0),
    )
    .build()
    .map_err(|e| AppError::InvalidPath(format!("failed to create window: {e}")))?;

    if let Err(e) = new_window.emit("workspace:ready", &info) {
        log::warn!("Failed to emit workspace:ready to window '{label}': {e}");
    }

    bind_window_cleanup(&new_window, &app, &label);
    maybe_close_window(&app, &close_window);

    Ok(OpenWorkspaceWindowResult::NewWindow)
}

/// 根据工作区路径生成稳定的窗口 label。
fn workspace_window_label(path: &str) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("ws-{:016x}", hasher.finish())
}

/// 为同步创建一个新工作区（不打开窗口），使用指定的 UUID。
///
/// 用于接收方在接收到远程工作区邀请时，在本地创建并注册对应工作区。
/// 与 `open_workspace` 的区别：
/// - 写入指定的 UUID（而非生成新的）
/// - 不创建窗口，使用 `sync-{uuid}` 作为临时 label
/// - 失败时清理已创建的目录
#[tauri::command]
pub async fn create_workspace_for_sync(
    uuid: String,
    name: String,
    base_path: String,
    db_state: State<'_, DbState>,
    ws_state: State<'_, WorkspaceState>,
    config_state: State<'_, GlobalConfigState>,
    identity: State<'_, IdentityState>,
) -> AppResult<String> {
    let ws_uuid =
        Uuid::parse_str(&uuid).map_err(|e| AppError::Config(format!("Invalid UUID: {e}")))?;

    // Validate workspace name — prevent path traversal
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name == "."
    {
        return Err(AppError::InvalidPath(format!(
            "Invalid workspace name: {name}"
        )));
    }

    let base = PathBuf::from(&base_path);
    if !base.is_dir() {
        tokio::fs::create_dir_all(&base).await.map_err(|e| {
            AppError::InvalidPath(format!("Failed to create base directory {base_path}: {e}"))
        })?;
    }
    let ws_path = base.join(&name);
    let ws_path_str = ws_path
        .to_str()
        .ok_or_else(|| AppError::InvalidPath("Workspace path is not valid UTF-8".to_owned()))?
        .to_owned();

    // Create the workspace directory
    tokio::fs::create_dir_all(&ws_path)
        .await
        .map_err(AppError::Io)?;

    // Write identity with the specified UUID
    let identity_data = super::identity::WorkspaceIdentity {
        uuid: ws_uuid,
        name: name.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let write_result = super::identity::write_identity(&ws_path, &identity_data).await;
    if let Err(e) = write_result {
        // Clean up on failure
        let _ = tokio::fs::remove_dir_all(&ws_path).await;
        return Err(e);
    }

    // Initialize workspace DB and run migrations
    let conn = match init_workspace_db(&ws_path).await {
        Ok(c) => c,
        Err(e) => {
            let _ = tokio::fs::remove_dir_all(&ws_path).await;
            return Err(e);
        }
    };

    // Insert workspace record into DB
    let peer_id = match identity.peer_id() {
        Ok(p) => p,
        Err(e) => {
            let _ = tokio::fs::remove_dir_all(&ws_path).await;
            return Err(e);
        }
    };
    let workspace_result = {
        use entity::workspace::workspaces;
        use sea_orm::EntityTrait;

        let found = match workspaces::Entity::find().one(&conn).await {
            Ok(v) => v,
            Err(e) => {
                let _ = tokio::fs::remove_dir_all(&ws_path).await;
                return Err(e.into());
            }
        };
        match found {
            Some(ws) => ws,
            None => {
                let model = workspaces::ActiveModel {
                    id: Set(ws_uuid),
                    name: Set(name.clone()),
                    created_by: Set(peer_id),
                    ..Default::default()
                };
                match model.insert(&conn).await {
                    Ok(ws) => ws,
                    Err(e) => {
                        let _ = tokio::fs::remove_dir_all(&ws_path).await;
                        return Err(e.into());
                    }
                }
            }
        }
    };

    // Register in DbState and WorkspaceState without a window label.
    // Sync-only: sync layer accesses by UUID, no window exists.
    db_state.register_db(ws_uuid, conn).await;

    let info = WorkspaceInfo::from_model(&workspace_result, &ws_path_str);
    ws_state.register(info).await;

    // Update recent workspaces in global config
    {
        let mut config = config_state.write().await;
        if let Err(e) = crate::config::update_last_workspace_with_uuid(
            &mut config,
            &ws_path_str,
            &name,
            &ws_uuid.to_string(),
        ) {
            tracing::warn!("Failed to update global config for sync workspace: {e}");
        }
    }

    tracing::info!("Created workspace for sync: {name} ({ws_uuid}) at {ws_path_str}");
    Ok(ws_path_str)
}

/// 创建 onboarding 引导窗口（仅在 setup 阶段调用）。
pub fn create_onboarding_window(app: &tauri::AppHandle) -> Result<(), AppError> {
    with_platform_decorations(
        WebviewWindowBuilder::new(app, "onboarding", WebviewUrl::App("/onboarding".into()))
            .title("SwarmNote")
            .inner_size(600.0, 500.0)
            .min_inner_size(600.0, 500.0)
            .maximizable(false),
    )
    .build()
    .map_err(|e| AppError::Window(format!("Failed to create onboarding window: {e}")))?;

    Ok(())
}

/// 完成 onboarding：原子化地创建工作区管理窗口并关闭 onboarding 窗口。
/// 前端应先调用 `onboardingStore.complete()` 持久化状态，再调用此命令处理窗口切换。
#[tauri::command]
pub async fn finish_onboarding(app: tauri::AppHandle) -> AppResult<()> {
    // 先创建管理窗口，确保用户看得到新窗口后再关闭旧窗口
    open_workspace_manager_window(app.clone()).await?;

    // 关闭 onboarding 窗口
    if let Some(win) = app.get_webview_window("onboarding") {
        let _ = win.destroy();
    }

    Ok(())
}

/// 从最近工作区列表中移除指定路径（幂等）。
#[tauri::command]
pub async fn remove_recent_workspace(
    app: tauri::AppHandle,
    path: String,
    config_state: State<'_, GlobalConfigState>,
) -> AppResult<()> {
    let mut config = config_state.write().await;
    config.recent_workspaces.retain(|w| w.path != path);
    // 如果被移除的是 last_workspace_path，也清掉
    if config.last_workspace_path.as_deref() == Some(&path) {
        config.last_workspace_path = config.recent_workspaces.first().map(|w| w.path.clone());
    }
    crate::config::save_config(&config)?;
    drop(config);

    // 刷新托盘菜单
    #[cfg(desktop)]
    crate::tray::refresh_tray_menu(&app).await;

    Ok(())
}

/// 打开或聚焦工作区管理窗口（label: "main"，兼容 MCP bridge 等基础设施）。
#[tauri::command]
pub async fn open_workspace_manager_window(app: tauri::AppHandle) -> AppResult<()> {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }

    with_platform_decorations(
        WebviewWindowBuilder::new(&app, "main", WebviewUrl::App("/workspace-manager".into()))
            .title("SwarmNote")
            .inner_size(780.0, 620.0)
            .min_inner_size(780.0, 620.0)
            .maximizable(false),
    )
    .build()
    .map_err(|e| AppError::Window(format!("Failed to create workspace manager window: {e}")))?;

    Ok(())
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

    let _settings_window = with_platform_decorations(
        WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App(target_route.into()))
            .title("SwarmNote 设置")
            .inner_size(720.0, 520.0)
            .resizable(false),
    )
    .build()
    .map_err(|e| AppError::Window(format!("Failed to create settings window: {e}")))?;

    Ok(())
}
