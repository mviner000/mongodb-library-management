use axum::{extract::{Path, State}, http::StatusCode, Json};
use rusqlite::{Connection, params, types::Null};
use serde_json::{Value, Map};
use serde::Deserialize;
use std::{
    fs,
    path::PathBuf,
    sync::Arc,
};
use tokio::{sync::Mutex, task};
use crate::api_server::state::ApiServerState;
use std::env;
use anyhow::{Result as AnyhowResult, anyhow, Result};

// Structure for request parsing
#[derive(Debug, Deserialize)]
pub struct CsvUpload {
    valid: Vec<Value>,
    invalid: Vec<Value>,
}

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

// Modified: Helper function to convert value to a rusqlite-compatible type
// Now returns Option<String> to properly represent NULL values
fn value_to_param(value: &Value) -> rusqlite::Result<Option<String>> {
    match value {
        Value::Null => Ok(None), // NULL value in JSON becomes NULL in SQL
        Value::Bool(b) => Ok(Some(b.to_string())),
        Value::Number(n) => Ok(Some(n.to_string())),
        Value::String(s) => {
            if s.is_empty() {
                Ok(None) // Treat empty strings as NULL
            } else {
                Ok(Some(s.clone()))
            }
        },
        Value::Array(_) | Value::Object(_) => Ok(Some(value.to_string())),
    }
}

// Create table for valid data
fn create_valid_table(conn: &Connection, data: &[Value]) -> Result<()> {
    conn.execute("DROP TABLE IF EXISTS valid_data", [])
        .map_err(|e| anyhow!("Failed to drop valid_data table: {}", e))?;
    
    if let Some(first) = data.first() {
        match first {
            Value::Object(obj) => {
                // Get headers and exclude 'id' field
                let headers: Vec<String> = obj.keys()
                    .filter(|&k| k != "id")
                    .map(|k| k.to_string())
                    .collect();
                
                // Build CREATE TABLE SQL
                let mut create_sql = "CREATE TABLE valid_data (id INTEGER PRIMARY KEY".to_string();
                for header in &headers {
                    // Escape column names to handle special characters
                    create_sql.push_str(&format!(", \"{}\" TEXT", header.replace("\"", "\"\"")));
                }
                create_sql.push_str(")");
                
                conn.execute(&create_sql, [])
                    .map_err(|e| anyhow!("Failed to create valid_data table: {}", e))?;
            },
            _ => return Err(anyhow!("First data item is not an object"))
        }
    } else {
        // Create empty table with just ID if no data
        conn.execute("CREATE TABLE valid_data (id INTEGER PRIMARY KEY)", [])
            .map_err(|e| anyhow!("Failed to create empty valid_data table: {}", e))?;
    }
    Ok(())
}

// Create table for invalid data
fn create_invalid_table(conn: &Connection) -> Result<()> {
    conn.execute("DROP TABLE IF EXISTS invalid_data", [])
        .map_err(|e| anyhow!("Failed to drop invalid_data table: {}", e))?;
        
    conn.execute(
        "CREATE TABLE invalid_data (
            id INTEGER PRIMARY KEY,
            row_data TEXT,
            errors TEXT
        )", 
        []
    ).map_err(|e| anyhow!("Failed to create invalid_data table: {}", e))?;
    
    Ok(())
}

// Modified: Insert valid data with proper NULL handling
fn insert_valid_data(conn: &Connection, data: &[Value]) -> Result<()> {
    if data.is_empty() {
        return Ok(());
    }
    
    let first_item = match &data[0] {
        Value::Object(obj) => obj,
        _ => return Err(anyhow!("Data items must be objects"))
    };
    
    // Get headers and exclude 'id' field
    let headers: Vec<String> = first_item.keys()
        .filter(|&k| k != "id")
        .map(|k| k.to_string())
        .collect();
    
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
        "INSERT INTO valid_data ({}) VALUES ({})",
        columns,
        placeholders.join(", ")
    );

    // Prepare statement once for multiple executions
    let mut stmt = conn.prepare(&insert_sql)
        .map_err(|e| anyhow!("Failed to prepare statement: {}", e))?;

    // Insert each row
    for item in data {
        let obj = match item {
            Value::Object(obj) => obj,
            _ => continue // Skip non-object items
        };
        
        // Extract values in the same order as headers
        let values: Vec<Option<String>> = headers.iter()
            .map(|h| {
                match obj.get(h) {
                    Some(val) => value_to_param(val)
                        .unwrap_or(None), // Convert to Option<String>
                    None => None // Missing field becomes NULL
                }
            })
            .collect();
            
        // Convert values to params for the prepared statement
        let params_slice: Vec<&dyn rusqlite::ToSql> = values
            .iter()
            .map(|v| match v {
                Some(s) => s as &dyn rusqlite::ToSql,
                None => &Null as &dyn rusqlite::ToSql, // Use rusqlite::types::Null for NULL values
            })
            .collect();
            
        stmt.execute(params_slice.as_slice())
            .map_err(|e| anyhow!("Failed to insert data: {}", e))?;
    }

    Ok(())
}

// Modified: Insert invalid data with proper NULL handling
fn insert_invalid_data(conn: &Connection, data: &[Value]) -> Result<()> {
    let mut stmt = conn.prepare(
        "INSERT INTO invalid_data (row_data, errors) VALUES (?, ?)"
    ).map_err(|e| anyhow!("Failed to prepare invalid data statement: {}", e))?;

    for item in data {
        let row_data = item.get("row")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "{}".to_string());
            
        let errors = item.get("errors")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "[]".to_string());
            
        stmt.execute(params![row_data, errors])
            .map_err(|e| anyhow!("Failed to insert invalid data: {}", e))?;
    }
    
    Ok(())
}

