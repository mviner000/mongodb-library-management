// src/auth.rs
use crate::session::SessionManager;
use mongodb::bson::{doc, Document};
use tauri::State;

#[tauri::command]
pub async fn login(
    identifier: String,  // Changed from email
    password: String,
    mongodb_state: State<'_, crate::mongodb_manager::MongoDbState>,
    session_manager: State<'_, SessionManager>,
) -> Result<String, String> {
    let db = mongodb_state.get_database().await?;
    let collection = db.collection::<Document>("users");

    // Search by either email or username
    let user = collection
        .find_one(
            doc! { 
                "$or": [
                    { "email": &identifier },
                    { "username": &identifier }
                ] 
            }, 
            None
        )
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "User not found".to_string())?;

    // Verify password
    let stored_hash = user.get_str("password").map_err(|_| "Invalid user data")?;
    bcrypt::verify(&password, stored_hash)
        .map_err(|e| e.to_string())?
        .then_some(())
        .ok_or_else(|| "Invalid password".to_string())?;

    let user_id = user
        .get_object_id("_id")
        .map_err(|_| "Invalid user ID")?
        .to_hex();

    let session = session_manager.create_session(&user_id).await
        .map_err(|e| format!("Failed to create session: {}", e))?;
    println!("Generated token for user {}: {}", user_id, session.token);
    Ok(session.token)
}

#[tauri::command]
pub async fn register(
    username: String,  // Add username parameter
    email: String,
    password: String,
    mongodb_state: State<'_, crate::mongodb_manager::MongoDbState>,
) -> Result<(), String> {
    println!("Registration attempt for: {}", email);
    
    let db = mongodb_state.get_database().await?;
    let collection = db.collection::<Document>("users");

    // Check for existing user by email or username
    let existing = collection.find_one(
        doc! { "$or": [ 
            { "email": &email }, 
            { "username": &username } 
        ]}, 
        None
    )
    .await
    .map_err(|e| e.to_string())?;
    
    if existing.is_some() {
        return Err("Email or username already registered".into());
    }

    // Hash password
    let hashed = bcrypt::hash(&password, bcrypt::DEFAULT_COST)
        .map_err(|e| e.to_string())?;

    // Create user document with all required fields
    let now = bson::DateTime::now();
    let user = doc! {
        "username": username,
        "email": email,
        "password": hashed,
        "created_at": now,
        // "updated_at": now  // updated_at field remains blank when creating a new document and is only set during updates
    };

    // Insert new user
    collection.insert_one(user, None)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn check_session(
    token: String,
    session_manager: State<'_, SessionManager>,
) -> Result<bool, String> {
    Ok(session_manager.validate_session(&token).await)
}