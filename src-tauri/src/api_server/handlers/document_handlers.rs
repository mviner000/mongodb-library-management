// src/api_server/handlers/document_handlers.rs

use axum::{
    http::StatusCode,
    Json, 
    extract::{State, Path, Query},
    response::IntoResponse,
};
use axum_extra::{
    headers::{Authorization, authorization::Bearer},
    TypedHeader,
};
use mongodb::{
    bson::{doc, Document, oid::ObjectId}, 
    Cursor
};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use futures_util::StreamExt;
use std::collections::HashMap;
use chrono;

use crate::api_server::services::schema_service::get_collection_schema_internal;
use crate::api_server::state::ApiServerState;
use crate::api_server::models::{
    ApiResponse, InsertResponse, UpdateResponse, DeleteResponse, 
    error_response
};
use crate::api_server::services::database_service::{
    get_database, process_document_fields
};

// Document handlers
pub async fn find_documents_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection_name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let mongodb_state = &state.lock().await.mongodb_state;
    
    // Extract filter from query parameters - fixed temporary value issue
    let filter_str = params.get("filter").cloned().unwrap_or_else(|| String::from("{}"));
    
    // Parse the JSON string into a Document
    let filter: Document = match serde_json::from_str(&filter_str) {
        Ok(f) => f,
        Err(e) => return error_response::<Vec<Document>>(
            StatusCode::BAD_REQUEST, 
            format!("Invalid filter JSON: {}", e)
        ),
    };
    
    match get_database(mongodb_state).await {
        Ok(db) => {
            let collection = db.collection::<Document>(&collection_name);
            
            match collection.find(filter, None).await {
                Ok(cursor) => {
                    match process_cursor(cursor).await {
                        Ok(documents) => {
                            (StatusCode::OK, Json(ApiResponse {
                                success: true,
                                data: Some(documents),
                                error: None,
                            }))
                        },
                        Err(e) => error_response::<Vec<Document>>(StatusCode::INTERNAL_SERVER_ERROR, e),
                    }
                },
                Err(e) => error_response::<Vec<Document>>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            }
        },
        Err((status, e)) => error_response::<Vec<Document>>(status, e),
    }
}

pub async fn find_archived_documents_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection_name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let mongodb_state = &state.lock().await.mongodb_state;
    
    let filter_str = params.get("filter").cloned().unwrap_or_else(|| String::from("{}"));
    
    let mut filter: Document = match serde_json::from_str(&filter_str) {
        Ok(f) => f,
        Err(e) => return error_response::<Vec<Document>>(
            StatusCode::BAD_REQUEST, 
            format!("Invalid filter JSON: {}", e)
        ),
    };
    
    // Filter for archived documents
    filter.insert("is_archive", true);
    
    match get_database(mongodb_state).await {
        Ok(db) => {
            let collection = db.collection::<Document>(&collection_name);
            
            match collection.find(filter, None).await {
                Ok(cursor) => {
                    match process_cursor(cursor).await {
                        Ok(documents) => {
                            (StatusCode::OK, Json(ApiResponse {
                                success: true,
                                data: Some(documents),
                                error: None,
                            }))
                        },
                        Err(e) => error_response::<Vec<Document>>(StatusCode::INTERNAL_SERVER_ERROR, e),
                    }
                },
                Err(e) => error_response::<Vec<Document>>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            }
        },
        Err((status, e)) => error_response::<Vec<Document>>(status, e),
    }
}

pub async fn find_recovered_documents_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection_name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let mongodb_state = &state.lock().await.mongodb_state;
    
    let filter_str = params.get("filter").cloned().unwrap_or_else(|| String::from("{}"));
    
    let mut filter: Document = match serde_json::from_str(&filter_str) {
        Ok(f) => f,
        Err(e) => return error_response::<Vec<Document>>(
            StatusCode::BAD_REQUEST, 
            format!("Invalid filter JSON: {}", e)
        ),
    };
    
    // Correctly filter for documents where the latest archive action is "recover"
    filter.insert("is_archive", false);
    filter.insert("$expr", doc! {
        "$eq": [
            { "$arrayElemAt": ["$archive_history.action", -1] },
            "recover"
        ]
    });
    
    match get_database(mongodb_state).await {
        Ok(db) => {
            let collection = db.collection::<Document>(&collection_name);
            
            match collection.find(filter, None).await {
                Ok(cursor) => {
                    match process_cursor(cursor).await {
                        Ok(documents) => {
                            (StatusCode::OK, Json(ApiResponse {
                                success: true,
                                data: Some(documents),
                                error: None,
                            }))
                        },
                        Err(e) => error_response::<Vec<Document>>(StatusCode::INTERNAL_SERVER_ERROR, e),
                    }
                },
                Err(e) => error_response::<Vec<Document>>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            }
        },
        Err((status, e)) => error_response::<Vec<Document>>(status, e),
    }
}