pub async fn save_csv_temp(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection): Path<String>,
    Json(data): Json<CsvUpload>,
) -> Result<(), (StatusCode, String)> {
    if data.valid.is_empty() && data.invalid.is_empty() {
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
    let data_clone = data;
    let db_path_clone = db_path.clone();
    
    // Execute SQLite operations in blocking task
    task::spawn_blocking(move || -> AnyhowResult<()> {
        let conn = Connection::open(&db_path_clone)
            .map_err(|e| anyhow!("Failed to open database: {}", e))?;
        
        // Create tables for valid and invalid data
        create_valid_table(&conn, &data_clone.valid)?;
        create_invalid_table(&conn)?;
        
        // Insert valid data
        insert_valid_data(&conn, &data_clone.valid)?;
        
        // Insert invalid data
        insert_invalid_data(&conn, &data_clone.invalid)?;

        Ok(())
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(())
}

// Modified: Load CSV temp with improved NULL handling in result processing
pub async fn load_csv_temp(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection): Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
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
    let results = task::spawn_blocking(move || -> AnyhowResult<Value> {
        let conn = Connection::open(&db_path)
            .map_err(|e| anyhow!("Failed to open database: {}", e))?;
        
        // Load valid data
        let valid_data = load_table_data(&conn, "valid_data")?;
        
        // Load invalid data
        let invalid_data = load_invalid_data(&conn)?;
        
        // Create response with both sets of data
        let mut response = Map::new();
        response.insert("valid".to_string(), Value::Array(valid_data));
        response.insert("invalid".to_string(), Value::Array(invalid_data));
        
        Ok(Value::Object(response))
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(results))
}

// Modified: Helper function with proper NULL handling
fn load_table_data(conn: &Connection, table_name: &str) -> Result<Vec<Value>> {
    // First check if table exists
    let table_exists = conn.query_row(
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?",
        params![table_name],
        |_| Ok(true)
    ).unwrap_or(false);
    
    if !table_exists {
        return Ok(Vec::new());
    }
    
    // Get the column names
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table_name))
        .map_err(|e| anyhow!("Failed to get table info: {}", e))?;
        
    let columns: Vec<String> = stmt.query_map([], |row| {
        let column_name: String = row.get(1)?;
        Ok(column_name)
    })
    .map_err(|e| anyhow!("Failed to query columns: {}", e))?
    .collect::<Result<Vec<String>, _>>()
    .map_err(|e| anyhow!("Column error: {}", e))?;
    
    // Prepare and execute SELECT statement for all rows
    let mut stmt = conn.prepare(&format!("SELECT * FROM {}", table_name))
        .map_err(|e| anyhow!("Failed to prepare statement: {}", e))?;
        
    let rows = stmt.query_map([], |row| {
        let mut obj = Map::new();
        
        // Include SQLite rowid as 'id'
        let id: i64 = row.get(0)?;
        obj.insert("id".to_string(), Value::Number(id.into()));
        
        // Add all other columns
        for (i, column_name) in columns.iter().enumerate() {
            if i == 0 { continue; } // Skip id column which we already handled
            
            // Modified: Check if column value is NULL
            let val: Option<String> = row.get(i)?;
            if let Some(s) = val {
                // Try to parse as JSON if it looks like JSON
                let json_val = if (s.starts_with('{') && s.ends_with('}')) || 
                   (s.starts_with('[') && s.ends_with(']')) ||
                   (s == "true" || s == "false" || s == "null") ||
                   s.parse::<f64>().is_ok() {
                    serde_json::from_str(&s).unwrap_or(Value::String(s))
                } else {
                    Value::String(s)
                };
                
                obj.insert(column_name.clone(), json_val);
            } else {
                // Insert explicit null for NULL values from DB
                obj.insert(column_name.clone(), Value::Null);
            }
        }
        
        Ok(Value::Object(obj))
    })
    .map_err(|e| anyhow!("Query failed: {}", e))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| anyhow!("Row error: {}", e))?);
    }

    Ok(results)
}

// Helper function to load invalid data
fn load_invalid_data(conn: &Connection) -> Result<Vec<Value>> {
    // Check if table exists
    let table_exists = conn.query_row(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='invalid_data'",
        [],
        |_| Ok(true)
    ).unwrap_or(false);
    
    if !table_exists {
        return Ok(Vec::new());
    }
    
    let mut stmt = conn.prepare("SELECT id, row_data, errors FROM invalid_data")
        .map_err(|e| anyhow!("Failed to prepare invalid data query: {}", e))?;
        
    let rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let row_data: String = row.get(1)?;
        let errors: String = row.get(2)?;
        
        let mut obj = Map::new();
        obj.insert("id".to_string(), Value::Number(id.into()));
        
        // Parse row_data JSON
        let row_json = match serde_json::from_str::<Value>(&row_data) {
            Ok(v) => v,
            Err(_) => {
                let mut map = Map::new();
                map.insert("raw_data".to_string(), Value::String(row_data));
                Value::Object(map)
            }
        };
        
        // Parse errors JSON
        let errors_json = match serde_json::from_str::<Value>(&errors) {
            Ok(v) => v,
            Err(_) => {
                let mut arr = Vec::new();
                arr.push(Value::String(errors));
                Value::Array(arr)
            }
        };
        
        obj.insert("row".to_string(), row_json);
        obj.insert("errors".to_string(), errors_json);
        
        Ok(Value::Object(obj))
    })
    .map_err(|e| anyhow!("Invalid data query failed: {}", e))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| anyhow!("Invalid data row error: {}", e))?);
    }

    Ok(results)
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