// src/api_server/commands.rs

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};
use crate::api_server::{
    state::ApiServerState,
    routes::create_api_router,
};

// Start the API server
pub async fn start_api_server_internal(api_state: Arc<Mutex<ApiServerState>>, port: u16) -> Result<(), String> {
    let (router, routes) = create_api_router();
    let app = router.with_state(api_state.clone());
    
    // Store routes in state
    {
        let mut state = api_state.lock().await;
        state.routes = routes;
    }
    
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    info!("Starting API server on {}", addr);
    
    // Updated server binding using axum::serve
    let listener = tokio::net::TcpListener::bind(addr).await
        .map_err(|e| format!("Failed to bind to address: {}", e))?;
    
    let server_handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app.into_make_service()).await {
            error!("Server error: {}", e);
        }
    });
    
    // Store server handle for potential shutdown later
    let mut state = api_state.lock().await;
    state.server_handle = Some(server_handle);
    
    Ok(())
}

// Tauri command to check if API server is running
#[tauri::command]
pub async fn is_api_server_running(
    api_state: tauri::State<'_, Arc<Mutex<ApiServerState>>>,
) -> Result<bool, ()> {
    let state = api_state.lock().await;
    Ok(state.server_handle.is_some())
}

// Tauri command to start API server
#[tauri::command]
pub async fn start_api_server(
    api_state: tauri::State<'_, Arc<Mutex<ApiServerState>>>,
    port: u16,
) -> Result<String, String> {
    // Check if server is already running
    let is_running = {
        let state = api_state.lock().await;
        state.server_handle.is_some()
    };
    
    if is_running {
        return Err("API server is already running".to_string());
    }
    
    start_api_server_internal(api_state.inner().clone(), port).await?;
    Ok(format!("API server started on port {}", port))
}

// Tauri command to stop API server
#[tauri::command]
pub async fn stop_api_server(
    api_state: tauri::State<'_, Arc<Mutex<ApiServerState>>>,
) -> Result<(), String> {
    let mut state = api_state.lock().await;
    if let Some(handle) = state.server_handle.take() {
        handle.abort();
        Ok(())
    } else {
        Err("API server is not running".to_string())
    }
}

// Tauri command to list API routes
#[tauri::command]
pub async fn list_api_routes(
    api_state: tauri::State<'_, Arc<Mutex<ApiServerState>>>,
) -> Result<Vec<String>, String> {
    let state = api_state.lock().await;
    Ok(state.routes.clone())
}