pub async fn find_empty_archive_history_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection_name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let mongodb_state = &state.lock().await.mongodb_state;
    
    // Extract filter from query parameters
    let filter_str = params.get("filter").cloned().unwrap_or_else(|| String::from("{}"));
    
    // Parse the JSON string into a Document
    let mut filter: Document = match serde_json::from_str(&filter_str) {
        Ok(f) => f,
        Err(e) => return error_response::<Vec<Document>>(
            StatusCode::BAD_REQUEST, 
            format!("Invalid filter JSON: {}", e)
        ),
    };
    
    // Add condition for empty archive history
    filter.insert("$or", vec![
        doc! { "archive_history": doc! { "$exists": false } },
        doc! { "archive_history": doc! { "$size": 0 } }
    ]);
    
    match get_database(mongodb_state).await {
        Ok(db) => {
            let collection = db.collection::<Document>(&collection_name);
            
            match collection.find(filter, None).await {
                Ok(cursor) => {
                    match process_cursor(cursor).await {
                        Ok(documents) => {
                            (StatusCode::OK, Json(ApiResponse {
                                success: true,
                                data: Some(documents),
                                error: None,
                            }))
                        },
                        Err(e) => error_response::<Vec<Document>>(StatusCode::INTERNAL_SERVER_ERROR, e),
                    }
                },
                Err(e) => error_response::<Vec<Document>>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            }
        },
        Err((status, e)) => error_response::<Vec<Document>>(status, e),
    }
}

pub async fn find_empty_or_recovered_documents_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection_name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let mongodb_state = &state.lock().await.mongodb_state;
    
    // Extract filter from query parameters
    let filter_str = params.get("filter").cloned().unwrap_or_else(|| String::from("{}"));
    
    // Parse the JSON string into a Document
    let mut filter: Document = match serde_json::from_str(&filter_str) {
        Ok(f) => f,
        Err(e) => return error_response::<Vec<Document>>(
            StatusCode::BAD_REQUEST, 
            format!("Invalid filter JSON: {}", e)
        ),
    };
    
    // Build the combined $or condition
    filter.insert("$or", vec![
        // Condition 1: Empty archive history (either doesn't exist or array is empty)
        doc! {
            "$or": [
                { "archive_history": { "$exists": false } },
                { "archive_history": { "$size": 0 } }
            ]
        },
        // Condition 2: Latest archive action is 'recover' and array is not empty
        doc! {
            "archive_history.0": { "$exists": true },
            "$expr": {
                "$eq": [
                    { "$arrayElemAt": ["$archive_history.action", -1] },
                    "recover"
                ]
            }
        }
    ]);
    
    // Execute the query with the combined filter
    match get_database(mongodb_state).await {
        Ok(db) => {
            let collection = db.collection::<Document>(&collection_name);
            
            match collection.find(filter, None).await {
                Ok(cursor) => {
                    match process_cursor(cursor).await {
                        Ok(documents) => {
                            (StatusCode::OK, Json(ApiResponse {
                                success: true,
                                data: Some(documents),
                                error: None,
                            }))
                        },
                        Err(e) => error_response::<Vec<Document>>(StatusCode::INTERNAL_SERVER_ERROR, e),
                    }
                },
                Err(e) => error_response::<Vec<Document>>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            }
        },
        Err((status, e)) => error_response::<Vec<Document>>(status, e),
    }
}

pub async fn find_pinned_documents_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection_name): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    // Validate auth token and get user_id
    let user_id = match async {
        let state_guard = state.lock().await;
        let session_manager_mutex = &state_guard.session_manager;
        let session_manager = session_manager_mutex.lock().await;
        
        let token = auth.token();
        
        if session_manager.validate_session(token).await {
            // If session is valid, get the user ID
            match session_manager.get_user_id(token).await {
                Some(user_id) => Ok(user_id),
                None => Err("Session valid but user ID not found".to_string())
            }
        } else {
            Err("Invalid or expired session".to_string())
        }
    }.await {
        Ok(id) => id,
        Err(e) => return error_response::<Vec<Document>>(
            StatusCode::UNAUTHORIZED,
            format!("Authentication failed: {}", e)
        ),
    };
    
    let mongodb_state = &state.lock().await.mongodb_state;
    
    let filter_str = params.get("filter").cloned().unwrap_or_else(|| String::from("{}"));
    
    let mut filter: Document = match serde_json::from_str(&filter_str) {
        Ok(f) => f,
        Err(e) => return error_response::<Vec<Document>>(
            StatusCode::BAD_REQUEST, 
            format!("Invalid filter JSON: {}", e)
        ),
    };
    
    // Filter for documents pinned by this user
    filter.insert("pinned_by", user_id);
    
    match get_database(mongodb_state).await {
        Ok(db) => {
            // Check if the collection supports pinning
            let schema = match get_collection_schema_internal(&db, &collection_name).await {
                Ok(s) => s,
                Err(e) => return error_response::<Vec<Document>>(
                    StatusCode::INTERNAL_SERVER_ERROR, 
                    e
                ),
            };

            if !schema_has_pinned_property(&schema) {
                return error_response::<Vec<Document>>(
                    StatusCode::BAD_REQUEST,
                    "This collection does not support pinning.".into(),
                );
            }
            
            let collection = db.collection::<Document>(&collection_name);
            
            match collection.find(filter, None).await {
                Ok(cursor) => {
                    match process_cursor(cursor).await {
                        Ok(documents) => {
                            (StatusCode::OK, Json(ApiResponse {
                                success: true,
                                data: Some(documents),
                                error: None,
                            }))
                        },
                        Err(e) => error_response::<Vec<Document>>(StatusCode::INTERNAL_SERVER_ERROR, e),
                    }
                },
                Err(e) => error_response::<Vec<Document>>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            }
        },
        Err((status, e)) => error_response::<Vec<Document>>(status, e),
    }
}

