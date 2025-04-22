// src/api_server/handlers/csv_temp_handlers.rs
use axum::{extract::{Path, State}, http::StatusCode, Json};
use rusqlite::{Connection, params, named_params};
use serde_json::{Value, Map};
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

// Helper function to convert value to a rusqlite-compatible type
fn value_to_param(value: &Value) -> rusqlite::Result<String> {
    match value {
        Value::Null => Ok("".to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Number(n) => Ok(n.to_string()),
        Value::String(s) => Ok(s.clone()),
        Value::Array(_) | Value::Object(_) => Ok(value.to_string()),
    }
}

pub async fn save_csv_temp(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection): Path<String>,
    Json(data): Json<Vec<Value>>,
) -> Result<(), (StatusCode, String)> {
    if data.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No data provided".to_string()));
    }

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
        
        // Extract headers from the first item
        let first_item = match &data_clone[0] {
            Value::Object(obj) => obj,
            _ => return Err(anyhow!("Data items must be objects"))
        };
        
        // Get headers and exclude 'id' field if present as we'll use SQLite's auto-increment
        let mut headers: Vec<String> = first_item.keys()
            .filter(|&k| k != "id")
            .map(|k| k.to_string())
            .collect();
        
        // Build CREATE TABLE SQL
        let mut create_sql = "CREATE TABLE temp_data (id INTEGER PRIMARY KEY".to_string();
        for header in &headers {
            // Escape column names to handle special characters
            create_sql.push_str(&format!(", \"{}\" TEXT", header.replace("\"", "\"\"")));
        }
        create_sql.push_str(")");
        
        conn.execute(&create_sql, [])
            .map_err(|e| anyhow!("Failed to create table: {}", e))?;

        // Build placeholders for prepared statement
        let placeholders: Vec<String> = (0..headers.len())
            .map(|_| "?".to_string())
            .collect();
            
        // Build columns for INSERT statement
        let columns = headers.iter()
            .map(|h| format!("\"{}\"", h.replace("\"", "\"\"")))
            .collect::<Vec<_>>()
            .join(", ");
            
        // Create INSERT SQL statement
        let insert_sql = format!(
            "INSERT INTO temp_data ({}) VALUES ({})",
            columns,
            placeholders.join(", ")
        );

        // Prepare statement once for multiple executions
        let mut stmt = conn.prepare(&insert_sql)
            .map_err(|e| anyhow!("Failed to prepare statement: {}", e))?;

        // Insert each row
        for item in data_clone {
            let obj = match item {
                Value::Object(obj) => obj,
                _ => continue // Skip non-object items
            };
            
            // Extract values in the same order as headers
            let values: Vec<String> = headers.iter()
                .map(|h| {
                    match obj.get(h) {
                        Some(val) => value_to_param(val)
                            .unwrap_or_else(|_| "".to_string()),
                        None => "".to_string()
                    }
                })
                .collect();
                
            // Convert values to params for the prepared statement
            let params_slice: Vec<&dyn rusqlite::ToSql> = values
                .iter()
                .map(|v| v as &dyn rusqlite::ToSql)
                .collect();
                
            stmt.execute(params_slice.as_slice())
                .map_err(|e| anyhow!("Failed to insert data: {}", e))?;
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
        
        // First get the column names
        let mut stmt = conn.prepare("PRAGMA table_info(temp_data)")
            .map_err(|e| anyhow!("Failed to get table info: {}", e))?;
            
        let columns: Vec<String> = stmt.query_map([], |row| {
            let column_name: String = row.get(1)?;
            Ok(column_name)
        })
        .map_err(|e| anyhow!("Failed to query columns: {}", e))?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| anyhow!("Column error: {}", e))?;
        
        // Prepare and execute SELECT statement for all rows
        let mut stmt = conn.prepare("SELECT * FROM temp_data")
            .map_err(|e| anyhow!("Failed to prepare statement: {}", e))?;
            
        let rows = stmt.query_map([], |row| {
            let mut obj = Map::new();
            
            // Include SQLite rowid as 'id'
            let id: i64 = row.get(0)?;
            obj.insert("id".to_string(), Value::Number(id.into()));
            
            // Add all other columns
            for (i, column_name) in columns.iter().enumerate() {
                if i == 0 { continue; } // Skip id column which we already handled
                
                let val: Option<String> = row.get(i)?;
                let json_val = match val {
                    Some(s) => {
                        // Try to parse as JSON if it looks like JSON
                        if (s.starts_with('{') && s.ends_with('}')) || 
                           (s.starts_with('[') && s.ends_with(']')) ||
                           (s == "true" || s == "false" || s == "null") ||
                           s.parse::<f64>().is_ok() {
                            serde_json::from_str(&s).unwrap_or(Value::String(s))
                        } else {
                            Value::String(s)
                        }
                    },
                    None => Value::Null
                };
                
                obj.insert(column_name.clone(), json_val);
            }
            
            Ok(Value::Object(obj))
        })
        .map_err(|e| anyhow!("Query failed: {}", e))?;

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
    // Get app data directory to find the collection directory
    let app_data_dir = task::spawn_blocking(get_app_data_dir)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to spawn blocking task: {}", e)))?;
    
    // The collection directory is in "temp/{collection}"
    let collection_dir = app_data_dir.join("temp").join(&collection);
    
    // Remove path from state
    let _ = remove_path_from_state(&state, &collection)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    // Check if directory exists and remove it recursively
    let collection_dir_clone = collection_dir.clone();
    let dir_exists = task::spawn_blocking(move || collection_dir_clone.exists())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))?;
    
    if dir_exists {
        // Remove the entire directory recursively
        task::spawn_blocking(move || fs::remove_dir_all(&collection_dir))
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))?
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to remove directory: {}", e)))?;
    }
    
    Ok(())
}