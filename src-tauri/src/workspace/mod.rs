//! 工作区管理：数据库初始化、自动恢复、per-window 资源生命周期。

pub mod commands;
pub mod db;
pub mod identity;
pub mod state;

use std::path::PathBuf;

use tauri::Manager;

use crate::config::GlobalConfigState;
use crate::error::AppError;
use state::{DbState, WorkspaceState};

/// 启动时应创建的窗口类型。
pub enum StartupWindow {
    /// 首次启动，显示 onboarding 引导窗口
    Onboarding,
    /// 显示工作区管理窗口（无历史或用户选择不自动恢复）
    WorkspaceManager,
    /// 自动恢复上次工作区
    RestoreWorkspace(String),
}

/// 初始化数据库层：devices.db。不再自动恢复工作区到固定窗口。
/// 启动时的窗口创建由 `determine_startup_window` + setup 统一处理。
pub fn init(app: &tauri::AppHandle) -> Result<(), AppError> {
    let devices_db = tauri::async_runtime::block_on(db::init_devices_db())?;

    let db_state = DbState::new(devices_db);
    let ws_state = WorkspaceState::new();

    app.manage(db_state);
    app.manage(ws_state);

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

    // ── swarmnote-core WorkspaceCore teardown (PR #2) ──
    // Unbind the label from the WorkspaceMap; if this was the last window
    // holding the workspace, authoritatively close it so the core flushes
    // YDocManager + unwatches the directory.
    if let Some(ws_map) = app.try_state::<crate::platform::WorkspaceMap>() {
        if let Some((workspace_uuid, last)) = ws_map.unbind(label).await {
            if last {
                if let Some(app_core) = app.try_state::<std::sync::Arc<swarmnote_core::AppCore>>() {
                    if let Err(e) = app_core.close_workspace(workspace_uuid).await {
                        tracing::warn!("AppCore::close_workspace({workspace_uuid}) failed: {e}");
                    }
                }
            }
        }
    }

    crate::fs::watcher::stop_watching(label, watcher_state);

    // Unsubscribe from workspace GossipSub topic before unbinding (need UUID)
    if let Some(ws_info) = ws_state.get_by_label(label).await {
        if let Some(net_state) = app.try_state::<crate::network::NetManagerState>() {
            if let Ok(sync_mgr) = net_state.sync().await {
                sync_mgr.unsubscribe_workspace(ws_info.id).await;
            }
        }
    }

    if ws_state.unbind_by_label(label).await {
        tracing::info!("Cleaned up WorkspaceInfo for window '{label}'");
    }
    if db_state.remove_workspace_db(label).await {
        tracing::info!("Cleaned up workspace DB for window '{label}'");
    }
}

/// 从前端 Tauri plugin-store 的 settings.json 中提取 Zustand persist 状态中的布尔字段。
/// 格式: `{ "store-key": "{\"state\":{\"field\":true},\"version\":0}" }`
fn read_store_bool(store: &serde_json::Value, store_key: &str, field: &str) -> Option<bool> {
    let inner_str = store.get(store_key)?.as_str()?;
    let inner: serde_json::Value = serde_json::from_str(inner_str).ok()?;
    inner.get("state")?.get(field)?.as_bool()
}

/// 根据 onboarding 完成状态和恢复偏好决定启动时创建哪种窗口。
pub fn determine_startup_window(app: &tauri::AppHandle) -> StartupWindow {
    // 一次性读取 settings.json，避免重复 I/O
    let store = (|| -> Option<serde_json::Value> {
        let app_data = app.path().app_data_dir().ok()?;
        let content = std::fs::read_to_string(app_data.join("settings.json")).ok()?;
        serde_json::from_str(&content).ok()
    })()
    .unwrap_or_default();

    let onboarding_completed =
        read_store_bool(&store, "swarmnote-onboarding", "isCompleted").unwrap_or(false);

    if !onboarding_completed {
        return StartupWindow::Onboarding;
    }

    let restore_last =
        read_store_bool(&store, "swarmnote-preferences", "restoreLastWorkspace").unwrap_or(true);

    if restore_last {
        if let Some(config_state) = app.try_state::<GlobalConfigState>() {
            let last_path = tauri::async_runtime::block_on(async {
                config_state.read().await.last_workspace_path.clone()
            });
            if let Some(path) = last_path {
                if !path.is_empty() && PathBuf::from(&path).is_dir() {
                    return StartupWindow::RestoreWorkspace(path);
                }
            }
        }
    }

    StartupWindow::WorkspaceManager
}
