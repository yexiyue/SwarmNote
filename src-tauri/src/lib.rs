mod config;
mod device;
mod document;
pub mod error;
mod fs;
mod identity;
mod network;
mod pairing;
mod protocol;
#[cfg(desktop)]
pub mod tray;
mod workspace;

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
        .plugin(tauri_plugin_notification::init());

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
            document::commands::db_delete_document,
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
            // P2P 网络
            network::commands::start_p2p_node,
            network::commands::stop_p2p_node,
            network::commands::get_connected_peers,
            // 配对管理
            pairing::commands::generate_pairing_code,
            pairing::commands::get_device_by_code,
            pairing::commands::request_pairing,
            pairing::commands::respond_pairing_request,
            pairing::commands::get_paired_devices,
            pairing::commands::unpair_device,
            pairing::commands::get_nearby_devices,
            workspace::commands::open_settings_window,
        ])
        .on_window_event(|window, event| {
            #[cfg(desktop)]
            {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let label = window.label();
                    let is_workspace = label == "main" || label.starts_with("ws-");

                    if is_workspace {
                        // 统计当前可见的工作区窗口数量
                        use tauri::Manager;
                        let visible_ws_count = window
                            .app_handle()
                            .webview_windows()
                            .iter()
                            .filter(|(l, w)| {
                                (*l == "main" || l.starts_with("ws-"))
                                    && w.is_visible().unwrap_or(false)
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
                    // 非工作区窗口（settings 等）：正常关闭销毁
                }
            }
        })
        .setup(|app| {
            use tauri::Manager;

            identity::init(app.handle())?;
            app.manage(fs::watcher::FsWatcherState::new());
            workspace::init(app.handle())?;

            // 如果主窗口自动恢复了工作区，启动对应的 fs watcher
            {
                let ws_state = app.state::<workspace::state::WorkspaceState>();
                let watcher_state = app.state::<fs::watcher::FsWatcherState>();
                if let Some(info) = tauri::async_runtime::block_on(ws_state.get("main")) {
                    let ws_path = std::path::PathBuf::from(&info.path);
                    if let Err(e) =
                        fs::watcher::start_watching(app.handle(), "main", &ws_path, &watcher_state)
                    {
                        log::warn!("Failed to start fs watcher for auto-restored workspace: {e}");
                    }
                }
            }

            // P2P 网络状态（初始为 None，由前端根据偏好触发启动）
            let net_state: network::NetManagerState = tokio::sync::Mutex::new(None);
            app.manage(net_state);

            // 创建系统托盘（仅桌面端）
            #[cfg(desktop)]
            {
                tray::TrayManager::init(app.handle())?;
            }

            #[cfg(not(target_os = "macos"))]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.set_decorations(false)?;
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
