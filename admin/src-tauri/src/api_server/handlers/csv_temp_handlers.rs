// src/api_server/handlers/csv_temp_handlers.rs

use axum::{extract::{Path, State, Query}, http::StatusCode, Json}; // [cite: 707]
use rusqlite::{Connection, params, types::{Null, ValueRef}}; // <<< Added Transaction, OptionalExtension, ValueRef
use serde_json::{Value, Map, json}; // <<< Added json macro
use serde::Deserialize;
use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    collections::{HashMap, HashSet}, // <<< Added HashMap, HashSet
};
use tokio::{sync::Mutex, task}; // [cite: 708]
use crate::api_server::state::ApiServerState; // [cite: 708]
use std::env; // [cite: 708]
use anyhow::{Result as AnyhowResult, anyhow, Result}; // [cite: 708]
use mongodb::{ // <<< Added mongodb imports
    bson::{doc, Document, oid::ObjectId, Bson},
    Collection,
    options::FindOptions
};
use futures_util::TryStreamExt; // <<< Added TryStreamExt for MongoDB cursor

// <<< Add imports for schema service and database service >>>
use crate::api_server::services::database_service::get_database;
use crate::api_server::services::schema_service::get_collection_schema_internal;
use crate::api_server::models::ApiResponse; // <<< Added ApiResponse, error_response


// Structure for request parsing
#[derive(Debug, Deserialize)]
pub struct CsvUpload { // [cite: 709]
    valid: Vec<Value>, // [cite: 709]
    invalid: Vec<Value>, // [cite: 709]
}

// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationQuery { // [cite: 709]
    #[serde(default = "default_page")] // [cite: 709]
    pub valid_page: u32, // [cite: 709]
    #[serde(default = "default_page_size")] // [cite: 709]
    pub valid_page_size: u32, // [cite: 709]
    #[serde(default = "default_page")] // [cite: 709]
    pub invalid_page: u32, // [cite: 709]
    #[serde(default = "default_page_size")] // [cite: 709]
    pub invalid_page_size: u32, // [cite: 709]
}

fn default_page() -> u32 { 1 } // [cite: 709]
fn default_page_size() -> u32 { 20 } // [cite: 709]

// Helper function to get app data directory
fn get_app_data_dir() -> PathBuf { // [cite: 709]
    let app_name = "vue-tauri"; // [cite: 710]

    #[cfg(target_os = "windows")] // [cite: 710]
    { // [cite: 710]
        let app_data = env::var("APPDATA").unwrap_or_else(|_| { // [cite: 710]
            env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string()) + "\\AppData\\Roaming" // [cite: 710]
        });
        PathBuf::from(app_data).join(app_name) // [cite: 711]
    }

    #[cfg(target_os = "macos")] // [cite: 711]
    { // [cite: 711]
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string()); // [cite: 711]
        PathBuf::from(home).join("Library").join("Application Support").join(app_name) // [cite: 712]
    }

    #[cfg(target_os = "linux")] // [cite: 712]
    { // [cite: 712]
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string()); // [cite: 712]
        PathBuf::from(home).join(".local").join("share").join(app_name) // [cite: 713]
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))] // [cite: 713]
    { // [cite: 713]
        PathBuf::from(".").join(app_name) // [cite: 713]
    }
}

// Get the database path for a collection
pub async fn get_db_path_from_state(state: &Arc<Mutex<ApiServerState>>, collection: &str) -> AnyhowResult<PathBuf> { // [cite: 713]
    let state_guard = state.lock().await; // Use a different name to avoid shadowing
    let temp_dirs = state_guard.temp_dirs.lock().await; // [cite: 714]
    if let Some(path) = temp_dirs.get(collection) { // [cite: 714]
        Ok(path.clone()) // [cite: 714]
    } else {
        // Create a path in the app data directory if it doesn't exist
        let app_data_dir = task::spawn_blocking(get_app_data_dir) // [cite: 714]
            .await // [cite: 714]
            .map_err(|e| anyhow!("Failed to spawn blocking task: {}", e))?; // [cite: 714]
        let temp_dir = app_data_dir.join("temp").join(collection); // [cite: 715]

        let temp_dir_clone = temp_dir.clone(); // [cite: 715]
        task::spawn_blocking(move || { // [cite: 715]
            fs::create_dir_all(&temp_dir_clone) // [cite: 715]
                .map_err(|e| anyhow!("Failed to create temp directory: {}", e)) // [cite: 715]
        })
        .await // [cite: 715]
        .map_err(|e| anyhow!("Task join error: {}", e))??; // [cite: 715]
        let db_path = temp_dir.join("temp_data.sqlite"); // [cite: 716]
        Ok(db_path) // [cite: 716]
    }
}

// Add path to state
async fn add_path_to_state(state: &Arc<Mutex<ApiServerState>>, collection: String, path: PathBuf) -> AnyhowResult<()> { // [cite: 716]
    let state_guard = state.lock().await; // Use a different name
    let mut temp_dirs = state_guard.temp_dirs.lock().await; // [cite: 717]
    temp_dirs.insert(collection, path); // [cite: 717]
    Ok(()) // [cite: 717]
}

// Remove path from state
async fn remove_path_from_state(state: &Arc<Mutex<ApiServerState>>, collection: &str) -> AnyhowResult<Option<PathBuf>> { // [cite: 717]
    let state_guard = state.lock().await; // Use a different name
    let mut temp_dirs = state_guard.temp_dirs.lock().await; // [cite: 718]
    let result = temp_dirs.remove(collection); // [cite: 718]
    Ok(result) // [cite: 718]
}

