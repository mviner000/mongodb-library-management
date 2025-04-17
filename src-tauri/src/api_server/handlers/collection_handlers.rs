// src/api_server/handlers/collection_handlers.rs

use axum::{
    http::StatusCode,
    Json,
    extract::{State, Path},
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use mongodb::Database;
use mongodb::bson::{doc, Document};
use anyhow::Result;
use std::collections::HashSet;
use futures_util::stream::StreamExt;

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


pub async fn get_required_and_unique_fields(db: &Database, coll_name: &str) -> Result<Vec<String>, mongodb::error::Error> {
    // Retrieve collection information to extract required fields
    let coll_info = db.run_command(doc! {
        "listCollections": 1,
        "filter": { "name": coll_name },
        "nameOnly": false,
    }, None).await?;

    let required_fields = extract_required_fields(&coll_info).unwrap_or_default();

    // Retrieve all indexes for the collection
    let mut cursor = db.collection::<Document>(coll_name).list_indexes(None).await?;

    // Collect all fields from unique indexes
    let mut unique_fields = HashSet::new();
    
    // Use StreamExt to iterate over the cursor asynchronously
    while let Some(index_result) = cursor.next().await {
        if let Ok(index) = index_result {
            // Check if this index is unique
            if index.options.as_ref().and_then(|opts| opts.unique).unwrap_or(false) {
                // Extract the key fields from this unique index
                for (field, _) in index.keys.iter() {
                    unique_fields.insert(field.clone());
                }
            }
        }
    }

    // Find intersection of required and unique fields
    let result: Vec<String> = required_fields.into_iter()
        .filter(|field| unique_fields.contains(field))
        .collect();

    Ok(result)
}

fn extract_required_fields(coll_info: &Document) -> Option<Vec<String>> {
    let cursor = coll_info.get_document("cursor").ok()?;
    let first_batch = cursor.get_array("firstBatch").ok()?;
    let coll_doc = first_batch.first()?.as_document()?;
    let options = coll_doc.get_document("options").ok()?;
    let validator = options.get_document("validator").ok()?;
    let json_schema = validator.get_document("$jsonSchema").ok()?;
    let required = json_schema.get_array("required").ok()?;

    Some(required.iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect())
}

pub async fn get_collection_schema_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection_name): Path<String>,
) -> impl IntoResponse {
    let mongodb_state = &state.lock().await.mongodb_state;
    
    match get_database(mongodb_state).await {
        Ok(db) => {
            // Get the schema with UI metadata
            let schema_result = get_collection_schema_with_ui(&db, &collection_name).await;
            
            // Get the required and unique fields
            let required_unique_result = get_required_and_unique_fields(&db, &collection_name).await;
            
            match (schema_result, required_unique_result) {
                (Ok(mut merged_schema), Ok(required_unique_fields)) => {
                    // Add the first required and unique field to the schema (if any exists)
                    let primary_key = required_unique_fields.first().cloned();
                    
                    // Insert the primary key into the merged schema
                    merged_schema.insert("primaryKey", bson::to_bson(&primary_key).unwrap_or(bson::Bson::Null));
                    
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
                (Ok(_), Err(e)) => error_response::<serde_json::Value>(
                    StatusCode::INTERNAL_SERVER_ERROR, 
                    format!("Failed to get required and unique fields: {}", e)
                ),
                (Err(e), _) => error_response::<serde_json::Value>(
                    StatusCode::INTERNAL_SERVER_ERROR, 
                    e
                ),
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