mod config;
mod document;
pub mod error;
mod fs;
mod identity;
mod workspace;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // 设备身份
            identity::commands::get_device_info,
            identity::commands::set_device_name,
            // 工作区管理
            workspace::commands::open_workspace,
            workspace::commands::get_workspace_info,
            workspace::commands::get_recent_workspaces,
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
        ])
        .setup(|app| {
            use tauri::Manager;
            // From<IdentityError> for AppError 允许直接使用 ?
            identity::init(app.handle())?;
            workspace::init(app.handle())?;
            app.manage(fs::watcher::FsWatcherState::new());

            #[cfg(not(target_os = "macos"))]
            {
                let window = app.get_webview_window("main").unwrap();
                window.set_decorations(false)?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
