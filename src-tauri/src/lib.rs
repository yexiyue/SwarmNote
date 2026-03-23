mod db;
pub mod error;
mod fs;
mod identity;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            identity::commands::get_device_info,
            identity::commands::set_device_name,
            db::commands::open_workspace,
            db::commands::get_workspace_info,
            db::commands::get_recent_workspaces,
            db::commands::db_get_documents,
            db::commands::db_upsert_document,
            db::commands::db_delete_document,
            db::commands::db_get_folders,
            db::commands::db_create_folder,
            db::commands::db_delete_folder,
            fs::commands::scan_workspace_tree,
            fs::commands::fs_create_file,
            fs::commands::fs_create_dir,
            fs::commands::fs_delete_file,
            fs::commands::fs_delete_dir,
            fs::commands::fs_rename,
        ])
        .setup(|app| {
            use tauri::Manager;
            // From<IdentityError> for AppError allows ? directly
            identity::init(app.handle())?;
            db::init(app.handle())?;
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
