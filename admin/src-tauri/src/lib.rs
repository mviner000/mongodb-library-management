// src/lib.rs

use tauri::{Manager, State};
use std::sync::Arc;
use tokio::sync::Mutex;

mod mongodb_installer;
mod mongodb_manager;
mod mongodb_schema;
mod lib_mongodb_schema;
mod api_server;
mod session; // Add the session module
mod auth; // Add the auth module

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing (for API server logs)
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Initialize MongoDB state
            let mongodb_state = mongodb_manager::MongoDbState::new("app_database");
            app.manage(mongodb_state.clone());

            // Initialize session manager
            let session_manager = session::SessionManager::new(mongodb_state.clone());
            app.manage(session_manager.clone());

            // Initialize API server state with MongoDB reference
            let api_server_state = Arc::new(Mutex::new(api_server::ApiServerState::new(
                mongodb_state.clone(),
                session_manager.clone()
            )));
            
            // Clone the state for the auto-start task
            let auto_start_state = api_server_state.clone();
            
            // Manage the state for command handlers
            app.manage(api_server_state);

            // Auto-start API server
            let port = 3000;
            tauri::async_runtime::spawn(async move {
                if let Err(e) = api_server::start_server(auto_start_state, port).await {
                    eprintln!("Failed to auto-start API server: {}", e);
                }
            });

            // Auto-connect if MongoDB is installed
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if mongodb_installer::is_mongodb_installed().await {
                    let state: State<'_, mongodb_manager::MongoDbState> = app_handle.state();
                    if let Err(e) = mongodb_manager::auto_connect(&state).await {
                        eprintln!("Auto-connect failed: {}", e);
                    }
                }
            });    

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            // MongoDB installation commands
            mongodb_installer::is_mongodb_installed,
            mongodb_installer::install_mongodb,
            
            // MongoDB database operations
            mongodb_manager::connect_mongodb,
            mongodb_manager::disconnect_mongodb,
            mongodb_manager::insert_document,
            mongodb_manager::find_documents,
            mongodb_manager::update_document,
            mongodb_manager::delete_document,
            mongodb_manager::list_collections,
            mongodb_manager::get_collection_schema,
            mongodb_manager::initialize_library_collections,
            
            // API server commands
            api_server::is_api_server_running,
            api_server::start_api_server,
            api_server::stop_api_server,
            api_server::list_api_routes,
            
            // Authentication commands
            auth::login,
            auth::register,
            auth::check_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}