// Helper function to convert value to a rusqlite-compatible type
// Helper function to convert value to a rusqlite-compatible type
fn value_to_param(value: &Value) -> rusqlite::Result<Option<String>> {
    match value {
        Value::Null => Ok(None),
        Value::Bool(b) => Ok(Some(b.to_string())),
        Value::Number(n) => Ok(Some(n.to_string())),
        Value::String(s) => {
            // Treat empty strings as NULL
            if s.is_empty() {
                Ok(None)
            } else {
                Ok(Some(s.clone()))
            }
        },
        Value::Array(_) => {
            // Just convert arrays to string representation
            let str_val = value.to_string();
            Ok(Some(str_val))
        },
        Value::Object(map) => {
            // Handle BSON ObjectId within Object
            if let Some(oid_val) = map.get("$oid") {
                if let Value::String(oid_str) = oid_val {
                    return Ok(Some(oid_str.clone()));
                }
            }
            // Default to string representation for objects
            let str_val = value.to_string();
            Ok(Some(str_val))
        },
    }
}

// Helper to get all column names from a SQLite table
fn get_table_columns(conn: &Connection, table_name: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table_name))
        .map_err(|e| anyhow!("Failed to get table info for {}: {}", table_name, e))?;
    let columns: Vec<String> = stmt.query_map([], |row| {
        let column_name: String = row.get(1)?;
        Ok(column_name)
    })
    .map_err(|e| anyhow!("Failed to query columns for {}: {}", table_name, e))?
    .collect::<Result<Vec<String>, _>>()
    .map_err(|e| anyhow!("Column error for {}: {}", table_name, e))?;
    Ok(columns)
}


// Create table for valid data with _id TEXT as primary key
fn create_valid_table(conn: &Connection, data: &[Value]) -> Result<()> { // [cite: 721]
    match conn.execute("DROP TABLE IF EXISTS valid_data", []) { // [cite: 721]
        Ok(_) => {}, // [cite: 721]
        Err(e) => return Err(anyhow!("Failed to drop valid_data table: {}", e)) // [cite: 721]
    }

    if let Some(first) = data.first() { // [cite: 721]
        match first { // [cite: 721]
            Value::Object(obj) => { // [cite: 722]
                // Get headers and exclude 'id' field (we'll use '_id' as primary key)
                let headers: Vec<String> = obj.keys() // [cite: 722]
                    .filter(|&k| k != "id" && k != "_id") // [cite: 722]
                    .map(|k| k.to_string()) // [cite: 723]
                    .collect(); // [cite: 723]
                // Build CREATE TABLE SQL with _id as primary key
                let mut create_sql = "CREATE TABLE valid_data (\"_id\" TEXT PRIMARY KEY".to_string(); // [cite: 724]
                for header in &headers { // [cite: 724]
                    // Escape column names to handle special characters
                    let column_def = format!(", \"{}\" TEXT", header.replace("\"", "\"\"")); // [cite: 725]
                    create_sql.push_str(&column_def); // [cite: 725]
                }
                create_sql.push_str(")"); // [cite: 725]

                match conn.execute(&create_sql, []) { // [cite: 725]
                    Ok(_) => Ok(()), // [cite: 726]
                    Err(e) => Err(anyhow!("Failed to create valid_data table: {}", e)) // [cite: 726]
                }
            },
            _ => Err(anyhow!("First data item is not an object")) // [cite: 726]
        }
    } else { // [cite: 727]
        // Create empty table with just _id if no data
        match conn.execute("CREATE TABLE valid_data (\"_id\" TEXT PRIMARY KEY)", []) { // [cite: 727]
            Ok(_) => Ok(()), // [cite: 727]
            Err(e) => Err(anyhow!("Failed to create empty valid_data table: {}", e)) // [cite: 727]
        }
    }
}

// Create table for invalid data with CSV headers + errors column
fn create_invalid_table(conn: &Connection, csv_columns: &[String]) -> Result<()> { // [cite: 728]
    match conn.execute("DROP TABLE IF EXISTS invalid_data", []) { // [cite: 728]
        Ok(_) => {}, // [cite: 728]
        Err(e) => return Err(anyhow!("Failed to drop invalid_data table: {}", e)) // [cite: 728]
    }

    // Build the CREATE TABLE SQL statement
    let mut create_sql = "CREATE TABLE invalid_data (\"_id\" TEXT PRIMARY KEY".to_string(); // [cite: 728]
    // Add all CSV columns as TEXT, ensuring _id isn't duplicated
    for column in csv_columns { // [cite: 729]
        if column != "_id" { // [cite: 729]
            let column_def = format!(", \"{}\" TEXT", column.replace("\"", "\"\"")); // [cite: 729]
            create_sql.push_str(&column_def); // [cite: 729]
        }
    }

    // Add errors column
    create_sql.push_str(", errors TEXT)"); // [cite: 729]

    // Execute the CREATE TABLE statement
    match conn.execute(&create_sql, []) { // [cite: 730]
        Ok(_) => Ok(()), // [cite: 730]
        Err(e) => Err(anyhow!("Failed to create invalid_data table: {}", e)) // [cite: 730]
    }
}


