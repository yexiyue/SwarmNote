mod db;
pub mod error;
mod identity;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            identity::commands::get_device_info,
            identity::commands::set_device_name,
            db::commands::db_init_workspace,
            db::commands::db_get_documents,
            db::commands::db_upsert_document,
            db::commands::db_delete_document,
            db::commands::db_get_folders,
            db::commands::db_create_folder,
            db::commands::db_delete_folder,
        ])
        .setup(|app| {
            identity::init(app.handle()).map_err(|e| error::AppError::Identity(e.to_string()))?;
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