pub async fn insert_document_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection_name): Path<String>,
    Json(document): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mongodb_state = &state.lock().await.mongodb_state;
    
    match get_database(mongodb_state).await {
        Ok(db) => {
            // Convert JSON to BSON document
            let doc_result = mongodb::bson::to_document(&document);
            match doc_result {
                Ok(mut doc) => {
                    // Remove any client-provided timestamp fields
                    doc.remove("created_at");
                    doc.remove("updated_at");
                    
                    // For attendance collection, also handle time_in_date
                    if collection_name == "attendance" {
                        doc.remove("time_in_date");
                    }
                    
                    // Add server-managed timestamp fields
                    let current_time = mongodb::bson::DateTime::now();
                    doc.insert("created_at", current_time.clone());
                    doc.insert("updated_at", current_time.clone());
                    
                    // For attendance collection, also set time_in_date
                    if collection_name == "attendance" {
                        doc.insert("time_in_date", current_time);
                    }
                    
                    // Process fields according to schema types (dates, integers, etc.)
                    if let Err(e) = process_document_fields(&db, &collection_name, &mut doc).await {
                        return error_response::<InsertResponse>(StatusCode::BAD_REQUEST, e);
                    }
                    
                    // Insert the document
                    let collection = db.collection::<Document>(&collection_name);
                    match collection.insert_one(doc, None).await {
                        Ok(result) => {
                            match result.inserted_id.as_object_id() {
                                Some(id) => {
                                    (StatusCode::CREATED, Json(ApiResponse {
                                        success: true,
                                        data: Some(InsertResponse { id: id.to_hex() }),
                                        error: None,
                                    }))
                                },
                                None => error_response::<InsertResponse>(
                                    StatusCode::INTERNAL_SERVER_ERROR, 
                                    "Failed to get inserted document ID".into()
                                ),
                            }
                        },
                        Err(e) => error_response::<InsertResponse>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
                    }
                },
                Err(e) => error_response::<InsertResponse>(
                    StatusCode::BAD_REQUEST, 
                    format!("Failed to convert document to BSON: {}", e)
                ),
            }
        },
        Err((status, e)) => error_response::<InsertResponse>(status, e),
    }
}

pub async fn update_document_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path((collection_name, id)): Path<(String, String)>,
    Json(update): Json<Document>,
) -> impl IntoResponse {
    let mongodb_state = &state.lock().await.mongodb_state;
    
    match get_database(mongodb_state).await {
        Ok(db) => {
            // Parse ObjectId
            match ObjectId::parse_str(&id) {
                Ok(object_id) => {
                    let collection = db.collection::<Document>(&collection_name);
                    let filter = doc! { "_id": object_id };
                    
                    // Process fields in the update document according to the schema
                    let mut update_doc = update.clone();
                    
                    // Remove any attempts to modify timestamp fields
                    update_doc.remove("created_at");
                    
                    // Process row_height if present
                    if let Some(row_height) = update_doc.get_i32("row_height").ok() {
                        update_doc.insert("row_height", row_height);
                    }
                    
                    // Always update the updated_at field with current timestamp
                    let current_time = mongodb::bson::DateTime::now();
                    update_doc.insert("updated_at", current_time);
                    
                    if let Err(e) = process_document_fields(&db, &collection_name, &mut update_doc).await {
                        return error_response::<UpdateResponse>(StatusCode::BAD_REQUEST, e);
                    }
                    
                    let update_bson = doc! { "$set": update_doc };
                    
                    match collection.update_one(filter, update_bson, None).await {
                        Ok(result) => {
                            (StatusCode::OK, Json(ApiResponse {
                                success: true,
                                data: Some(UpdateResponse {
                                    success: true,
                                    modified_count: result.modified_count,
                                }),
                                error: None,
                            }))
                        },
                        Err(e) => error_response::<UpdateResponse>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
                    }
                },
                Err(e) => error_response::<UpdateResponse>(
                    StatusCode::BAD_REQUEST, 
                    format!("Invalid ObjectId: {}", e)
                ),
            }
        },
        Err((status, e)) => error_response::<UpdateResponse>(status, e),
    }
}

