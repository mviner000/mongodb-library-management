// src/mongodb_manager.rs

use crate::mongodb_schema;

use mongodb::{Client, Database, options::ClientOptions};
use mongodb::bson::{doc, Document};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;
use anyhow::Result;
use futures_util::stream::StreamExt;
use tracing::{info, error};

// Define MongoDB connection state
#[derive(Debug)]
pub struct MongoDbState {
    pub client: Arc<Mutex<Option<Client>>>, // Add pub modifier here
    pub database_name: String,              // And here if needed
}

impl MongoDbState {
    pub fn new(database_name: &str) -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            database_name: database_name.to_string(),
        }
    }

    pub async fn get_database(&self) -> Result<Database, String> {
        let client_guard = self.client.lock().await;
        
        if client_guard.is_none() {
            return Err("Database connection not initialized. Call connect() first.".into());
        }
        
        let client = client_guard.as_ref().unwrap();
        Ok(client.database(&self.database_name))
    }
}

// Make MongoDbState cloneable
impl Clone for MongoDbState {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            database_name: self.database_name.clone(),
        }
    }
}

#[tauri::command]
pub async fn connect_mongodb(
    mongodb_state: State<'_, MongoDbState>,
    connection_string: String,
) -> Result<(), String> {
    let mut client_guard = mongodb_state.client.lock().await;
    
    if client_guard.is_some() {
        // Already connected
        return Ok(());
    }
    
    // Parse connection string and create client options
    let client_options = ClientOptions::parse(&connection_string)
        .await
        .map_err(|e| format!("Failed to parse connection string: {}", e))?;
    
    // Create a new client
    let client = Client::with_options(client_options)
        .map_err(|e| format!("Failed to create MongoDB client: {}", e))?;
    
    // Test the connection by pinging the server
    client
        .database("admin")
        .run_command(mongodb::bson::doc! { "ping": 1 }, None)
        .await
        .map_err(|e| format!("Failed to connect to MongoDB: {}", e))?;
    
    // Store the client
    *client_guard = Some(client);
    
    Ok(())
}

#[tauri::command]
pub async fn disconnect_mongodb(mongodb_state: State<'_, MongoDbState>) -> Result<(), String> {
    let mut client_guard = mongodb_state.client.lock().await;
    *client_guard = None;
    Ok(())
}

#[tauri::command]
pub async fn get_collection_schema(
    mongodb_state: State<'_, MongoDbState>,
    collection_name: String,
) -> Result<serde_json::Value, String> {
    println!("DEBUG: get_collection_schema called for collection '{}'", collection_name);
    
    let db = mongodb_state.get_database().await?;
    println!("DEBUG: Successfully got database reference");
    
    let command = doc! {
        "listCollections": 1,
        "filter": { "name": &collection_name }
    };
    
    println!("DEBUG: Executing listCollections command with filter for '{}'", collection_name);
    let response = db.run_command(command, None)
        .await
        .map_err(|e| {
            println!("DEBUG: Error running command: {}", e);
            format!("Failed to get collection info: {}", e)
        })?;
    
    println!("DEBUG: Got response from listCollections");
    let cursor = response.get_document("cursor")
        .map_err(|e| {
            println!("DEBUG: Error getting cursor from response: {:?}", e);
            "Invalid response format"
        })?;
    
    let batches = cursor.get_array("firstBatch")
        .map_err(|e| {
            println!("DEBUG: Error getting firstBatch from cursor: {:?}", e);
            "No collections found"
        })?;
    
    if batches.is_empty() {
        println!("DEBUG: No collections found matching the name '{}'", collection_name);
        return Err("Collection not found".into());
    }
    
    println!("DEBUG: Found {} collections", batches.len());
    let coll_info = batches[0].as_document().ok_or_else(|| {
        println!("DEBUG: First item in batches is not a document");
        "Invalid collection info"
    })?;
    
    println!("DEBUG: Getting options from collection info");
    let options = coll_info.get_document("options")
        .map_err(|e| {
            println!("DEBUG: No options found in collection info: {:?}", e);
            "No options found"
        })?;
    
    println!("DEBUG: Getting validator from options");
    let validator = options.get_document("validator")
        .map_err(|e| {
            println!("DEBUG: No validator found in options: {:?}", e);
            "No validator found"
        })?;
    
    println!("DEBUG: Getting $jsonSchema from validator");
    let json_schema = validator.get_document("$jsonSchema")
        .map_err(|e| {
            println!("DEBUG: No JSON schema found in validator: {:?}", e);
            "No JSON schema found"
        })?;
    
    println!("DEBUG: Converting BSON to JSON");
    let schema: serde_json::Value = mongodb::bson::from_bson(json_schema.clone().into())
        .map_err(|e| {
            println!("DEBUG: BSON to JSON conversion failed: {}", e);
            format!("BSON to JSON conversion failed: {}", e)
        })?;
    
    println!("DEBUG: Successfully retrieved schema for collection '{}'", collection_name);
    Ok(schema)
}

