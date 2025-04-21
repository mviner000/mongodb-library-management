// src/api_server/handlers/csv_temp_handlers.rs
use axum::{extract::{Path, State}, http::StatusCode, Json};
use rusqlite::{Connection, params};
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    sync::Arc,
};
use tokio::{sync::Mutex, task};
use crate::api_server::state::ApiServerState;
use std::env;
use anyhow::{Result as AnyhowResult, anyhow};

// Helper function to get app data directory
fn get_app_data_dir() -> PathBuf {
    let app_name = "vue-tauri";
    
    #[cfg(target_os = "windows")]
    {
        let app_data = env::var("APPDATA").unwrap_or_else(|_| {
            env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string()) + "\\AppData\\Roaming"
        });
        PathBuf::from(app_data).join(app_name)
    }
    
    #[cfg(target_os = "macos")]
    {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join("Library").join("Application Support").join(app_name)
    }
    
    #[cfg(target_os = "linux")]
    {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".local").join("share").join(app_name)
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        PathBuf::from(".").join(app_name)
    }
}

// Get the database path for a collection
async fn get_db_path_from_state(state: &Arc<Mutex<ApiServerState>>, collection: &str) -> AnyhowResult<PathBuf> {
    let state = state.lock().await;
    let temp_dirs = state.temp_dirs.lock().await;
    if let Some(path) = temp_dirs.get(collection) {
        Ok(path.clone())
    } else {
        // Create a path in the app data directory if it doesn't exist
        let app_data_dir = task::spawn_blocking(get_app_data_dir)
            .await
            .map_err(|e| anyhow!("Failed to spawn blocking task: {}", e))?;
            
        let temp_dir = app_data_dir.join("temp").join(collection);
        
        let temp_dir_clone = temp_dir.clone();
        task::spawn_blocking(move || {
            fs::create_dir_all(&temp_dir_clone)
                .map_err(|e| anyhow!("Failed to create temp directory: {}", e))
        })
        .await
        .map_err(|e| anyhow!("Task join error: {}", e))??;
        
        Ok(temp_dir.join("temp_data.sqlite"))
    }
}

// Add path to state
async fn add_path_to_state(state: &Arc<Mutex<ApiServerState>>, collection: String, path: PathBuf) -> AnyhowResult<()> {
    let state = state.lock().await;
    let mut temp_dirs = state.temp_dirs.lock().await;
    temp_dirs.insert(collection, path);
    Ok(())
}

// Remove path from state
async fn remove_path_from_state(state: &Arc<Mutex<ApiServerState>>, collection: &str) -> AnyhowResult<Option<PathBuf>> {
    let state = state.lock().await;
    let mut temp_dirs = state.temp_dirs.lock().await;
    Ok(temp_dirs.remove(collection))
}

pub async fn save_csv_temp(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection): Path<String>,
    Json(data): Json<Vec<Value>>,
) -> Result<(), (StatusCode, String)> {
    // Get app data directory in a non-blocking way
    let app_data_dir = task::spawn_blocking(get_app_data_dir)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to spawn blocking task: {}", e)))?;
    
    let temp_dir = app_data_dir.join("temp").join(&collection);
    let db_path = temp_dir.join("temp_data.sqlite");
    
    // Create directories using blocking task
    let temp_dir_clone = temp_dir.clone();
    task::spawn_blocking(move || {
        fs::create_dir_all(&temp_dir_clone)
            .map_err(|e| anyhow!("Failed to create directory: {}", e))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Store path in state
    add_path_to_state(&state, collection.clone(), db_path.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Clone data and path for move into blocking task
    let data_clone = data.clone();
    let db_path_clone = db_path.clone();
    
    // Execute SQLite operations in blocking task
    task::spawn_blocking(move || -> AnyhowResult<()> {
        let conn = Connection::open(&db_path_clone)
            .map_err(|e| anyhow!("Failed to open database: {}", e))?;
        
        // Drop existing table if it exists
        conn.execute("DROP TABLE IF EXISTS temp_data", [])
            .map_err(|e| anyhow!("Failed to drop table: {}", e))?;
        
        conn.execute(
            "CREATE TABLE temp_data (id INTEGER PRIMARY KEY, data TEXT)",
            [],
        ).map_err(|e| anyhow!("Failed to create table: {}", e))?;

        for item in data_clone {
            let json_str = serde_json::to_string(&item)
                .map_err(|e| anyhow!("Failed to serialize JSON: {}", e))?;
            
            conn.execute(
                "INSERT INTO temp_data (data) VALUES (?)",
                params![json_str],
            ).map_err(|e| anyhow!("Failed to insert data: {}", e))?;
        }

        Ok(())
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(())
}

pub async fn load_csv_temp(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection): Path<String>,
) -> Result<Json<Vec<Value>>, (StatusCode, String)> {
    // Get the database path from state
    let db_path = get_db_path_from_state(&state, &collection)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Check if the database file exists
    let db_path_clone = db_path.clone();
    let file_exists = task::spawn_blocking(move || db_path_clone.exists())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))?;
    
    if !file_exists {
        return Err((StatusCode::NOT_FOUND, format!("No temporary data found for collection: {}", collection)));
    }
    
    // Execute SQLite operations in blocking task
    let results = task::spawn_blocking(move || -> AnyhowResult<Vec<Value>> {
        let conn = Connection::open(&db_path)
            .map_err(|e| anyhow!("Failed to open database: {}", e))?;
        
        let mut stmt = conn.prepare("SELECT data FROM temp_data")
            .map_err(|e| anyhow!("Failed to prepare statement: {}", e))?;
        
        let rows = stmt.query_map([], |row| {
            let data: String = row.get(0)?;
            let value: Value = serde_json::from_str(&data)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            Ok(value)
        }).map_err(|e| anyhow!("Query failed: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| anyhow!("Row error: {}", e))?);
        }

        Ok(results)
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(results))
}

pub async fn delete_csv_temp(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection): Path<String>,
) -> Result<(), (StatusCode, String)> {
    // Remove path from state
    let path_opt = remove_path_from_state(&state, &collection)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    if let Some(path) = path_opt {
        // Determine if path is a file or directory
        let db_file = task::spawn_blocking(move || {
            if path.is_file() {
                path.clone()
            } else {
                path.join("temp_data.sqlite")
            }
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))?;
        
        // Check if file exists and remove it
        let db_file_clone = db_file.clone();
        let file_exists = task::spawn_blocking(move || db_file_clone.exists())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))?;
        
        if file_exists {
            // Remove the file
            let db_file_clone = db_file.clone();
            task::spawn_blocking(move || fs::remove_file(&db_file_clone))
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))?
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to remove file: {}", e)))?;
        }
        
        // Try to remove the directory too
        let dir = db_file.parent().unwrap_or(&db_file).to_path_buf();
        let _ = task::spawn_blocking(move || fs::remove_dir(&dir)).await; // Optional cleanup, ignore errors
    }
    
    Ok(())
}