// Insert valid data with _id as primary key
fn insert_valid_data(conn: &Connection, data: &[Value]) -> Result<()> { // [cite: 730]
    if data.is_empty() { // [cite: 730]
        return Ok(()); // [cite: 730]
    }

    let first_item = match &data[0] { // [cite: 730]
        Value::Object(obj) => obj, // [cite: 730]
        _ => return Err(anyhow!("Data items must be objects")) // [cite: 731]
    };

    // Get headers excluding 'id' field
    let headers: Vec<String> = first_item.keys() // [cite: 731]
        .filter(|&k| k != "id" && k != "_id") // [cite: 731]
        .map(|k| k.to_string()) // [cite: 731]
        .collect(); // [cite: 731]
    // Build placeholders for prepared statement (including _id placeholder)
    let placeholders: Vec<String> = (0..headers.len() + 1) // [cite: 732]
        .map(|_| "?".to_string()) // [cite: 732]
        .collect(); // [cite: 732]
    // Build columns for INSERT statement (including _id column)
    let columns = format!("\"_id\", {}", headers.iter() // [cite: 733]
        .map(|h| format!("\"{}\"", h.replace("\"", "\"\""))) // [cite: 733]
        .collect::<Vec<_>>() // [cite: 733]
        .join(", ")); // [cite: 733]

    // Create INSERT SQL statement
    let insert_sql = format!( // [cite: 733]
        "INSERT INTO valid_data ({}) VALUES ({})", // [cite: 733]
        columns, // [cite: 733]
        placeholders.join(", ") // [cite: 733]
    ); // [cite: 734]

    // Prepare statement once for multiple executions
    let mut stmt = match conn.prepare(&insert_sql) { // [cite: 734]
        Ok(stmt) => stmt, // [cite: 734]
        Err(e) => return Err(anyhow!("Failed to prepare statement: {}", e)) // [cite: 734]
    };

    // Insert each row
    for item in data { // [cite: 734]
        let obj = match item { // [cite: 734]
            Value::Object(obj) => obj, // [cite: 734]
            _ => continue // Skip non-object items // [cite: 735]
        };

        // Extract _id, prioritize "_id" field, then look for "$oid", finally "id"
         let mongo_id = match obj.get("_id") {
            Some(Value::Object(id_obj)) => {
                match id_obj.get("$oid") {
                    Some(Value::String(oid_str)) if !oid_str.is_empty() => oid_str.clone(),
                    _ => uuid::Uuid::new_v4().to_string() // Fallback if $oid is not string or empty
                }
            }
            Some(Value::String(id_str)) if !id_str.is_empty() => id_str.clone(), // Handle simple string _id
            _ => match obj.get("id") { // Fallback to "id"
                 Some(Value::String(id)) if !id.is_empty() => id.clone(), // [cite: 736]
                 Some(Value::Number(n)) => n.to_string(), // [cite: 736]
                 _ => uuid::Uuid::new_v4().to_string() // Generate UUID if no id // [cite: 736]
             }
        };

        // Extract values in the same order as headers
        let mut values: Vec<Option<String>> = vec![Some(mongo_id)]; // [cite: 737]
        // Start with _id // [cite: 738]

        for h in &headers { // [cite: 738]
            let value = match obj.get(h) { // [cite: 738]
                Some(val) => { // [cite: 738]
                    match value_to_param(val) { // [cite: 738]
                        Ok(v) => v, // [cite: 739]
                        Err(e) => { // [cite: 739]
                            return Err(anyhow!("Failed to convert value for {}: {}", h, e)); // [cite: 739]
                        } // [cite: 740]
                    }
                },
                None => None // Missing field becomes NULL // [cite: 740]
            };
            values.push(value); // [cite: 741]
        }

        // Convert values to params for the prepared statement
        let params_slice: Vec<&dyn rusqlite::ToSql> = values // [cite: 741]
            .iter() // [cite: 741]
            .map(|v| match v { // [cite: 741]
                Some(s) => s as &dyn rusqlite::ToSql, // [cite: 741]
                None => &Null as &dyn rusqlite::ToSql // Use rusqlite::types::Null for NULL values // [cite: 742]
            })
            .collect(); // [cite: 742]
        match stmt.execute(params_slice.as_slice()) { // [cite: 743]
            Ok(_) => {}, // [cite: 743]
            Err(e) => return Err(anyhow!("Failed to insert data: {}", e)) // [cite: 743]
        }
    }

    Ok(()) // [cite: 743]
}


// Insert invalid data with specified columns + errors
fn insert_invalid_data_with_cols(
    conn: &Connection,
    data: &[Map<String, Value>], // Change to take Map directly
    all_columns: &[String], // Columns defined in invalid_data table
) -> Result<()> {
    if data.is_empty() {
        return Ok(());
    }

    // Build columns list including _id and errors
    let mut columns_str = Vec::new();
    for col in all_columns {
        columns_str.push(format!("\"{}\"", col.replace("\"", "\"\"")));
    }

    // Build placeholders
    let placeholders: Vec<String> = (0..all_columns.len()).map(|_| "?".to_string()).collect();

    // Build INSERT or REPLACE statement (use REPLACE to handle updates easily)
    let insert_sql = format!(
        "INSERT OR REPLACE INTO invalid_data ({}) VALUES ({})",
        columns_str.join(", "),
        placeholders.join(", ")
    );

    let mut stmt = conn.prepare(&insert_sql)
        .map_err(|e| anyhow!("Failed to prepare invalid data statement: {}", e))?;

    for obj in data { // Iterate over Map directly
        // Extract or generate _id
        let mongo_id = match obj.get("_id") {
             Some(Value::Object(id_obj)) => id_obj.get("$oid").and_then(Value::as_str).map(String::from),
             Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
             _ => match obj.get("id") {
                  Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
                  Some(Value::Number(n)) => Some(n.to_string()),
                  _ => None // Let it be generated if truly missing later
              }
        }.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()); // Generate UUID only if completely missing

        // Extract values for each column defined in the invalid_data table
        let mut values: Vec<Option<String>> = Vec::with_capacity(all_columns.len());
        for column in all_columns {
             let value = match obj.get(column) {
                 Some(val) => value_to_param(val).ok().flatten(), // Convert to param, handle error gracefully
                 None => None, // Missing field becomes NULL
             };
             values.push(value);
        }

        // Ensure the first value corresponds to _id (if _id is the first column)
        if !all_columns.is_empty() && all_columns[0] == "_id" {
            values[0] = Some(mongo_id); // Overwrite first value with the definite mongo_id
        } else {
            // Handle case where _id might not be the first column (less likely)
            // Find the index of _id and insert/replace there if necessary
        }


        // Convert values to params for the prepared statement
        let params_slice: Vec<&dyn rusqlite::ToSql> = values
            .iter()
            .map(|v| match v {
                Some(s) => s as &dyn rusqlite::ToSql,
                None => &Null as &dyn rusqlite::ToSql,
            })
            .collect();

        stmt.execute(params_slice.as_slice())
            .map_err(|e| anyhow!("Failed to insert/replace invalid data: {}", e))?;
    }

    Ok(())
}