#[tauri::command]
pub async fn initialize_library_collections(
    state: tauri::State<'_, MongoDbState>,
) -> Result<(), String> {
    info!("Initializing library collections via Tauri command");
    let db = state.get_database().await?;
    
    match crate::mongodb_schema::initialize_library_collections(&db).await {
        Ok(_) => {
            info!("Successfully initialized library collections via Tauri command");
            Ok(())
        },
        Err(e) => {
            error!("Failed to initialize library collections: {}", e);
            Err(e.to_string())  // Convert anyhow::Error to String
        }
    }
}

// Insert document function (not generic)
#[tauri::command]
pub async fn insert_document(
    mongodb_state: State<'_, MongoDbState>,
    collection_name: String,
    document: serde_json::Value,
) -> Result<String, String> {
    let db = mongodb_state.get_database().await?;
    let collection = db.collection::<Document>(&collection_name);
    
    // Convert JSON to BSON document
    let mut doc = mongodb::bson::to_document(&document)
        .map_err(|e| format!("Failed to convert document to BSON: {}", e))?;
    
    // Retrieve collection schema
    let schema = get_collection_schema_internal(&db, &collection_name).await?;
    
    // Process date fields
    if let Some(properties) = schema.get("properties").and_then(|p| p.as_document()) {
        for (field, spec) in properties {
            // Check if the field is a date type in the schema
            if let Some(spec_doc) = spec.as_document() {
                if let Some("date") = spec_doc.get("bsonType").and_then(|b| b.as_str()) {
                    // Check if the field exists in the document as a string
                    if let Some(bson::Bson::String(date_str)) = doc.get(field) {
                        // Parse the string date to a DateTime
                        // This assumes ISO format like "2025-03-31T17:18"
                        if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(date_str) {
                            // Convert to MongoDB DateTime and update the document
                            let mongo_date = bson::DateTime::from_millis(datetime.timestamp_millis());
                            doc.insert(field, mongo_date);
                        } else {
                            // If parsing fails, try a more flexible approach
                            // This handles formats like "2025-03-31T17:18" without seconds
                            let extended_date_str = format!("{}:00Z", date_str);
                            if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(&extended_date_str) {
                                let mongo_date = bson::DateTime::from_millis(datetime.timestamp_millis());
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
    
    let result = collection.insert_one(doc, None)
        .await
        .map_err(|e| format!("Failed to insert document: {}", e))?;
    
    match result.inserted_id.as_object_id() {
        Some(id) => Ok(id.to_hex()),
        None => Err("Failed to get inserted document ID".into()),
    }
}


async fn get_collection_schema_internal(
    db: &Database,
    collection_name: &str,
) -> Result<Document, String> {
    println!("DEBUG: get_collection_schema_internal called for collection '{}'", collection_name);
    
    let command = doc! {
        "listCollections": 1,
        "filter": { "name": collection_name }
    };
    
    println!("DEBUG: Executing listCollections command with filter for '{}'", collection_name);
    let response = db.run_command(command, None)
        .await
        .map_err(|e| {
            println!("DEBUG: Error running command: {}", e);
            format!("Failed to get collection info: {}", e)
        })?;
    
    println!("DEBUG: Got response from listCollections");
    let cursor = response.get_document("cursor")
        .map_err(|e| {
            println!("DEBUG: Error getting cursor from response: {:?}", e);
            "Invalid response format".to_string()
        })?;
    
    let batches = cursor.get_array("firstBatch")
        .map_err(|e| {
            println!("DEBUG: Error getting firstBatch from cursor: {:?}", e);
            "No collections found".to_string()
        })?;
    
    if batches.is_empty() {
        println!("DEBUG: No collections found matching the name '{}'", collection_name);
        return Err("Collection not found".into());
    }
    
    println!("DEBUG: Found {} collections", batches.len());
    let coll_info = batches[0].as_document().ok_or_else(|| {
        println!("DEBUG: First item in batches is not a document");
        "Invalid collection info".to_string()
    })?;
    
    println!("DEBUG: Getting options from collection info");
    let options = coll_info.get_document("options")
        .map_err(|e| {
            println!("DEBUG: No options found in collection info: {:?}", e);
            "No options found".to_string()
        })?;
    
    println!("DEBUG: Getting validator from options");
    let validator = options.get_document("validator")
        .map_err(|e| {
            println!("DEBUG: No validator found in options: {:?}", e);
            "No validator found".to_string()
        })?;
    
    println!("DEBUG: Getting $jsonSchema from validator");
    let json_schema = validator.get_document("$jsonSchema")
        .map_err(|e| {
            println!("DEBUG: No JSON schema found in validator: {:?}", e);
            "No JSON schema found".to_string()
        })?;
    
    println!("DEBUG: Successfully retrieved schema for collection '{}'", collection_name);
    Ok(json_schema.clone())
}

// Find documents function (not generic)
#[tauri::command]
pub async fn find_documents(
    mongodb_state: State<'_, MongoDbState>,
    collection_name: String,
    filter: Document,
) -> Result<Vec<Document>, String> {
    let db = mongodb_state.get_database().await?;
    let collection = db.collection::<Document>(&collection_name);
    
    let mut cursor = collection.find(filter, None)
        .await
        .map_err(|e| format!("Failed to find documents: {}", e))?;
    
    let mut documents = Vec::new();
    while let Some(document_result) = cursor.next().await {
        match document_result {
            Ok(mut doc) => {
                // Convert BSON date fields to readable string format
                format_date_fields(&mut doc);
                documents.push(doc);
            },
            Err(e) => return Err(format!("Error retrieving document: {}", e)),
        }
    }
    
    Ok(documents)
}

// Helper function to format date fields in a document
fn format_date_fields(doc: &mut Document) {
    // Get a list of keys to avoid borrowing issues
    let keys: Vec<String> = doc.keys().cloned().collect();
    
    for key in keys {
        if let Some(bson::Bson::DateTime(date_time)) = doc.get(&key) {
            // Convert MongoDB DateTime to chrono::DateTime
            let chrono_date = chrono::DateTime::from_timestamp_millis(date_time.timestamp_millis())
                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
            
            // Format the date as a readable string (ISO 8601 format)
            let formatted_date = chrono_date.format("%Y-%m-%d %H:%M:%S").to_string();
            
            // Replace the BSON DateTime with the formatted string
            doc.insert(key, bson::Bson::String(formatted_date));
        }
    }
}

// Update document by ID
#[tauri::command]
pub async fn update_document(
    mongodb_state: State<'_, MongoDbState>,
    collection_name: String,
    id: String,
    update: Document, // Use concrete Document type
) -> Result<bool, String> {
    let db = mongodb_state.get_database().await?;
    let collection = db.collection::<Document>(&collection_name);
    
    let object_id = mongodb::bson::oid::ObjectId::parse_str(&id)
        .map_err(|e| format!("Invalid ObjectId: {}", e))?;
    
    let filter = mongodb::bson::doc! { "_id": object_id };
    let update_doc = mongodb::bson::doc! { "$set": update };
    
    let result = collection.update_one(filter, update_doc, None)
        .await
        .map_err(|e| format!("Failed to update document: {}", e))?;
    
    Ok(result.modified_count > 0)
}

// Delete document by ID
#[tauri::command]
pub async fn delete_document(
    mongodb_state: State<'_, MongoDbState>,
    collection_name: String,
    id: String,
) -> Result<bool, String> {
    let db = mongodb_state.get_database().await?;
    let collection = db.collection::<Document>(&collection_name);
    
    let object_id = mongodb::bson::oid::ObjectId::parse_str(&id)
        .map_err(|e| format!("Invalid ObjectId: {}", e))?;
    
    let filter = mongodb::bson::doc! { "_id": object_id };
    
    let result = collection.delete_one(filter, None)
        .await
        .map_err(|e| format!("Failed to delete document: {}", e))?;
    
    Ok(result.deleted_count > 0)
}

pub async fn auto_connect(mongodb_state: &MongoDbState) -> Result<(), String> {
    let connection_string = "mongodb://localhost:27017";
    let mut client_guard = mongodb_state.client.lock().await;
    
    if client_guard.is_some() {
        return Ok(());
    }
    
    let client_options = ClientOptions::parse(connection_string)
        .await
        .map_err(|e| format!("Failed to parse connection string: {}", e))?;
    
    let client = Client::with_options(client_options)
        .map_err(|e| format!("Failed to create MongoDB client: {}", e))?;
    
    // Verify connection
    client
        .database("admin")
        .run_command(bson::doc! { "ping": 1 }, None)
        .await
        .map_err(|e| format!("Failed to connect to MongoDB: {}", e))?;

    // Initialize database schema
    let db = client.database(&mongodb_state.database_name);
    mongodb_schema::initialize_database(&db)
        .await
        .map_err(|e| format!("Failed to initialize database: {}", e))?;

    *client_guard = Some(client);
    Ok(())
}


#[tauri::command]
pub async fn list_collections(
    mongodb_state: State<'_, MongoDbState>
) -> Result<Vec<String>, String> {
    let db = mongodb_state.get_database().await?;
    let filter = Some(bson::doc! {}); // Include all collections
    let collections = db.list_collection_names(filter)
        .await
        .map_err(|e| format!("Failed to list collections: {}", e))?;
    Ok(collections)
}