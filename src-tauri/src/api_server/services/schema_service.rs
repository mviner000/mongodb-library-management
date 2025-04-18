// src/api_server/services/schema_service.rs

use mongodb::{bson::{doc, Document}, Database};
use tracing::error;

// Get collection schema - extract common functionality for reuse
pub async fn get_collection_schema_internal(db: &Database, collection_name: &str) -> Result<Document, String> {
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

// Get collection schema with UI metadata
pub async fn get_collection_schema_with_ui(
    db: &Database, 
    collection_name: &str
) -> Result<Document, String> {
    let schema = get_collection_schema_internal(db, collection_name).await?;
    
    // Fetch UI metadata from ui_metadata collection
    let ui_metadata_collection = db.collection::<Document>("ui_metadata");
    let filter = doc! {
        "collection": collection_name,
        "user_id": { "$exists": false } // Global settings
    };
    
    // Try to get UI metadata
    match ui_metadata_collection.find_one(filter, None).await {
        Ok(ui_metadata) => {
            let mut merged_schema = schema.clone();
            if let Some(ui_metadata_doc) = ui_metadata {
                if let Ok(ui) = ui_metadata_doc.get_document("ui") {
                    merged_schema.insert("ui", ui.clone());
                }
            }
            Ok(merged_schema)
        },
        Err(e) => {
            error!("Failed to fetch UI metadata: {}", e);
            // If we can't get UI metadata, just return the base schema
            Ok(schema)
        }
    }
}

// Update UI metadata for a collection
pub async fn update_ui_metadata(
    db: &Database,
    collection_name: &str,
    ui_update: &Document,
) -> Result<(), String> {
    let now = mongodb::bson::DateTime::now();
    
    // Build the $set document
    let mut set_doc = Document::new();
    for (key, value) in ui_update {
        set_doc.insert(format!("ui.{}", key), value.clone());
    }
    set_doc.insert("updated_at", now);

    let filter = doc! {
        "collection": collection_name,
        "user_id": { "$exists": false } // Global settings
    };

    let options = mongodb::options::UpdateOptions::builder()
        .upsert(true)
        .build();

    db.collection::<Document>("ui_metadata")
        .update_one(filter, doc! { "$set": set_doc }, options)
        .await
        .map_err(|e| format!("Failed to update UI metadata: {}", e))?;
    
    Ok(())
}
