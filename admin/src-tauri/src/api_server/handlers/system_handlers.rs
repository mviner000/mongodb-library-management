// src/api_server/handlers/system_handlers.rs

use axum::{
    http::StatusCode,
    Json, 
    extract::State,
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

use crate::api_server::state::ApiServerState;
use crate::api_server::models::{ApiResponse, error_response};
use crate::api_server::services::database_service::get_database;
use crate::mongodb_schema;

// System handlers
pub async fn health_check_handler() -> impl IntoResponse {
    // Simple 200 OK response
    StatusCode::OK
}

pub async fn initialize_library_collections_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
) -> impl IntoResponse {
    info!("API endpoint called: initialize library collections");
    let mongodb_state = &state.lock().await.mongodb_state;
    match get_database(mongodb_state).await {
        Ok(db) => {
            match mongodb_schema::initialize_library_collections(&db).await {
                Ok(_) => {
                    info!("Successfully initialized library collections via API endpoint");
                    (StatusCode::OK, Json(ApiResponse {
                        success: true,
                        data: None,
                        error: None,
                    }))
                },
                Err(e) => {
                    error!("API endpoint error: Failed to initialize library collections: {}", e);
                    error_response::<()>(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Initialization failed: {}", e)
                    )
                },
            }
        },
        Err((status, e)) => {
            error!("API endpoint error: Database connection failed: {}", e);
            error_response::<()>(status, e)
        },
    }
}