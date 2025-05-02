// src/api_server/mod.rs

pub mod state;
pub mod models;
pub mod handlers;
pub mod services;
pub mod routes;
pub mod commands;

// Re-export the main components for backwards compatibility
pub use state::ApiServerState;
pub use commands::*;

// Start the API server function for backwards compatibility
pub async fn start_server(api_state: std::sync::Arc<tokio::sync::Mutex<ApiServerState>>, port: u16) -> Result<(), String> {
    commands::start_api_server_internal(api_state, port).await
}