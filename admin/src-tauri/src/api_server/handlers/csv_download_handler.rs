// src/api_server/handlers/csv_download_handler.rs

use axum::{
    extract::{Path, State, Query, Json},
    http::{StatusCode, HeaderMap, header::{CONTENT_TYPE, CONTENT_DISPOSITION}},
    response::IntoResponse,
};
use rusqlite::{Connection, types::ValueRef};
use std::{sync::Arc, path::PathBuf};
use tokio::sync::Mutex;
use csv::Writer;
use serde::{Deserialize, Serialize};
use crate::api_server::state::ApiServerState;

// Make the struct public and derive Serialize
#[derive(Debug, Deserialize, Serialize)]
pub struct DownloadParams {
    pub ids: Option<String>,
}

// Request body for POST endpoint
#[derive(Debug, Deserialize)]
pub struct DownloadJsonRequest {
    pub ids: Vec<String>,
}

// GET endpoint with query parameters
pub async fn download_temp_csv(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection): Path<String>,
    Query(params): Query<DownloadParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Debug statement to show when this handler is called
    println!("download_temp_csv GET handler called for collection: {}", collection);
    
    // Get database path from state
    let db_path = super::csv_temp_handlers::get_db_path_from_state(&state, &collection)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Split comma-separated IDs into Vec<String>
    let ids = params.ids
        .map(|s| s.split(',').map(|s| s.to_string()).collect::<Vec<String>>())
        .unwrap_or_default();

    // Process the CSV generation with the IDs
    generate_csv(db_path, ids).await
}

// POST endpoint with JSON body
pub async fn download_temp_csv_post(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection): Path<String>,
    Json(payload): Json<DownloadJsonRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Debug statement to show when this handler is called
    println!("download_temp_csv POST handler called for collection: {} with {} IDs", 
             collection, payload.ids.len());
    
    // Get database path from state
    let db_path = super::csv_temp_handlers::get_db_path_from_state(&state, &collection)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Process the CSV generation with the IDs from the JSON body
    generate_csv(db_path, payload.ids).await
}

// Refactored common CSV generation logic - updated to accept PathBuf
async fn generate_csv(db_path: PathBuf, ids: Vec<String>) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Execute in blocking task
    let task_result = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, String> {
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open SQLite database: {}", e))?;
        
        // Get column names in order
        let mut stmt = conn.prepare("PRAGMA table_info(valid_data)")
            .map_err(|e| format!("PRAGMA failed: {}", e))?;
        
        let columns: Vec<String> = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(1)?)
        })
        .map_err(|e| format!("Column mapping failed: {}", e))?
        .collect::<Result<_, _>>()
        .map_err(|e| format!("Column collection failed: {}", e))?;
        
        // Build query with optional ID filtering
        let mut query = "SELECT * FROM valid_data".to_string();
        if !ids.is_empty() {
            query.push_str(&format!(" WHERE _id IN ({})", ids.iter().map(|_| "?").collect::<Vec<_>>().join(",")));
        }

        let mut stmt = conn.prepare(&query)
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        // Use rusqlite's params_from_iter for safe parameter binding
        let mut rows = if ids.is_empty() {
            stmt.query([])
        } else {
            stmt.query(rusqlite::params_from_iter(ids.iter()))
        }.map_err(|e| format!("Query failed: {}", e))?;

        let mut wtr = Writer::from_writer(vec![]);
        
        // Write headers using actual column names
        wtr.write_record(&columns)
            .map_err(|e| format!("CSV header error: {}", e))?;
        
        // Write data rows
        loop {
            match rows.next() {
                Ok(Some(row)) => {
                    let mut record = Vec::with_capacity(columns.len());
                    for (i, _) in columns.iter().enumerate() {
                        let value = match row.get_ref(i) {
                            Ok(ValueRef::Null) => "".to_string(),
                            Ok(ValueRef::Integer(i)) => i.to_string(),
                            Ok(ValueRef::Real(f)) => f.to_string(),
                            Ok(ValueRef::Text(t)) => String::from_utf8_lossy(t).to_string(),
                            Ok(ValueRef::Blob(b)) => format!("[blob:{}bytes]", b.len()),
                            Err(_) => "".to_string(),
                        };
                        record.push(value);
                    }
                    
                    wtr.write_record(&record)
                        .map_err(|e| format!("CSV write error: {}", e))?;
                },
                Ok(None) => break,
                Err(e) => return Err(format!("Row iteration error: {}", e)),
            }
        }

        wtr.flush()
            .map_err(|e| format!("CSV flush error: {}", e))?;
            
        Ok(wtr.into_inner()
            .map_err(|e| format!("CSV finalization error: {}", e))?)
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task failed: {}", e)))?;

    // Handle the Result from the task
    let csv_data = task_result.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Create response with CSV headers
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        "text/csv; charset=utf-8".parse().unwrap()
    );
    headers.insert(
        CONTENT_DISPOSITION,
        format!("attachment; filename=\"export_{}.csv\"", 
                chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")).parse().unwrap()
    );

    Ok((headers, csv_data))
}