// Original function kept for saving initial CSV data
fn insert_invalid_data(conn: &Connection, data: &[Value], csv_columns: &[String]) -> Result<()> { // [cite: 743]
    if data.is_empty() { // [cite: 743]
        return Ok(()); // [cite: 743]
    } // [cite: 744]

    // Build columns list including _id and errors
    let mut all_columns = vec!["_id".to_string()]; // Start with _id
    all_columns.extend(csv_columns.iter().filter(|&c| c != "_id").cloned()); // Add CSV columns except _id
    all_columns.push("errors".to_string()); // Add errors column

    // Build columns string for SQL
    let columns_sql = all_columns.iter()
        .map(|c| format!("\"{}\"", c.replace("\"", "\"\"")))
        .collect::<Vec<_>>()
        .join(", ");

    // Build placeholders
    let placeholders: Vec<String> = (0..all_columns.len()) // [cite: 745]
        .map(|_| "?".to_string()) // [cite: 745]
        .collect(); // [cite: 745]

    // Build INSERT SQL statement
    let insert_sql = format!( // [cite: 746]
        "INSERT INTO invalid_data ({}) VALUES ({})", // [cite: 746]
        columns_sql, // Use generated columns string
        placeholders.join(", ") // [cite: 746]
    ); // [cite: 746]

    let mut stmt = match conn.prepare(&insert_sql) { // [cite: 746]
        Ok(stmt) => stmt, // [cite: 746]
        Err(e) => return Err(anyhow!("Failed to prepare invalid data statement: {}", e)) // [cite: 746]
    };

    for item in data { // [cite: 746]
        let obj = match item { // [cite: 747]
            Value::Object(obj) => obj, // [cite: 747]
            _ => continue // Skip non-object items // [cite: 747]
        };

         // Extract or generate _id
         let mongo_id = match obj.get("_id") {
             Some(Value::Object(id_obj)) => id_obj.get("$oid").and_then(Value::as_str).map(String::from),
             Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
             _ => match obj.get("id") {
                  Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
                  Some(Value::Number(n)) => Some(n.to_string()),
                  _ => None // Let it be generated if truly missing later
              }
         }.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()); // Generate UUID only if completely missing

         // Extract values for each column in the correct order
         let mut values: Vec<Option<String>> = Vec::with_capacity(all_columns.len());
         for column in &all_columns {
             if column == "_id" {
                 values.push(Some(mongo_id.clone())); // Push the determined _id
             } else if column == "errors" {
                 let errors_val = match obj.get("errors") {
                     Some(v) => v.to_string(), // Serialize errors array/object to string
                     None => "[]".to_string() // Default to empty JSON array string
                 };
                 values.push(Some(errors_val));
             } else {
                 // Get value for other CSV columns
                 let value = match obj.get(column) {
                     Some(val) => value_to_param(val).ok().flatten(),
                     None => None
                 };
                 values.push(value);
             }
         }


        // Convert values to params for the prepared statement
        let params_slice: Vec<&dyn rusqlite::ToSql> = values // [cite: 754]
            .iter() // [cite: 754]
            .map(|v| match v { // [cite: 754]
                Some(s) => s as &dyn rusqlite::ToSql, // [cite: 754]
                None => &Null as &dyn rusqlite::ToSql // [cite: 754]
            }) // [cite: 755]
            .collect(); // [cite: 755]
        match stmt.execute(params_slice.as_slice()) { // [cite: 756]
            Ok(_) => {}, // [cite: 756]
            Err(e) => return Err(anyhow!("Failed to insert invalid data: {}", e)) // [cite: 756]
        }
    }

    Ok(()) // [cite: 756]
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

        // Create valid data table
        create_valid_table(&conn, &data_clone.valid)?;

        // Extract CSV columns from invalid data (all columns except errors)
        let csv_columns = if let Some(first_invalid) = data_clone.invalid.first() {
            if let Value::Object(obj) = first_invalid {
                obj.keys()
                    .filter(|&k| k != "errors")
                    .map(|k| k.to_string())
                    .collect::<Vec<_>>()
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        // Create invalid table with CSV columns + errors
        create_invalid_table(&conn, &csv_columns)?;

        // Insert valid data
        insert_valid_data(&conn, &data_clone.valid)?;
        // Insert invalid data with CSV columns
        insert_invalid_data(&conn, &data_clone.invalid, &csv_columns)?;
        Ok(())
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(())
}

// Build paginated response
fn build_pagination_response( // [cite: 768]
    data: Vec<Value>, // [cite: 768]
    total: u32, // [cite: 768]
    page: u32, // [cite: 768]
    page_size: u32 // [cite: 768]
) -> Value {
    let mut obj = Map::new(); // [cite: 768]
    obj.insert("data".to_string(), Value::Array(data)); // [cite: 769]
    obj.insert("total".to_string(), Value::Number(total.into())); // [cite: 769]
    obj.insert("page".to_string(), Value::Number(page.into())); // [cite: 769]
    obj.insert("page_size".to_string(), Value::Number(page_size.into())); // [cite: 769]
    Value::Object(obj) // [cite: 769]
}


// Modified load_table_data function with pagination
fn load_table_data( // [cite: 769]
    conn: &Connection, // [cite: 769]
    table_name: &str, // [cite: 769]
    page: u32, // [cite: 769]
    page_size: u32, // [cite: 769]
) -> Result<(Vec<Value>, u32)> {
    println!("[DEBUG] Loading data from {} table, page {}, page size {}", table_name, page, page_size); // [cite: 769]
    // First check if table exists
    let table_exists = conn.query_row( // [cite: 770]
        "SELECT name FROM sqlite_master WHERE type='table' AND name=?", // [cite: 770]
        params![table_name], // [cite: 770]
        |_| Ok(true) // [cite: 770]
    ).unwrap_or(false); // [cite: 770]
    if !table_exists { // [cite: 771]
        println!("[DEBUG] Table {} does not exist", table_name); // [cite: 771]
        return Ok((Vec::new(), 0)); // [cite: 771]
    } // [cite: 772]

    // Get total count
    let total: u32 = conn.query_row( // [cite: 772]
        &format!("SELECT COUNT(*) FROM {}", table_name), // [cite: 772]
        [], // [cite: 772]
        |row| row.get(0), // [cite: 772]
    )?; // [cite: 772]
    println!("[DEBUG] Total records in {}: {}", table_name, total); // [cite: 773]

    // Get the column names using helper function
    let columns = get_table_columns(conn, table_name)?; // [cite: 774]

    println!("[DEBUG] Found {} columns in {}: {:?}", columns.len(), table_name, columns); // [cite: 775]
    // Calculate offset for pagination
    let offset = (page - 1) * page_size; // [cite: 776]
    println!("[DEBUG] Calculated offset: {}", offset); // [cite: 777]

    // Prepare and execute SELECT statement with pagination
    let query = format!("SELECT * FROM {} ORDER BY \"_id\" LIMIT ? OFFSET ?", table_name); // [cite: 777]
    println!("[DEBUG] Executing query: {} with params [page_size={}, offset={}]", query, page_size, offset); // [cite: 778]
    let mut stmt = conn.prepare(&query) // [cite: 779]
        .map_err(|e| anyhow!("Failed to prepare statement: {}", e))?; // [cite: 779]
    // Modified query_map implementation to return Value directly
    let rows = stmt.query_map( 
        params![page_size as i64, offset as i64],
        |row| { 
            let mut obj = Map::new();

            // Process each column
            for (i, column_name) in columns.iter().enumerate() {
                // Check column type and get value accordingly
                let value_ref = row.get_ref(i)?;

                let val = match value_ref {
                    ValueRef::Null => Value::Null,
                    ValueRef::Integer(i) => Value::Number(i.into()),
                    ValueRef::Real(f) => Value::Number(serde_json::Number::from_f64(f).unwrap_or_else(|| 0.into())),
                    ValueRef::Text(t) => {
                        let s = String::from_utf8_lossy(t).to_string();
                        // Special handling for errors column in invalid_data table
                        if table_name == "invalid_data" && column_name == "errors" {
                            // Try to parse errors as JSON
                            if let Ok(json_val) = serde_json::from_str::<Value>(&s) {
                                obj.insert(column_name.clone(), json_val); // Insert directly into obj
                                continue; // Skip the rest of this iteration
                            }
                        }
                        // Try to parse other text that looks like JSON, bool, null, or number
                        if (s.starts_with('{') && s.ends_with('}')) || 
                        (s.starts_with('[') && s.ends_with(']')) || 
                        (s == "true" || s == "false" || s == "null") || 
                        s.parse::<f64>().is_ok() {
                            serde_json::from_str(&s).unwrap_or(Value::String(s))
                        } else {
                            Value::String(s)
                        }
                    }
                    ValueRef::Blob(_) => Value::String("<blob>".to_string()), // Represent blob as string
                };

                // BSON ObjectId representation for _id if it's a string
                if column_name == "_id" {
                    if let Value::String(id_str) = val {
                        obj.insert(column_name.clone(), json!({ "$oid": id_str }));
                    } else {
                        obj.insert(column_name.clone(), val); // Insert as is if not string _id
                    }
                } else {
                    obj.insert(column_name.clone(), val);
                }
            }

            // Include _id as id for backward compatibility (using the BSON ObjectId format)
            if let Some(mongo_id_obj) = obj.get("_id").cloned() {
                if mongo_id_obj.is_object() {
                    obj.insert("id".to_string(), mongo_id_obj.clone()); // Insert the {"$oid": ...} object
                }
            }

            Ok(Value::Object(obj)) // Return Value directly
        }
    )
    .map_err(|e| anyhow!("Query failed: {}", e))?;
    let mut results = Vec::new(); // [cite: 790]
    for row in rows { // [cite: 790]
        results.push(row.map_err(|e| anyhow!("Row error: {}", e))?); // [cite: 790]
    } // [cite: 791]

    println!("[DEBUG] Query returned {} rows", results.len()); // [cite: 791]
    Ok((results, total)) // [cite: 792]
}


// Load CSV temp with pagination support
pub async fn load_csv_temp( // [cite: 792]
    State(state): State<Arc<Mutex<ApiServerState>>>, // [cite: 792]
    Path(collection): Path<String>, // [cite: 792]
    Query(params): Query<PaginationQuery>, // [cite: 792]
) -> Result<Json<Value>, (StatusCode, String)> {
    // Add request logging
    println!("[DEBUG] Loading CSV temp data for {} with params: {:?}", collection, params); // [cite: 792]
    // Validate and clamp pagination values
    let valid_page = params.valid_page.max(1); // [cite: 793]
    let valid_page_size = params.valid_page_size.clamp(1, 100); // [cite: 793]
    let invalid_page = params.invalid_page.max(1); // [cite: 794]
    let invalid_page_size = params.invalid_page_size.clamp(1, 100); // [cite: 794]

    println!("[DEBUG] Processed pagination: valid_page={}, valid_page_size={}, invalid_page={}, invalid_page_size={}", // [cite: 794]
        valid_page, valid_page_size, invalid_page, invalid_page_size); // [cite: 794]
    // Get the database path from state
    let db_path = match get_db_path_from_state(&state, &collection).await { // Use match for better error handling // [cite: 795]
        Ok(path) => path, // [cite: 795]
        Err(_) => { // If path doesn't exist in state, it likely means no temp file was created yet
            println!("[DEBUG] No db path found in state for collection: {}", collection);
            // Return empty data structure instead of 404? Or let it fail below?
            // Returning empty structure seems more graceful for the frontend.
             let empty_response = json!({
                 "valid": build_pagination_response(vec![], 0, 1, valid_page_size),
                 "invalid": build_pagination_response(vec![], 0, 1, invalid_page_size)
             });
             return Ok(Json(empty_response));
        }
    };
    println!("[DEBUG] Database path: {:?}", db_path); // [cite: 796]

    // Check if the database file exists
    let db_path_clone = db_path.clone(); // [cite: 796]
    let file_exists = task::spawn_blocking(move || db_path_clone.exists()) // [cite: 797]
        .await // [cite: 797]
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))?; // [cite: 797]
    if !file_exists { // [cite: 798]
        println!("[DEBUG] Database file does not exist at path: {:?}", db_path); // [cite: 798]
         // Return empty structure if file doesn't exist
         let empty_response = json!({
             "valid": build_pagination_response(vec![], 0, 1, valid_page_size),
             "invalid": build_pagination_response(vec![], 0, 1, invalid_page_size)
         });
         return Ok(Json(empty_response)); // Return OK with empty data
        // return Err((StatusCode::NOT_FOUND, format!("No temporary data file found for collection: {}", collection))); // [cite: 799]
    } // [cite: 800]

    println!("[DEBUG] Database file exists, proceeding with query"); // [cite: 800]
    // Execute SQLite operations in blocking task
    let results = task::spawn_blocking(move || -> AnyhowResult<Value> { // [cite: 801]
        let conn = Connection::open(&db_path) // [cite: 801]
            .map_err(|e| anyhow!("Failed to open database: {}", e))?; // [cite: 801]

        // Load valid data with pagination
        println!("[DEBUG] Loading valid data - page: {}, size: {}", valid_page, valid_page_size); // [cite: 801]
        let (valid_data, valid_total) = load_table_data( // [cite: 801]
            &conn, "valid_data", // [cite: 802]
            valid_page, valid_page_size // [cite: 802]
        )?; // [cite: 802]
        println!("[DEBUG] Loaded {} valid items (total: {})", valid_data.len(), valid_total); // [cite: 802]

        // Load invalid data with pagination
        println!("[DEBUG] Loading invalid data - page: {}, size: {}", invalid_page, invalid_page_size); // [cite: 802]
        let (invalid_data, invalid_total) = load_table_data( // [cite: 802]
            &conn, "invalid_data", // [cite: 803]
            invalid_page, invalid_page_size // [cite: 803]
        )?; // [cite: 803]
        println!("[DEBUG] Loaded {} invalid items (total: {})", invalid_data.len(), invalid_total); // [cite: 803]
        // Create response with both sets of data
        let mut response = Map::new(); // [cite: 804]
        response.insert("valid".to_string(), build_pagination_response( // [cite: 805]
            valid_data, valid_total, valid_page, valid_page_size // [cite: 805]
        )); // [cite: 805]
        response.insert("invalid".to_string(), build_pagination_response( // [cite: 806]
            invalid_data, invalid_total, invalid_page, invalid_page_size // [cite: 806]
        )); // [cite: 806]
        println!("[DEBUG] Created response with pagination data"); // [cite: 807]

        Ok(Value::Object(response)) // [cite: 807]
    })
    .await // [cite: 807]
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))? // [cite: 807]
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?; // [cite: 808]

    println!("[DEBUG] Returning response with valid and invalid data"); // [cite: 808]
    Ok(Json(results)) // [cite: 809]
}

