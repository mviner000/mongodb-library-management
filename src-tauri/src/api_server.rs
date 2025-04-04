// src/api_server.rs

use axum::{
    routing::{get, post, put, delete},
    http::{StatusCode, Method},
    Json, Router, extract::Path, extract::State, response::IntoResponse,
};
use mongodb::{bson::{doc, Document, oid::ObjectId}, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, error};
use std::time::SystemTime;

use crate::mongodb_manager::MongoDbState;

// API server state
pub struct ApiServerState {
    mongodb_state: Arc<Mutex<MongoDbState>>,
    server_handle: Option<tokio::task::JoinHandle<()>>,
}

impl ApiServerState {
    pub fn new(mongodb_state: MongoDbState) -> Self {
        Self {
            mongodb_state: Arc::new(Mutex::new(mongodb_state)),
            server_handle: None,
        }
    }
}

// Document response types
#[derive(Serialize)]
pub struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct InsertResponse {
    id: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateResponse {
    success: bool,
    modified_count: u64,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteResponse {
    success: bool,
    deleted_count: u64,
}

// Error helper - Updated to return concrete type
fn error_response<T: Serialize>(status: StatusCode, message: String) -> (StatusCode, Json<ApiResponse<T>>) {
    (status, Json(ApiResponse {
        success: false,
        data: None,
        error: Some(message),
    }))
}

// Route handlers
async fn get_database(mongodb_state: &Arc<Mutex<MongoDbState>>) -> Result<Database, (StatusCode, String)> {
    let state = mongodb_state.lock().await;
    match state.get_database().await {
        Ok(db) => Ok(db),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

async fn list_collections_handler(
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

async fn get_collection_schema_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection_name): Path<String>,
) -> impl IntoResponse {
    let mongodb_state = &state.lock().await.mongodb_state;
    
    match get_database(mongodb_state).await {
        Ok(db) => {
            let command = doc! {
                "listCollections": 1,
                "filter": { "name": &collection_name }
            };
            
            match db.run_command(command, None).await {
                Ok(response) => {
                    match extract_schema_from_response(response) {
                        Ok(schema) => {
                            (StatusCode::OK, Json(ApiResponse {
                                success: true,
                                data: Some(schema),
                                error: None,
                            }))
                        },
                        Err(e) => error_response::<serde_json::Value>(StatusCode::INTERNAL_SERVER_ERROR, e),
                    }
                },
                Err(e) => error_response::<serde_json::Value>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            }
        },
        Err((status, e)) => error_response::<serde_json::Value>(status, e),
    }
}

fn extract_schema_from_response(response: Document) -> Result<serde_json::Value, String> {
    let cursor = response.get_document("cursor")
        .map_err(|e| format!("Invalid response format: {}", e))?;
    
    let batches = cursor.get_array("firstBatch")
        .map_err(|_| "No collections found".to_string())?;
    
    if batches.is_empty() {
        return Err("Collection not found".into());
    }
    
    let coll_info = batches[0].as_document()
        .ok_or_else(|| "Invalid collection info".to_string())?;
    
    let options = coll_info.get_document("options")
        .map_err(|_| "No options found".to_string())?;
    
    let validator = options.get_document("validator")
        .map_err(|_| "No validator found".to_string())?;
    
    let json_schema = validator.get_document("$jsonSchema")
        .map_err(|_| "No JSON schema found".to_string())?;
    
    mongodb::bson::from_bson(json_schema.clone().into())
        .map_err(|e| format!("BSON to JSON conversion failed: {}", e))
}

async fn find_documents_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection_name): Path<String>,
    Json(filter): Json<Document>,
) -> impl IntoResponse {
    let mongodb_state = &state.lock().await.mongodb_state;
    
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

async fn process_cursor(
    mut cursor: mongodb::Cursor<Document>
) -> Result<Vec<Document>, String> {
    use futures_util::StreamExt;
    
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

fn format_date_fields(doc: &mut Document) {
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

async fn insert_document_handler(
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
                    // Process date fields using the schema (similar to your implementation)
                    if let Err(e) = process_date_fields(&db, &collection_name, &mut doc).await {
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

async fn process_date_fields(
    db: &Database, 
    collection_name: &str, 
    doc: &mut Document
) -> Result<(), String> {
    // Get collection schema
    let command = doc! {
        "listCollections": 1,
        "filter": { "name": collection_name }
    };
    
    let response = db.run_command(command, None)
        .await
        .map_err(|e| format!("Failed to get collection info: {}", e))?;
    
    let schema = extract_schema_from_response(response)?;
    
    // Process date fields using the schema
    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        for (field, spec) in properties {
            // Check if the field is a date type in the schema
            if let Some(spec_obj) = spec.as_object() {
                if let Some(bson_type) = spec_obj.get("bsonType").and_then(|b| b.as_str()) {
                    if bson_type == "date" {
                        // Check if the field exists in the document as a string
                        if let Some(mongodb::bson::Bson::String(date_str)) = doc.get(field) {
                            // Try to parse the date
                            if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(date_str) {
                                // Convert to SystemTime then to bson::DateTime
                                let system_time: SystemTime = datetime.into();
                                let mongo_date = mongodb::bson::DateTime::from_system_time(system_time);
                                doc.insert(field, mongo_date);
                            } else {
                                // Try with extended format
                                let extended_date_str = format!("{}:00Z", date_str);
                                if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(&extended_date_str) {
                                    // Convert to SystemTime then to bson::DateTime
                                    let system_time: SystemTime = datetime.into();
                                    let mongo_date = mongodb::bson::DateTime::from_system_time(system_time);
                                    doc.insert(field, mongo_date);
                                } else {
                                    return Err(format!("Failed to parse date field '{}': {}", field, date_str));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

async fn update_document_handler(
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
                    let update_doc = doc! { "$set": update };
                    
                    match collection.update_one(filter, update_doc, None).await {
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

async fn delete_document_handler(
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

// Create the API router
fn create_api_router() -> Router<Arc<Mutex<ApiServerState>>> {
    // Setup CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin(Any);
    
    Router::new()
        .route("/collections", get(list_collections_handler))
        .route("/collections/:collection_name/schema", get(get_collection_schema_handler))
        .route("/collections/:collection_name/documents", get(find_documents_handler))
        .route("/collections/:collection_name/documents", post(insert_document_handler))
        .route("/collections/:collection_name/documents/:id", put(update_document_handler))
        .route("/collections/:collection_name/documents/:id", delete(delete_document_handler))
        .layer(cors)
}

// Start the API server
pub async fn start_server(api_state: Arc<Mutex<ApiServerState>>, port: u16) -> Result<(), String> {
    let app = create_api_router().with_state(api_state.clone());
    
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

#[tauri::command]
pub async fn is_api_server_running(
    api_state: tauri::State<'_, Arc<Mutex<ApiServerState>>>,
) -> Result<bool, ()> {
    let state = api_state.lock().await;
    Ok(state.server_handle.is_some())
}

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

    start_server(api_state.inner().clone(), port).await?;
    Ok(format!("API server started on port {}", port))
}