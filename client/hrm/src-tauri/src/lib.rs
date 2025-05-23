// src/lib.rs

use tauri;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}