pub async fn delete_document_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path((collection_name, id)): Path<(String, String)>,
) -> impl IntoResponse {
    let mongodb_state = &state.lock().await.mongodb_state;
    
    match get_database(mongodb_state).await {
        Ok(db) => {
            // Parse ObjectId
            match ObjectId::parse_str(&id) {
                Ok(object_id) => {
                    let collection = db.collection::<Document>(&collection_name);
                    let filter = doc! { "_id": object_id };
                    
                    match collection.delete_one(filter, None).await {
                        Ok(result) => {
                            (StatusCode::OK, Json(ApiResponse {
                                success: true,
                                data: Some(DeleteResponse {
                                    success: true,
                                    deleted_count: result.deleted_count,
                                }),
                                error: None,
                            }))
                        },
                        Err(e) => error_response::<DeleteResponse>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
                    }
                },
                Err(e) => error_response::<DeleteResponse>(
                    StatusCode::BAD_REQUEST, 
                    format!("Invalid ObjectId: {}", e)
                ),
            }
        },
        Err((status, e)) => error_response::<DeleteResponse>(status, e),
    }
}

pub async fn batch_delete_documents_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection_name): Path<String>,
    Json(payload): Json<HashMap<String, Vec<String>>>,
) -> impl IntoResponse {
    let mongodb_state = &state.lock().await.mongodb_state;
    
    let ids = match payload.get("ids") {
        Some(ids) => ids,
        None => return error_response::<DeleteResponse>(
            StatusCode::BAD_REQUEST, 
            "Missing 'ids' in payload".into()
        ),
    };

    let object_ids: Result<Vec<ObjectId>, _> = ids.iter()
        .map(|id| ObjectId::parse_str(id))
        .collect();

    let object_ids = match object_ids {
        Ok(ids) => ids,
        Err(e) => return error_response::<DeleteResponse>(
            StatusCode::BAD_REQUEST, 
            format!("Invalid ObjectId: {}", e)
        ),
    };

    match get_database(mongodb_state).await {
        Ok(db) => {
            let collection = db.collection::<Document>(&collection_name);
            let filter = doc! { "_id": { "$in": object_ids } };
            
            match collection.delete_many(filter, None).await {
                Ok(result) => {
                    (StatusCode::OK, Json(ApiResponse {
                        success: true,
                        data: Some(DeleteResponse {
                            success: true,
                            deleted_count: result.deleted_count,
                        }),
                        error: None,
                    }))
                },
                Err(e) => error_response::<DeleteResponse>(
                    StatusCode::INTERNAL_SERVER_ERROR, 
                    e.to_string()
                ),
            }
        },
        Err((status, e)) => error_response::<DeleteResponse>(status, e),
    }
}

// archive handlers

pub async fn archive_document_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path((collection_name, id)): Path<(String, String)>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    let token = auth.token();
    let state = state.lock().await;
    let session_manager = &state.session_manager;
    
    // Validate session
    let valid = session_manager.lock().await.validate_session(token).await;
    if !valid {
        return error_response::<()>(StatusCode::UNAUTHORIZED, "Invalid session".into());
    }

    // Get user ID from session
    let user_id = match session_manager.lock().await.get_user_id(token).await {
        Some(id) => id,
        None => return error_response::<()>(StatusCode::UNAUTHORIZED, "Session expired".into()),
    };

    // Convert to ObjectId
    let user_oid = match ObjectId::parse_str(&user_id) {
        Ok(oid) => oid,
        Err(_) => return error_response::<()>(StatusCode::INTERNAL_SERVER_ERROR, "Invalid user ID format".into()),
    };

    // Process the archive operation
    match get_database(&state.mongodb_state).await {
        Ok(db) => {
            let collection = db.collection::<Document>(&collection_name);
            let doc_id = match ObjectId::parse_str(&id) {
                Ok(oid) => oid,
                Err(e) => return error_response::<()>(StatusCode::BAD_REQUEST, format!("Invalid document ID: {}", e)),
            };

            // Create timestamp
            let now = mongodb::bson::DateTime::now();

            // Try to update only if not already archived
            let filter = doc! {
                "_id": doc_id,
                "is_archive": { "$ne": true }
            };

            let update = doc! {
                "$set": { "is_archive": true },
                "$push": {
                    "archive_history": {
                        "action": "archive",
                        "user_id": user_oid,
                        "timestamp": now
                    }
                }
            };

            match collection.update_one(filter, update, None).await {
                Ok(result) => {
                    if result.matched_count == 0 {
                        // Check if document exists
                        match collection.count_documents(doc! { "_id": doc_id }, None).await {
                            Ok(count) if count > 0 => {
                                // Document exists but already archived
                                (StatusCode::OK, Json(ApiResponse {
                                    success: true,
                                    data: Some(()),
                                    error: None,
                                }))
                            },
                            _ => error_response::<()>(StatusCode::NOT_FOUND, "Document not found".into()),
                        }
                    } else {
                        (StatusCode::OK, Json(ApiResponse {
                            success: true,
                            data: Some(()),
                            error: None,
                        }))
                    }
                },
                Err(e) => error_response::<()>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            }
        },
        Err((status, e)) => error_response::<()>(status, e),
    }
}

