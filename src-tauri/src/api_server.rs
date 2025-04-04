// src/api_server.rs

use axum::{
    routing::{get, post, put, delete},
    http::{StatusCode, Method},
    Json, Router, extract::Path, extract::State, response::IntoResponse,
};

use axum::extract::Query;

use mongodb::{bson::{doc, Document, oid::ObjectId, Bson}, Database};
use serde::{Deserialize, Serialize};
use serde_json;
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
    Query(params): Query<std::collections::HashMap<String, String>>,
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
                    // Remove any client-provided timestamp fields
                    doc.remove("created_at");
                    doc.remove("updated_at");
                    
                    // Add server-managed timestamp fields
                    let current_time = mongodb::bson::DateTime::now();
                    doc.insert("created_at", current_time.clone());
                    doc.insert("updated_at", current_time);
                    
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

// Get collection schema - extract common functionality for reuse
async fn get_collection_schema_internal(db: &Database, collection_name: &str) -> Result<Document, String> {
    let command = doc! {
        "listCollections": 1,
        "filter": { "name": collection_name }
    };
    
    let response = db.run_command(command, None)
        .await
        .map_err(|e| format!("Failed to get collection info: {}", e))?;
    
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
    
    Ok(json_schema.clone())
}

// Process all types of fields according to schema
async fn process_document_fields(
    db: &Database, 
    collection_name: &str, 
    doc: &mut Document
) -> Result<(), String> {
    // Get collection schema
    let schema = get_collection_schema_internal(db, collection_name).await?;
    
    if let Some(properties) = schema.get("properties").and_then(|p| p.as_document()) {
        for (field, spec) in properties.iter() {
            // Skip if field doesn't exist in document
            if !doc.contains_key(field) {
                continue;
            }
            
            if let Some(spec_doc) = spec.as_document() {
                if let Some(bson_type) = spec_doc.get("bsonType") {
                    let bson_type_str = bson_type.as_str().unwrap_or("");
                    
                    match bson_type_str {
                        // Handle date fields
                        "date" => {
                            process_date_field(doc, field)?;
                        },
                        // Handle integer fields
                        "int" => {
                            process_int_field(doc, field)?;
                        },
                        // Handle double/decimal fields
                        "double" => {
                            process_double_field(doc, field)?;
                        },
                        // Add other types as needed
                        _ => {}
                    }
                }
            }
        }
    }
    
    Ok(())
}

// Process date fields
fn process_date_field(doc: &mut Document, field: &str) -> Result<(), String> {
    if let Some(Bson::String(date_str)) = doc.get(field) {
        // Try to parse the date using different formats
        let datetime = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
            dt
        } else {
            // Try with extended format
            let extended_date_str = format!("{}:00Z", date_str);
            chrono::DateTime::parse_from_rfc3339(&extended_date_str)
                .map_err(|e| format!("Failed to parse date field '{}': {} - Error: {}", field, date_str, e))?
        };
        
        // Convert to SystemTime then to bson::DateTime
        let system_time: SystemTime = datetime.into();
        let mongo_date = mongodb::bson::DateTime::from_system_time(system_time);
        doc.insert(field, mongo_date);
    }
    
    Ok(())
}

// Process integer fields
fn process_int_field(doc: &mut Document, field: &str) -> Result<(), String> {
    match doc.get(field) {
        Some(Bson::String(int_str)) => {
            // Convert string to integer
            let int_value = int_str.parse::<i32>()
                .map_err(|e| format!("Failed to parse integer field '{}': {} - Error: {}", field, int_str, e))?;
            doc.insert(field, Bson::Int32(int_value));
        },
        Some(Bson::Double(double_val)) => {
            // Convert double to integer
            let int_value = *double_val as i32;
            doc.insert(field, Bson::Int32(int_value));
        },
        _ => {}
    }
    
    Ok(())
}

// Process double/decimal fields
fn process_double_field(doc: &mut Document, field: &str) -> Result<(), String> {
    match doc.get(field) {
        Some(Bson::String(double_str)) => {
            // Convert string to double
            let double_value = double_str.parse::<f64>()
                .map_err(|e| format!("Failed to parse double field '{}': {} - Error: {}", field, double_str, e))?;
            doc.insert(field, Bson::Double(double_value));
        },
        Some(Bson::Int32(int_val)) => {
            // Convert integer to double
            let double_value = *int_val as f64;
            doc.insert(field, Bson::Double(double_value));
        },
        Some(Bson::Int64(int_val)) => {
            // Convert integer to double
            let double_value = *int_val as f64;
            doc.insert(field, Bson::Double(double_value));
        },
        _ => {}
    }
    
    Ok(())
}

// Legacy function for backward compatibility (now calls the more comprehensive process_document_fields)
async fn process_date_fields(
    db: &Database, 
    collection_name: &str, 
    doc: &mut Document
) -> Result<(), String> {
    process_document_fields(db, collection_name, doc).await
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
                    
                    // Process fields in the update document according to the schema
                    let mut update_doc = update.clone();
                    
                    // Remove any attempts to modify timestamp fields
                    update_doc.remove("created_at");
                    
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

#[tauri::command]
pub async fn list_api_routes(
    api_state: tauri::State<'_, Arc<Mutex<ApiServerState>>>,
) -> Result<Vec<String>, String> {
    let _state = api_state.lock().await;
    Ok(vec![
        "GET /collections".to_string(),
        "GET /collections/:collection_name/schema".to_string(),
        "GET /collections/:collection_name/documents".to_string(),
        "POST /collections/:collection_name/documents".to_string(),
        "PUT /collections/:collection_name/documents/:id".to_string(),
        "DELETE /collections/:collection_name/documents/:id".to_string(),
    ])
}