mod db;
pub mod error;
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
        ])
        .setup(|app| {
            // From<IdentityError> for AppError allows ? directly
            identity::init(app.handle())?;
            db::init(app.handle())?;

            #[cfg(not(target_os = "macos"))]
            {
                use tauri::Manager;
                let window = app.get_webview_window("main").unwrap();
                window.set_decorations(false)?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