pub async fn batch_archive_documents_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection_name): Path<String>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<HashMap<String, Vec<String>>>,
) -> impl IntoResponse {
    let token = auth.token();
    let state = state.lock().await;
    let session_manager = &state.session_manager;
    
    // Authentication check
    let valid = session_manager.lock().await.validate_session(token).await;
    if !valid {
        return error_response::<serde_json::Value>(
            StatusCode::UNAUTHORIZED, 
            "Invalid session".into()
        );
    }

    // Get user ID from session
    let user_id = match session_manager.lock().await.get_user_id(token).await {
        Some(id) => id,
        None => return error_response::<serde_json::Value>(
            StatusCode::UNAUTHORIZED, 
            "Session expired".into()
        ),
    };

    // Convert to ObjectId
    let user_oid = match ObjectId::parse_str(&user_id) {
        Ok(oid) => oid,
        Err(_) => return error_response::<serde_json::Value>(
            StatusCode::INTERNAL_SERVER_ERROR, 
            "Invalid user ID format".into()
        ),
    };

    let ids = match payload.get("ids") {
        Some(ids) => ids,
        None => return error_response::<serde_json::Value>(
            StatusCode::BAD_REQUEST, 
            "Missing 'ids' in payload".into()
        ),
    };

    // Convert string IDs to ObjectIds
    let object_ids: Result<Vec<ObjectId>, _> = ids.iter()
        .map(|id| ObjectId::parse_str(id))
        .collect();

    let object_ids = match object_ids {
        Ok(ids) => ids,
        Err(e) => return error_response::<serde_json::Value>(
            StatusCode::BAD_REQUEST, 
            format!("Invalid ObjectId: {}", e)
        ),
    };

    match get_database(&state.mongodb_state).await {
        Ok(db) => {
            let collection = db.collection::<Document>(&collection_name);
            let now = mongodb::bson::DateTime::now();

            // First check if all documents are already archived
            let count_total = match collection.count_documents(
                doc! { "_id": { "$in": &object_ids } },
                None
            ).await {
                Ok(count) => count,
                Err(e) => return error_response::<serde_json::Value>(
                    StatusCode::INTERNAL_SERVER_ERROR, 
                    e.to_string()
                ),
            };

            let count_already_archived = match collection.count_documents(
                doc! { 
                    "_id": { "$in": &object_ids },
                    "is_archive": true 
                },
                None
            ).await {
                Ok(count) => count,
                Err(e) => return error_response::<serde_json::Value>(
                    StatusCode::INTERNAL_SERVER_ERROR, 
                    e.to_string()
                ),
            };

            // If all documents are already archived
            if count_already_archived == count_total {
                return (StatusCode::OK, Json(ApiResponse {
                    success: true,
                    data: Some(json!({
                        "message": "All selected documents are already archived",
                        "archived_count": 0
                    })),
                    error: None,
                }));
            }

            // Update only non-archived documents
            let filter = doc! {
                "_id": { "$in": &object_ids },
                "is_archive": { "$ne": true }
            };

            let update = doc! {
                "$set": { "is_archive": true },
                "$push": {
                    "archive_history": {
                        "action": "archive",
                        "user_id": user_oid,
                        "timestamp": now
                    }
                }
            };

            match collection.update_many(filter, update, None).await {
                Ok(result) => {
                    (StatusCode::OK, Json(ApiResponse {
                        success: true,
                        data: Some(json!({
                            "message": format!("Successfully archived {} documents", result.modified_count),
                            "archived_count": result.modified_count
                        })),
                        error: None,
                    }))
                },
                Err(e) => error_response::<serde_json::Value>(
                    StatusCode::INTERNAL_SERVER_ERROR, 
                    e.to_string()
                ),
            }
        },
        Err((status, e)) => error_response::<serde_json::Value>(status, e),
    }
}

// recovery handlers

