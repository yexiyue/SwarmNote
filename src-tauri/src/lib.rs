mod identity;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            identity::commands::get_device_info,
            identity::commands::set_device_name,
        ])
        .setup(|app| {
            identity::init(app.handle()).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

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