pub async fn delete_csv_temp( // [cite: 809]
    State(state): State<Arc<Mutex<ApiServerState>>>, // [cite: 809]
    Path(collection): Path<String>, // [cite: 809]
) -> Result<(), (StatusCode, String)> {
    // Get app data directory to find the collection directory
    let app_data_dir = task::spawn_blocking(get_app_data_dir) // [cite: 809]
        .await // [cite: 809]
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to spawn blocking task: {}", e)))?; // [cite: 809]
    // The collection directory is in "temp/{collection}"
    let collection_dir = app_data_dir.join("temp").join(&collection); // [cite: 810]
    // Remove path from state
    let _ = remove_path_from_state(&state, &collection) // [cite: 811]
        .await // [cite: 811]
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?; // [cite: 811]
    // Check if directory exists and remove it recursively
    let collection_dir_clone = collection_dir.clone(); // [cite: 812]
    let dir_exists = task::spawn_blocking(move || collection_dir_clone.exists()) // [cite: 813]
        .await // [cite: 813]
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))?; // [cite: 813]
    if dir_exists { // [cite: 814]
        // Remove the entire directory recursively
        task::spawn_blocking(move || fs::remove_dir_all(&collection_dir)) // [cite: 814]
            .await // [cite: 814]
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))? // [cite: 814]
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to remove directory: {}", e)))?; // [cite: 815]
    }

    Ok(()) // [cite: 815]
}