pub async fn recover_document_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path((collection_name, id)): Path<(String, String)>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    let token = auth.token();
    let state = state.lock().await;
    let session_manager = &state.session_manager;
    
    // Validate session
    let valid = session_manager.lock().await.validate_session(token).await;
    if !valid {
        return error_response::<()>(StatusCode::UNAUTHORIZED, "Invalid session".into());
    }

    // Get user ID from session
    let user_id = match session_manager.lock().await.get_user_id(token).await {
        Some(id) => id,
        None => return error_response::<()>(StatusCode::UNAUTHORIZED, "Session expired".into()),
    };

    // Convert to ObjectId
    let user_oid = match ObjectId::parse_str(&user_id) {
        Ok(oid) => oid,
        Err(_) => return error_response::<()>(StatusCode::INTERNAL_SERVER_ERROR, "Invalid user ID format".into()),
    };

    match get_database(&state.mongodb_state).await {
        Ok(db) => {
            let collection = db.collection::<Document>(&collection_name);
            let doc_id = match ObjectId::parse_str(&id) {
                Ok(oid) => oid,
                Err(e) => return error_response::<()>(StatusCode::BAD_REQUEST, format!("Invalid document ID: {}", e)),
            };

            let now = mongodb::bson::DateTime::now();

            // Try to update only if archived
            let filter = doc! {
                "_id": doc_id,
                "is_archive": true
            };

            let update = doc! {
                "$set": { "is_archive": false },
                "$push": {
                    "archive_history": {
                        "action": "recover",
                        "user_id": user_oid,
                        "timestamp": now
                    }
                }
            };

            match collection.update_one(filter, update, None).await {
                Ok(result) => {
                    if result.matched_count == 0 {
                        match collection.count_documents(doc! { "_id": doc_id }, None).await {
                            Ok(count) if count > 0 => (StatusCode::OK, Json(ApiResponse {
                                success: true,
                                data: Some(()),
                                error: None,
                            })),
                            _ => error_response::<()>(StatusCode::NOT_FOUND, "Document not found".into()),
                        }
                    } else {
                        (StatusCode::OK, Json(ApiResponse {
                            success: true,
                            data: Some(()),
                            error: None,
                        }))
                    }
                },
                Err(e) => error_response::<()>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            }
        },
        Err((status, e)) => error_response::<()>(status, e),
    }
}

pub async fn batch_recover_documents_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection_name): Path<String>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<HashMap<String, Vec<String>>>,
) -> impl IntoResponse {
    let token = auth.token();
    let state = state.lock().await;
    let session_manager = &state.session_manager;
    
    let valid = session_manager.lock().await.validate_session(token).await;
    if !valid {
        return error_response::<serde_json::Value>(
            StatusCode::UNAUTHORIZED, 
            "Invalid session".into()
        );
    }

    let user_id = match session_manager.lock().await.get_user_id(token).await {
        Some(id) => id,
        None => return error_response::<serde_json::Value>(
            StatusCode::UNAUTHORIZED, 
            "Session expired".into()
        ),
    };

    let user_oid = match ObjectId::parse_str(&user_id) {
        Ok(oid) => oid,
        Err(_) => return error_response::<serde_json::Value>(
            StatusCode::INTERNAL_SERVER_ERROR, 
            "Invalid user ID format".into()
        ),
    };

    let ids = match payload.get("ids") {
        Some(ids) => ids,
        None => return error_response::<serde_json::Value>(
            StatusCode::BAD_REQUEST, 
            "Missing 'ids' in payload".into()
        ),
    };

    let object_ids: Result<Vec<ObjectId>, _> = ids.iter()
        .map(|id| ObjectId::parse_str(id))
        .collect();

    let object_ids = match object_ids {
        Ok(ids) => ids,
        Err(e) => return error_response::<serde_json::Value>(
            StatusCode::BAD_REQUEST, 
            format!("Invalid ObjectId: {}", e)
        ),
    };

    match get_database(&state.mongodb_state).await {
        Ok(db) => {
            let collection = db.collection::<Document>(&collection_name);
            let now = mongodb::bson::DateTime::now();

            let count_archived = match collection.count_documents(
                doc! { 
                    "_id": { "$in": &object_ids },
                    "is_archive": true 
                },
                None
            ).await {
                Ok(count) => count,
                Err(e) => return error_response::<serde_json::Value>(
                    StatusCode::INTERNAL_SERVER_ERROR, 
                    e.to_string()
                ),
            };

            if count_archived == 0 {
                return (StatusCode::OK, Json(ApiResponse {
                    success: true,
                    data: Some(json!({
                        "message": "No archived documents found to recover",
                        "recovered_count": 0
                    })),
                    error: None,
                }));
            }

            let filter = doc! {
                "_id": { "$in": &object_ids },
                "is_archive": true
            };

            let update = doc! {
                "$set": { "is_archive": false },
                "$push": {
                    "archive_history": {
                        "action": "recover",
                        "user_id": user_oid,
                        "timestamp": now
                    }
                }
            };

            match collection.update_many(filter, update, None).await {
                Ok(result) => {
                    (StatusCode::OK, Json(ApiResponse {
                        success: true,
                        data: Some(json!({
                            "message": format!("Successfully recovered {} documents", result.modified_count),
                            "recovered_count": result.modified_count
                        })),
                        error: None,
                    }))
                },
                Err(e) => error_response::<serde_json::Value>(
                    StatusCode::INTERNAL_SERVER_ERROR, 
                    e.to_string()
                ),
            }
        },
        Err((status, e)) => error_response::<serde_json::Value>(status, e),
    }
}

