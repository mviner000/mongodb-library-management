// src/api_server/handlers/csv_import_handler.rs

use axum::{
    extract::{State, Json},
    http::StatusCode,
};
use bson::{doc, Document, Bson};
use mongodb::{bson::oid::ObjectId, options::UpdateOptions};
use rusqlite::{Connection, ToSql};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
use std::collections::HashMap;
use futures::executor::block_on;
use anyhow::Result;

use crate::api_server::state::ApiServerState;
use crate::api_server::models::ApiResponse;
use crate::api_server::services::database_service::get_database;
use crate::api_server::services::schema_service::get_collection_schema_internal;
use crate::api_server::handlers::csv_temp_handlers::get_db_path_from_state;

// Structure for the import summary response
#[derive(serde::Serialize)]
pub struct ImportSummary {
    inserted_count: u64,
    modified_count: u64,
    errors: Vec<String>,
}

// Helper function to convert string values to BSON based on schema type
fn convert_str_to_bson(value: &str, bson_type: &str) -> Result<Bson, String> {
    if value.is_empty() {
        return Ok(Bson::Null);
    }
    
    match bson_type {
        "objectId" => ObjectId::parse_str(value)
            .map(Bson::ObjectId)
            .map_err(|e| format!("Invalid ObjectId: {}", e)),
        "date" => {
            match chrono::DateTime::parse_from_rfc3339(value) {
                Ok(dt) => {
                    let utc_dt = dt.to_utc();
                    let system_time = utc_dt.into();
                    Ok(Bson::DateTime(bson::DateTime::from_system_time(system_time)))
                }
                Err(_) => {
                    match chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
                        Ok(dt) => {
                            let dt_utc = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc);
                            let system_time = dt_utc.into();
                            Ok(Bson::DateTime(bson::DateTime::from_system_time(system_time)))
                        }
                        Err(_) => {
                            chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
                                .map(|d| {
                                    let dt = d.and_hms_opt(0, 0, 0).unwrap();
                                    let dt_utc = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc);
                                    let system_time = dt_utc.into();
                                    Bson::DateTime(bson::DateTime::from_system_time(system_time))
                                })
                                .map_err(|e| format!("Invalid date format: {}", e))
                        }
                    }
                }
            }
        }
        "int" | "int32" => value
            .parse::<i32>()
            .map(Bson::Int32)
            .map_err(|e| format!("Invalid integer: {}", e)),
        "long" | "int64" => value
            .parse::<i64>()
            .map(Bson::Int64)
            .map_err(|e| format!("Invalid long integer: {}", e)),
        "double" | "number" => value
            .parse::<f64>()
            .map(Bson::Double)
            .map_err(|e| format!("Invalid double: {}", e)),
        "bool" | "boolean" => match value.to_lowercase().as_str() {
            "true" | "yes" | "1" | "t" | "y" => Ok(Bson::Boolean(true)),
            "false" | "no" | "0" | "f" | "n" => Ok(Bson::Boolean(false)),
            _ => Err(format!("Invalid boolean value: {}", value)),
        },
        "string" => Ok(Bson::String(value.to_string())),
        "array" => serde_json::from_str::<Vec<serde_json::Value>>(value)
            .map(|arr| bson::to_bson(&arr).unwrap_or(Bson::Null))
            .map_err(|e| format!("Invalid array: {}", e)),
        "object" | "document" => serde_json::from_str::<serde_json::Value>(value)
            .map(|obj| bson::to_bson(&obj).unwrap_or(Bson::Null))
            .map_err(|e| format!("Invalid object: {}", e)),
        _ => Ok(Bson::String(value.to_string())),
    }
}

// Handler for importing validated CSV data
pub async fn import_valid_csv_data_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Json(payload): Json<HashMap<String, Vec<String>>>,
) -> Result<Json<ApiResponse<ImportSummary>>, (StatusCode, String)> {
    let ids = payload.get("ids").cloned().unwrap_or_default();
    let coll_vec = payload.get("collection").cloned().unwrap_or_default();
    if coll_vec.len() != 1 {
        let msg = if coll_vec.is_empty() {
            "Collection name is required"
        } else {
            "Expected single collection name"
        };
        return Err((StatusCode::BAD_REQUEST, msg.to_string()));
    }
    let collection_name = &coll_vec[0];

    let db = get_database(&state.lock().await.mongodb_state).await
        .map_err(|(s, e)| (s, e))?;
    let schema = get_collection_schema_internal(&db, collection_name).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let db_path = get_db_path_from_state(&state, collection_name).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let collection = db.collection::<Document>(collection_name);

    let ids_clone = ids.clone();
    let schema_clone = schema.clone();
    let path_clone = db_path.clone();
    let coll_clone = collection.clone();

    let blocking = task::spawn_blocking(move || -> Result<(u64, u64, Vec<String>), anyhow::Error> {
        let conn = Connection::open(&path_clone)?;
        // Prepare query
        let placeholders = ids_clone.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!("SELECT * FROM valid_data WHERE _id IN ({})", placeholders);
        let mut stmt = conn.prepare(&query)?;

        // Capture column names before querying
        let column_names: Vec<String> = stmt.column_names().iter().map(ToString::to_string).collect();
        let params: Vec<&dyn ToSql> = ids_clone.iter().map(|s| s as &dyn ToSql).collect();
        let mut rows = stmt.query(params.as_slice())?;

        let mut docs = Vec::new();
        let mut errors = Vec::new();
        while let Some(row) = rows.next()? {
            let mut doc = Document::new();
            for (i, col) in column_names.iter().enumerate() {
                if let Ok(Some(val)) = row.get::<_, Option<String>>(i) {
                    // Determine BSON type, default to string
                    let bson_type = match schema_clone.get_document("properties") {
                        Ok(props) => match props.get_document(col) {
                            Ok(field_schema) => match field_schema.get("bsonType") {
                                Some(Bson::String(s)) => s.as_str(),
                                _ => "string",
                            },
                            Err(_) => "string",
                        },
                        Err(_) => "string",
                    };
                    

                    match convert_str_to_bson(&val, bson_type) {
                        Ok(b) => { doc.insert(col, b); },
                        Err(e) => { errors.push(format!("Field {}: {}", col, e)); },
                    }
                }
            }
            if doc.contains_key("_id") {
                docs.push(doc);
            } else {
                errors.push("Document missing _id".to_string());
            }
        }

        // Upsert each document
        let mut inserted = 0;
        let mut modified = 0;
        let upsert_opts = UpdateOptions::builder().upsert(true).build();
        let fut = async {
            for doc in docs {
                let id = doc.get("_id").unwrap().clone();
                let filter = doc! { "_id": id };
                let update = doc! { "$set": doc.clone() };
                match coll_clone.update_one(filter, update, Some(upsert_opts.clone())).await {
                    Ok(res) => {
                        if res.upserted_id.is_some() {
                            inserted += 1;
                        } else {
                            modified += res.modified_count;
                        }
                    },
                    Err(e) => errors.push(format!("update_one error: {}", e)),
                }
            }
            Ok((inserted, modified, errors))
        };
        block_on(fut)
    }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (ins, modif, errs) = blocking.map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Internal error: {}", e)
    ))?;

    let summary = ImportSummary { inserted_count: ins, modified_count: modif, errors: errs };
    Ok(Json(ApiResponse { success: true, data: Some(summary), error: None }))
}
