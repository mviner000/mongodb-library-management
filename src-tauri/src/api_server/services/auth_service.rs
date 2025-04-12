// src/api_server/services/auth_service.rs

use bcrypt;
use mongodb::bson::{doc, Document};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{warn, error, debug};

use crate::{
    session::SessionManager,
    mongodb_manager::MongoDbState,
};

// Login user service function
pub async fn login_user(
    mongodb_state: &Arc<Mutex<MongoDbState>>,
    session_manager: &Arc<Mutex<SessionManager>>,
    identifier: &str,
    password: &str,
) -> Result<String, String> {
    debug!("Attempting login process for: {}", identifier);
    let db = mongodb_state.lock().await.get_database().await.map_err(|e| {
        error!("Database connection error during login: {}", e);
        e.to_string()
    })?;
    
    let collection = db.collection::<Document>("users");
    let filter = doc! { "$or": [{ "email": identifier }, { "username": identifier }] };

    let user = collection.find_one(filter, None)
        .await
        .map_err(|e| {
            error!("Database query error: {}", e);
            format!("Database error: {}", e)
        })?
        .ok_or_else(|| {
            warn!("User not found: {}", identifier);
            "User not found".to_string()
        })?;

    let stored_hash = user.get_str("password")
        .map_err(|_| {
            warn!("Invalid password format for user: {}", identifier);
            "Invalid user data".to_string()
        })?;
    
    bcrypt::verify(password, stored_hash)
        .map_err(|e| {
            error!("Password verification error: {}", e);
            format!("Password verification failed: {}", e)
        })?
        .then_some(())
        .ok_or_else(|| {
            warn!("Password mismatch for: {}", identifier);
            "Invalid password".to_string()
        })?;

    let user_id = user.get_object_id("_id")
        .map_err(|_| {
            error!("Invalid user ID format for: {}", identifier);
            "Invalid user ID".to_string()
        })?
        .to_hex();

    session_manager.lock().await.create_session(&user_id)
        .await
        .map(|session| {
            debug!("Session created successfully for: {}", user_id);
            session.token
        })
        .map_err(|e| {
            error!("Session creation failed for {}: {}", user_id, e);
            format!("Session creation failed: {}", e)
        })
}

// Register user service function
pub async fn register_user(
    mongodb_state: &Arc<Mutex<MongoDbState>>,
    username: &str,
    email: &str,
    password: &str,
) -> Result<(), String> {
    debug!("Starting registration for: {}", username);
    let db = mongodb_state.lock().await.get_database().await.map_err(|e| {
        error!("Database connection error during registration: {}", e);
        e.to_string()
    })?;
    
    let collection = db.collection::<Document>("users");
    let existing = collection.find_one(
        doc! { "$or": [{ "email": email }, { "username": username }] }, 
        None
    )
    .await
    .map_err(|e| {
        error!("Database query error during registration check: {}", e);
        format!("Database error: {}", e)
    })?;

    if existing.is_some() {
        warn!("Duplicate registration attempt - Email: {}, Username: {}", email, username);
        return Err("Email or username already registered".into());
    }

    let hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| {
            error!("Password hashing error: {}", e);
            format!("Password hashing failed: {}", e)
        })?;

    let now = mongodb::bson::DateTime::now();
    let user = doc! {
        "username": username,
        "email": email,
        "password": hashed,
        "created_at": now,
        "updated_at": now
    };

    collection.insert_one(user, None)
        .await
        .map(|_| {
            debug!("User successfully registered: {}", username);
            Ok(())
        })
        .map_err(|e| {
            error!("User creation failed for {}: {}", username, e);
            format!("User creation failed: {}", e)
        })?
}