// pin and unpin handlers
pub async fn pin_document_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path((collection_name, id)): Path<(String, String)>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    tracing::debug!(
        "pin_document_handler called: collection={}, document_id={}", 
        collection_name, id
    );
    
    let token = auth.token();
    let state = state.lock().await;
    let session_manager = &state.session_manager;
    
    let valid = session_manager.lock().await.validate_session(token).await;
    if !valid {
        tracing::warn!("pin_document_handler: Invalid session token");
        return error_response::<Document>(StatusCode::UNAUTHORIZED, "Invalid session".into());
    }

    let user_id = match session_manager.lock().await.get_user_id(token).await {
        Some(id) => id,
        None => {
            tracing::warn!("pin_document_handler: Session expired");
            return error_response::<Document>(StatusCode::UNAUTHORIZED, "Session expired".into());
        }
    };

    // temporary commented, TODO: check this part if truly not needed
    // let user_oid = match ObjectId::parse_str(&user_id) {
    //     Ok(oid) => oid,
    //     Err(e) => {
    //         tracing::error!("pin_document_handler: Invalid user ID format: {}", e);
    //         return error_response::<Document>(StatusCode::INTERNAL_SERVER_ERROR, "Invalid user ID format".into());
    //     }
    // };

    match get_database(&state.mongodb_state).await {
        Ok(db) => {
            let schema = match get_collection_schema_internal(&db, &collection_name).await {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("pin_document_handler: Failed to get schema: {}", e);
                    return error_response::<Document>(StatusCode::INTERNAL_SERVER_ERROR, e);
                }
            };

            if !schema_has_pinned_property(&schema) {
                tracing::warn!("pin_document_handler: Collection doesn't support pinning");
                return error_response::<Document>(
                    StatusCode::BAD_REQUEST,
                    "Collection does not support pinning".into(),
                );
            }

            let collection = db.collection::<Document>(&collection_name);
            let doc_id = match ObjectId::parse_str(&id) {
                Ok(oid) => oid,
                Err(e) => {
                    tracing::warn!("pin_document_handler: Invalid document ID: {}", e);
                    return error_response::<Document>(StatusCode::BAD_REQUEST, format!("Invalid document ID: {}", e));
                }
            };

            let now = mongodb::bson::DateTime::now();
            let filter = doc! {
                "_id": doc_id,
                "pinned_by": { "$ne": &user_id }  // Use a reference here
            };
            let update = doc! {
                "$addToSet": { "pinned_by": &user_id },
                "$push": {
                    "pinned_history": {
                        "action": "pin",
                        "user_id": &user_id, // Store the user_id string
                        "timestamp": now
                    }
                },
                "$set": { "updated_at": now }
            };            
            
            let options = FindOneAndUpdateOptions::builder()
                .return_document(ReturnDocument::After)
                .build();

                match collection.find_one_and_update(filter, update, options).await {
                    Ok(Some(mut updated_doc)) => {
                        format_date_fields(&mut updated_doc);
                        tracing::info!("Successfully pinned document {}", id);
                        let response = Json(ApiResponse {
                            success: true,
                            data: Some(updated_doc),
                            error: None,
                        });
                        tracing::info!("Response: {:?}", response); // Log the success response
                        (StatusCode::OK, response)
                    },
                Ok(None) => {
                    match collection.find_one(doc! { "_id": doc_id }, None).await {
                        Ok(Some(mut existing_doc)) => {
                            format_date_fields(&mut existing_doc);
                            (StatusCode::OK, Json(ApiResponse {
                                success: true,
                                data: Some(existing_doc),
                                error: None,
                            }))
                        },
                        Ok(None) => {
                            tracing::warn!("Document not found: {}", id);
                            error_response::<Document>(StatusCode::NOT_FOUND, "Document not found".into())
                        },
                        Err(e) => {
                            tracing::error!("Database error: {}", e);
                            error_response::<Document>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                        }
                    }
                },
                Err(e) => {
                    tracing::error!("Update error: {}", e);
                    error_response::<Document>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                }
            }
        },
        Err((status, e)) => {
            tracing::error!("Database connection error: {}", e);
            error_response::<Document>(status, e)
        },
    }
}

