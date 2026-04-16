mod config;
mod device;
mod document;
pub mod error;
mod fs;
mod identity;
mod network;
mod pairing;
mod platform;
mod protocol;
mod sync;
#[cfg(desktop)]
pub mod tray;
mod workspace;
mod yjs;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "swarmnote_lib=info,swarm_p2p_core=info".into()),
        )
        .init();

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init());

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .invoke_handler(tauri::generate_handler![
            // 设备身份
            identity::commands::get_device_info,
            identity::commands::set_device_name,
            // 工作区管理
            workspace::commands::open_workspace,
            workspace::commands::get_workspace_info,
            workspace::commands::get_recent_workspaces,
            workspace::commands::open_workspace_window,
            // 文档 & 文件夹
            document::commands::db_get_documents,
            document::commands::db_upsert_document,
            document::commands::delete_document_by_rel_path,
            document::commands::delete_documents_by_prefix,
            document::commands::rename_document,
            document::commands::move_document,
            document::commands::db_get_folders,
            document::commands::db_create_folder,
            document::commands::db_delete_folder,
            // 文件系统
            fs::commands::scan_workspace_tree,
            fs::commands::fs_create_file,
            fs::commands::fs_create_dir,
            fs::commands::fs_delete_file,
            fs::commands::fs_delete_dir,
            fs::commands::fs_rename,
            fs::commands::load_document,
            fs::commands::save_document,
            fs::commands::save_media,
            // P2P 网络
            network::commands::start_p2p_node,
            network::commands::stop_p2p_node,
            network::commands::get_network_status,
            network::commands::get_connected_peers,
            // 配对管理
            pairing::commands::generate_pairing_code,
            pairing::commands::get_device_by_code,
            pairing::commands::request_pairing,
            pairing::commands::respond_pairing_request,
            pairing::commands::get_paired_devices,
            pairing::commands::unpair_device,
            pairing::commands::get_nearby_devices,
            pairing::commands::list_devices,
            pairing::commands::get_remote_workspaces,
            workspace::commands::finish_onboarding,
            workspace::commands::remove_recent_workspace,
            workspace::commands::open_workspace_manager_window,
            workspace::commands::open_settings_window,
            // Y.Doc 管理
            yjs::commands::open_ydoc,
            yjs::commands::apply_ydoc_update,
            yjs::commands::close_ydoc,
            yjs::commands::rename_ydoc,
            yjs::commands::reload_ydoc_confirmed,
            yjs::commands::hydrate_workspace,
            // 同步
            sync::commands::trigger_workspace_sync,
            workspace::commands::create_workspace_for_sync,
        ])
        .on_window_event(|window, event| {
            #[cfg(desktop)]
            {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let label = window.label();
                    let is_workspace = label.starts_with("ws-");

                    if is_workspace {
                        // 统计当前可见的工作区窗口数量
                        use tauri::Manager;
                        let visible_ws_count = window
                            .app_handle()
                            .webview_windows()
                            .iter()
                            .filter(|(l, w)| {
                                l.starts_with("ws-") && w.is_visible().unwrap_or(false)
                            })
                            .count();

                        if visible_ws_count <= 1 {
                            // 最后一个工作区窗口：隐藏到托盘，记录 label
                            api.prevent_close();
                            let _ = window.hide();
                            if let Some(state) =
                                window.app_handle().try_state::<tray::TrayManagerState>()
                            {
                                if let Ok(mut mgr) = state.try_lock() {
                                    mgr.set_last_hidden(label);
                                }
                            }
                        }
                        // 否则：还有其他工作区窗口，正常关闭
                    }
                    // 非工作区窗口（settings, workspace-manager, onboarding）：正常关闭销毁
                }
            }
        })
        .setup(|app| {
            use std::sync::Arc;
            use tauri::Manager;

            // ── swarmnote-core AppCore (PR #1) ──
            // Bootstrap the platform-independent core. Other (not-yet-ported)
            // modules continue to use their own Tauri State for now — they
            // coexist peacefully until PR #2/#3 migrates them over.
            let app_data_dir = config::swarmnote_global_dir()?;
            let keychain = Arc::new(platform::DesktopKeychain::new());
            let event_bus = Arc::new(platform::TauriEventBus::new(app.handle().clone()));
            let app_core = tauri::async_runtime::block_on(swarmnote_core::AppCore::new(
                keychain,
                event_bus,
                app_data_dir,
            ))?;
            app.manage(app_core);

            // ── legacy per-module init (kept until PR #2/#3 ports them) ──
            identity::init(app.handle())?;
            app.manage(fs::watcher::FsWatcherState::new());
            workspace::init(app.handle())?;

            // Y.Doc 管理器
            app.manage(yjs::manager::YDocManager::new());

            // P2P 网络状态（初始为 None，由前端根据偏好触发启动）
            app.manage(network::NetManagerState::new());

            // 创建系统托盘（仅桌面端）
            #[cfg(desktop)]
            {
                tray::TrayManager::init(app.handle())?;
            }

            // 根据 onboarding 状态和恢复偏好决定创建哪种窗口
            match workspace::determine_startup_window(app.handle()) {
                workspace::StartupWindow::Onboarding => {
                    workspace::commands::create_onboarding_window(app.handle())?;
                }
                workspace::StartupWindow::WorkspaceManager => {
                    tauri::async_runtime::block_on(async {
                        if let Err(e) =
                            workspace::commands::open_workspace_manager_window(app.handle().clone())
                                .await
                        {
                            log::error!("Failed to create workspace manager window: {e}");
                        }
                    });
                }
                workspace::StartupWindow::RestoreWorkspace(path) => {
                    // 创建工作区窗口并绑定。复用 open_workspace_window 的逻辑。
                    let handle = app.handle().clone();
                    tauri::async_runtime::block_on(async {
                        let db_state = handle.state::<workspace::state::DbState>();
                        let identity = handle.state::<identity::IdentityState>();
                        let config_state = handle.state::<config::GlobalConfigState>();
                        let ws_state = handle.state::<workspace::state::WorkspaceState>();
                        let watcher_state = handle.state::<fs::watcher::FsWatcherState>();
                        if let Err(e) = workspace::commands::open_workspace_window(
                            handle.clone(),
                            path,
                            None,
                            None,
                            db_state,
                            identity,
                            config_state,
                            ws_state,
                            watcher_state,
                        )
                        .await
                        {
                            log::error!("Failed to restore workspace: {e}");
                            // Fallback to workspace manager
                            if let Err(e2) =
                                workspace::commands::open_workspace_manager_window(handle).await
                            {
                                log::error!("Failed to create workspace manager window: {e2}");
                            }
                        }
                    });
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
