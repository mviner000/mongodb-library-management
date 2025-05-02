// src/api_server/services/database_service.rs

use axum::http::StatusCode;
use mongodb::{bson::{Document, Bson}, Database};
use std::{sync::Arc, time::SystemTime};
use tokio::sync::Mutex;

// Get database connection
pub async fn get_database(mongodb_state: &Arc<Mutex<MongoDbState>>) -> Result<Database, (StatusCode, String)> {
    let state = mongodb_state.lock().await;
    match state.get_database().await {
        Ok(db) => Ok(db),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

// Process all types of fields according to schema
pub async fn process_document_fields(
    db: &Database, 
    collection_name: &str, 
    doc: &mut Document
) -> Result<(), String> {
    // Get collection schema
    let schema = crate::api_server::services::schema_service::get_collection_schema_internal(db, collection_name).await?;
    
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
pub fn process_date_field(doc: &mut Document, field: &str) -> Result<(), String> {
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
pub fn process_int_field(doc: &mut Document, field: &str) -> Result<(), String> {
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
pub fn process_double_field(doc: &mut Document, field: &str) -> Result<(), String> {
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

// Re-export from MongoDbState for interface compatibility
pub use crate::mongodb_manager::MongoDbState;