pub async fn unpin_document_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path((collection_name, id)): Path<(String, String)>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    tracing::debug!(
        "unpin_document_handler called: collection={}, document_id={}", 
        collection_name, id
    );
    
    let token = auth.token();
    let state = state.lock().await;
    let session_manager = &state.session_manager;
    
    let valid = session_manager.lock().await.validate_session(token).await;
    if !valid {
        tracing::warn!("unpin_document_handler: Invalid session token");
        return error_response::<Document>(StatusCode::UNAUTHORIZED, "Invalid session".into());
    }

    let user_id = match session_manager.lock().await.get_user_id(token).await {
        Some(id) => id,
        None => {
            tracing::warn!("unpin_document_handler: Session expired");
            return error_response::<Document>(StatusCode::UNAUTHORIZED, "Session expired".into());
        }
    };

    // temporary commented, TODO: check this part if truly not needed
    // let user_oid = match ObjectId::parse_str(&user_id) {
    //     Ok(oid) => oid,
    //     Err(e) => {
    //         tracing::error!("unpin_document_handler: Invalid user ID format: {}", e);
    //         return error_response::<Document>(StatusCode::INTERNAL_SERVER_ERROR, "Invalid user ID format".into());
    //     }
    // };

    match get_database(&state.mongodb_state).await {
        Ok(db) => {
            let schema = match get_collection_schema_internal(&db, &collection_name).await {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("unpin_document_handler: Failed to get schema: {}", e);
                    return error_response::<Document>(StatusCode::INTERNAL_SERVER_ERROR, e);
                }
            };

            if !schema_has_pinned_property(&schema) {
                tracing::warn!("unpin_document_handler: Collection doesn't support pinning");
                return error_response::<Document>(
                    StatusCode::BAD_REQUEST,
                    "Collection does not support pinning".into(),
                );
            }

            let collection = db.collection::<Document>(&collection_name);
            let doc_id = match ObjectId::parse_str(&id) {
                Ok(oid) => oid,
                Err(e) => {
                    tracing::warn!("unpin_document_handler: Invalid document ID: {}", e);
                    return error_response::<Document>(StatusCode::BAD_REQUEST, format!("Invalid document ID: {}", e));
                }
            };

            let now = mongodb::bson::DateTime::now();
            let filter = doc! {
                "_id": doc_id,
                "pinned_by": &user_id  // Use a reference here
            };
            let update = doc! {
                "$pull": { "pinned_by": &user_id },
                "$push": {
                    "pinned_history": {
                        "action": "unpin",
                        "user_id": &user_id, // Store the user_id string
                        "timestamp": now
                    }
                },
                "$set": { "updated_at": now }
            };

            let options = FindOneAndUpdateOptions::builder()
                .return_document(ReturnDocument::After)
                .build();

            match collection.find_one_and_update(filter, update, options).await {
                Ok(Some(mut updated_doc)) => {
                    format_date_fields(&mut updated_doc);
                    tracing::info!("Successfully unpinned document {}", id);
                    let response = Json(ApiResponse {
                        success: true,
                        data: Some(updated_doc),
                        error: None,
                    });
                    tracing::info!("Response: {:?}", response); // Log the success response
                    (StatusCode::OK, response)
                },
                Ok(None) => {
                    match collection.find_one(doc! { "_id": doc_id }, None).await {
                        Ok(Some(mut existing_doc)) => {
                            format_date_fields(&mut existing_doc);
                            (StatusCode::OK, Json(ApiResponse {
                                success: true,
                                data: Some(existing_doc),
                                error: None,
                            }))
                        },
                        Ok(None) => {
                            tracing::warn!("Document not found: {}", id);
                            error_response::<Document>(StatusCode::NOT_FOUND, "Document not found".into())
                        },
                        Err(e) => {
                            tracing::error!("Database error: {}", e);
                            error_response::<Document>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                        }
                    }
                },
                Err(e) => {
                    tracing::error!("Update error: {}", e);
                    error_response::<Document>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                }
            }
        },
        Err((status, e)) => {
            tracing::error!("Database connection error: {}", e);
            error_response::<Document>(status, e)
        },
    }
}

// Helper function to check if the schema includes the 'pinned_by' property
fn schema_has_pinned_property(schema: &Document) -> bool {
    if let Ok(properties) = schema.get_document("properties") {
        let has = properties.contains_key("pinned_by");
        tracing::debug!("Schema has pinned_by: {}", has);
        has
    } else {
        tracing::error!("Schema properties not found");
        false
    }
}

// Helper functions for document handlers
pub async fn process_cursor(
    mut cursor: Cursor<Document>
) -> Result<Vec<Document>, String> {
    let mut documents = Vec::new();
    while let Some(document_result) = cursor.next().await {
        match document_result {
            Ok(mut doc) => {
                format_date_fields(&mut doc);
                documents.push(doc);
            },
            Err(e) => return Err(format!("Error retrieving document: {}", e)),
        }
    }
    
    Ok(documents)
}

pub fn format_date_fields(doc: &mut Document) {
    // Similar to your existing implementation
    let keys: Vec<String> = doc.keys().cloned().collect();
    
    for key in keys {
        if let Some(mongodb::bson::Bson::DateTime(date_time)) = doc.get(&key) {
            let chrono_date = chrono::DateTime::from_timestamp_millis(date_time.timestamp_millis())
                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
            
            let formatted_date = chrono_date.format("%Y-%m-%d %H:%M:%S").to_string();
            doc.insert(key, mongodb::bson::Bson::String(formatted_date));
        }
    }
}