// ==========================================================================
// New Validation Handler
// ==========================================================================

// Structure for the validation response
#[derive(serde::Serialize, Debug)] 
pub struct ValidationSummary {
    validated_count: usize,
    conflicts_found: usize,
    remaining_valid: usize,
}

pub async fn validate_csv_temp_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Path(collection_name): Path<String>,
) -> Result<Json<ApiResponse<ValidationSummary>>, (StatusCode, String)> {
    println!("[VALIDATE] Starting validation for collection: {}", collection_name);

    // 1. Get DB Paths and Connections
    let db_path = get_db_path_from_state(&state, &collection_name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get temp DB path: {}", e)))?;

    if !db_path.exists() {
         println!("[VALIDATE] Temp DB file not found: {:?}", db_path);
         return Err((StatusCode::NOT_FOUND, "Temporary data not found. Please upload again.".to_string()));
    }

    let state_guard = state.lock().await; // Lock state once
    let mongodb_state = &state_guard.mongodb_state;
    let mongo_db = match get_database(mongodb_state).await {
         Ok(db) => db,
         Err((status, e)) => return Err((status, format!("MongoDB connection error: {}", e))),
     };
    println!("[VALIDATE] MongoDB connection successful");


    // Clone necessary data for blocking task
    let db_path_clone = db_path.clone();
    let collection_name_clone = collection_name.clone();
    let mongo_db_clone = mongo_db.clone(); // Clone the Database handle


    // 2. Execute Core Logic in Blocking Task
    let validation_result = task::spawn_blocking(move || -> AnyhowResult<ValidationSummary> {
        println!("[VALIDATE Task] Opening SQLite connection to {:?}", db_path_clone);
        // Make conn mutable for transaction
        let mut conn = Connection::open(&db_path_clone)
            .map_err(|e| anyhow!("Failed to open SQLite database: {}", e))?;

        // 3. Fetch Schema & Identify Unique Fields (Inside blocking task is fine for schema fetch)
        println!("[VALIDATE Task] Fetching MongoDB schema for {}", collection_name_clone);
        let schema = futures::executor::block_on(get_collection_schema_internal(&mongo_db_clone, &collection_name_clone))
             .map_err(|e| anyhow!("Failed to fetch MongoDB schema: {}", e))?;
        println!("[VALIDATE Task] Schema fetched successfully");

        let unique_fields = schema
            .get_document("properties")?
            .iter()
            .filter_map(|(key, value)| {
                if let Bson::Document(props) = value {
                    if props.get_bool("unique").unwrap_or(false) {
                        return Some(key.clone());
                    }
                }
                None
            })
            .collect::<HashSet<String>>();
        println!("[VALIDATE Task] Identified unique fields: {:?}", unique_fields);


        // 4. Read `valid_data` table - use scoping to limit the borrow
        println!("[VALIDATE Task] Reading data from valid_data table");
        let valid_data_sqlite: Vec<HashMap<String, Option<String>>> = {
            let valid_columns = get_table_columns(&conn, "valid_data")?;
            let mut stmt = conn.prepare("SELECT * FROM valid_data")?;
            let valid_rows_iter = stmt.query_map([], |row| {
                let mut map = HashMap::new();
                for (i, col_name) in valid_columns.iter().enumerate() {
                    let value: Option<String> = row.get(i)?;
                    map.insert(col_name.clone(), value);
                }
                Ok(map)
            })?;

            let mut data = Vec::new();
            for row_result in valid_rows_iter {
                data.push(row_result?);
            }
            data
        };
        
        let initial_valid_count = valid_data_sqlite.len();
        println!("[VALIDATE Task] Read {} rows from valid_data", initial_valid_count);

        if initial_valid_count == 0 {
             println!("[VALIDATE Task] No valid data to validate.");
             return Ok(ValidationSummary {
                 validated_count: 0,
                 conflicts_found: 0,
                 remaining_valid: 0,
             });
        }


        // 5. Perform MongoDB Checks (Batched)
        let mongo_coll: Collection<Document> = mongo_db_clone.collection(&collection_name_clone);
         let mut existing_ids = HashSet::new();
         let mut existing_unique_values: HashMap<String, HashSet<String>> = HashMap::new();

         // --- Batch ID Check ---
         let ids_to_check: Vec<String> = valid_data_sqlite.iter()
             .filter_map(|row| row.get("_id").cloned().flatten())
             .collect();

         if !ids_to_check.is_empty() {
             println!("[VALIDATE Task] Checking {} _ids against MongoDB", ids_to_check.len());
             let id_object_ids_result: Result<Vec<ObjectId>, _> = ids_to_check.iter()
                 .map(|id| ObjectId::parse_str(id).map_err(|e| anyhow!("Invalid ObjectId format '{}': {}", id, e)))
                 .collect();

             match id_object_ids_result {
                 Ok(id_object_ids) => {
                     let filter = doc! { "_id": { "$in": id_object_ids } };
                     let cursor = futures::executor::block_on(mongo_coll.find(filter, None))?;
                     let existing_docs: Vec<Document> = futures::executor::block_on(cursor.try_collect())?;
                     existing_ids = existing_docs.iter()
                         .filter_map(|doc| doc.get_object_id("_id").ok().map(|oid| oid.to_hex()))
                         .collect();
                     println!("[VALIDATE Task] Found {} existing _ids in MongoDB", existing_ids.len());
                 }
                 Err(e) => {
                     eprintln!("[VALIDATE Task] Error parsing ObjectIds: {}", e);
                     // Decide how to handle: fail validation or continue without ID check?
                     // Failing seems safer.
                     return Err(e);
                 }
             }
         }


         // --- Batch Unique Field Checks ---
         for field in &unique_fields {
             let values_to_check: Vec<String> = valid_data_sqlite.iter()
                 .filter_map(|row| row.get(field).cloned().flatten())
                 .filter(|v| !v.is_empty()) // Don't check empty strings for uniqueness
                 .collect::<HashSet<_>>() // Check only distinct values from SQLite
                 .into_iter()
                 .collect();

             if !values_to_check.is_empty() {
                 println!("[VALIDATE Task] Checking {} unique values for field '{}' against MongoDB", values_to_check.len(), field);
                 let filter = doc! { field: { "$in": values_to_check } };
                 // Optimization: Only project the necessary field and _id
                 let options = FindOptions::builder().projection(doc! { field: 1 }).build();
                 let cursor = futures::executor::block_on(mongo_coll.find(filter, options))?;
                 let existing_docs: Vec<Document> = futures::executor::block_on(cursor.try_collect())?;

                 let found_values: HashSet<String> = existing_docs.iter()
                    .filter_map(|doc| doc.get_str(field).ok().map(String::from))
                    .collect();

                 existing_unique_values.insert(field.clone(), found_values);
                 println!("[VALIDATE Task] Found {} existing values for field '{}'", existing_unique_values.get(field).map_or(0, |s| s.len()), field);
             }
         }


        // 6. Identify Conflicts & Prepare Data for Update
        let mut conflicting_rows_to_move: Vec<Map<String, Value>> = Vec::new();
        let mut ids_to_delete_from_valid: Vec<String> = Vec::new();

        println!("[VALIDATE Task] Identifying conflicting rows...");
        for row_map in valid_data_sqlite {
            let row_id = row_map.get("_id").cloned().flatten().unwrap_or_default();
             let mut conflict_errors: Vec<String> = Vec::new();

             // Check _id conflict
             if existing_ids.contains(&row_id) {
                 conflict_errors.push("Existing document with the same _id".to_string());
             }

             // Check unique fields
             for field in &unique_fields {
                 if let Some(Some(value)) = row_map.get(field) {
                    if !value.is_empty() { // Only check non-empty values
                         if let Some(existing_values) = existing_unique_values.get(field) {
                             if existing_values.contains(value) {
                                 conflict_errors.push(format!("Duplicate value for unique field '{}'", field));
                             }
                         }
                    }
                 }
             }

             // If conflicts found, prepare to move the row
             if !conflict_errors.is_empty() {
                 ids_to_delete_from_valid.push(row_id.clone());

                 // Convert HashMap<String, Option<String>> back to serde_json::Map<String, Value>
                 let mut json_map = Map::new();
                 for (key, opt_val) in row_map {
                     json_map.insert(key, opt_val.map_or(Value::Null, Value::String));
                 }

                 // Add errors (as a JSON array string for simplicity with existing structure)
                 json_map.insert("errors".to_string(), Value::String(serde_json::to_string(&conflict_errors)?));
                 conflicting_rows_to_move.push(json_map);
             }
        }
        let conflicts_found_count = conflicting_rows_to_move.len();
        println!("[VALIDATE Task] Identified {} conflicting rows", conflicts_found_count);


        // 7. Update SQLite Database (Transaction)
        println!("[VALIDATE Task] Starting SQLite transaction");
        let tx = conn.transaction()?;

        // Move conflicting rows to invalid_data
        if !conflicting_rows_to_move.is_empty() {
            println!("[VALIDATE Task] Moving {} rows to invalid_data", conflicts_found_count);
             // Get columns of invalid_data table to ensure compatibility
             let invalid_columns = get_table_columns(&tx, "invalid_data")?;
             insert_invalid_data_with_cols(&tx, &conflicting_rows_to_move, &invalid_columns)?;
        }

        // Delete conflicting rows from valid_data
        if !ids_to_delete_from_valid.is_empty() {
            println!("[VALIDATE Task] Deleting {} rows from valid_data", ids_to_delete_from_valid.len());
            let mut stmt_delete = tx.prepare("DELETE FROM valid_data WHERE \"_id\" = ?")?;
            for id in ids_to_delete_from_valid {
                stmt_delete.execute(params![id])?;
            }
        }

        println!("[VALIDATE Task] Committing SQLite transaction");
        tx.commit()?;


        // 8. Prepare Summary
        let remaining_valid_count = initial_valid_count - conflicts_found_count;
        Ok(ValidationSummary {
            validated_count: initial_valid_count,
            conflicts_found: conflicts_found_count,
            remaining_valid: remaining_valid_count,
        })

    }).await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Task join error: {}", e)))? // Outer task error
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Validation failed: {}", e)))?; // Inner logic error


    // 9. Return Response
     println!("[VALIDATE] Validation successful: {:?}", validation_result);
     Ok(Json(ApiResponse {
        success: true,
        data: Some(validation_result),
        error: None,
    }))
}