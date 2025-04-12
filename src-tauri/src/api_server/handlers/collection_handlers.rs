// src/api_server/handlers/collection_handlers.rs

use axum::{
    http::StatusCode,
    Json,
    extract::{State, Path},
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::api_server::state::ApiServerState;
use crate::api_server::models::{ApiResponse, error_response};
use crate::api_server::services::database_service::get_database;
use crate::api_server::services::get_collection_schema_with_ui;
use crate::api_server::services::update_ui_metadata;

// Collection handlers
pub async fn list_collections_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>
) -> impl IntoResponse {
    let mongodb_state = &state.lock().await.mongodb_state;
    
    match get_database(mongodb_state).await {
        Ok(db) => {
            match db.list_collection_names(None).await {
                Ok(collections) => {
                    (StatusCode::OK, Json(ApiResponse {
                        success: true,
                        data: Some(collections),
                        error: None,
                    }))
                },
                Err(e) => error_response::<Vec<String>>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            }
        },
        Err((status, e)) => error_response::<Vec<String>>(status, e),
    }
}

pub async fn get_collection_schema_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection_name): Path<String>,
) -> impl IntoResponse {
    let mongodb_state = &state.lock().await.mongodb_state;
    
    match get_database(mongodb_state).await {
        Ok(db) => {
            match get_collection_schema_with_ui(&db, &collection_name).await {
                Ok(merged_schema) => {
                    // Convert merged schema to JSON
                    match bson::from_bson(bson::Bson::Document(merged_schema)) {
                        Ok(merged_schema_json) => {
                            (StatusCode::OK, Json(ApiResponse {
                                success: true,
                                data: Some(merged_schema_json),
                                error: None,
                            }))
                        },
                        Err(e) => error_response::<serde_json::Value>(
                            StatusCode::INTERNAL_SERVER_ERROR, 
                            format!("Failed to convert merged schema to JSON: {}", e)
                        ),
                    }
                },
                Err(e) => error_response::<serde_json::Value>(StatusCode::INTERNAL_SERVER_ERROR, e),
            }
        },
        Err((status, e)) => error_response::<serde_json::Value>(status, e),
    }
}

pub async fn update_ui_metadata_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection_name): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mongodb_state = &state.lock().await.mongodb_state;
    let db = match get_database(mongodb_state).await {
        Ok(db) => db,
        Err((status, e)) => return error_response::<()>(status, e),
    };

    // Extract column widths from payload
    let column_widths = match payload.get("columnWidths")
        .and_then(|v| v.as_object()) {
        Some(widths) => serde_json::Value::Object(widths.clone()),
        None => return error_response::<()>(StatusCode::BAD_REQUEST, "Invalid columnWidths format".into()),
    };

    // Use the service function instead of duplicating logic
    match update_ui_metadata(&db, &collection_name, &column_widths).await {
        Ok(_) => (StatusCode::OK, Json(ApiResponse {
            success: true,
            data: None,
            error: None,
        })),
        Err(e) => error_response::<()>(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}