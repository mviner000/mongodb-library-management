// src/api_server/handlers/auth_handlers.rs

use axum::{
    http::StatusCode,
    Json,
    extract::State,
    response::IntoResponse,
};
use axum_extra::{
    headers::{Authorization, authorization::Bearer},
    TypedHeader,
};
use mongodb::bson::{doc, Document};
use mongodb::bson::oid::ObjectId;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error, debug};

use crate::api_server::services::{login_user, register_user};
use crate::api_server::state::ApiServerState;
use crate::api_server::models::{
    LoginPayload, RegisterPayload, SessionCheckPayload,
    ApiResponse, LoginResponse, SessionCheckResponse, UserResponse,
    error_response
};

// Auth handlers

pub async fn auth_login_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    info!("Login attempt for identifier: {}", payload.identifier);
    let state = state.lock().await;
    let mongodb_state = &state.mongodb_state;
    let session_manager = &state.session_manager;

    match login_user(mongodb_state, session_manager, &payload.identifier, &payload.password).await {
        Ok(token) => {
            info!("Successful login for identifier: {}", payload.identifier);
            (StatusCode::OK, Json(ApiResponse {
                success: true,
                data: Some(LoginResponse { token }),
                error: None,
            }))
        },
        Err(e) => {
            error!("Login failed for {}: {}", payload.identifier, e);
            error_response::<LoginResponse>(StatusCode::UNAUTHORIZED, e)
        },
    }
}

pub async fn auth_get_me_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    let token = auth.token();
    // Log token snippet for tracing (first 6 chars only for security)
    let token_snippet = token.chars().take(6).collect::<String>();
    info!("GET /api/auth/me request received with token: {}...", token_snippet);
    
    let state = state.lock().await;
    let session_manager = &state.session_manager;
    
    // Validate session with improved logging
    debug!("Validating session token: {}...", token_snippet);
    let valid = session_manager.lock().await.validate_session(token).await;
    info!("Session validation result for token {}...: {}", token_snippet, valid);
    
    if !valid {
        error!("Invalid session token: {}...", token_snippet);
        return error_response::<UserResponse>(StatusCode::UNAUTHORIZED, "Invalid session".into());
    }

    // Get user data with improved logging
    debug!("Getting user ID from session token: {}...", token_snippet);
    let user_id = match session_manager.lock().await.get_user_id(token).await {
        Some(id) => {
            // Convert string ID to ObjectId
            match ObjectId::parse_str(&id) {
                Ok(oid) => oid,
                Err(e) => {
                    error!("Invalid user ID format: {}", e);
                    return error_response::<UserResponse>(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Invalid user ID format".into()
                    );
                }
            }
        },
        None => {
            error!("Session expired for token: {}...", token_snippet);
            return error_response::<UserResponse>(StatusCode::UNAUTHORIZED, "Session expired".into());
        }
    };

    debug!("Connecting to database to fetch user details");
    let mongodb_result = state.mongodb_state.lock().await.get_database().await;
    let db = match mongodb_result {
        Ok(db) => db,
        Err(e) => {
            error!("Database connection error: {}", e);
            return error_response::<UserResponse>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }
    };

    info!("Looking up user with ID: {} in database", &user_id);
    let collection = db.collection::<Document>("users");
    
    let user_result = collection.find_one(doc! { "_id": user_id }, None).await;
    
    let user = match user_result {
        Ok(Some(user)) => {
            info!("User found for ID: {}", &user_id);
            user
        },
        Ok(None) => {
            error!("User with ID: {} not found in database", &user_id);
            return error_response::<UserResponse>(StatusCode::NOT_FOUND, "User not found".into());
        },
        Err(e) => {
            error!("Database error while fetching user {}: {}", &user_id, e);
            return error_response::<UserResponse>(StatusCode::INTERNAL_SERVER_ERROR, e.to_string());
        }
    };

    // Log user fields for debugging
    debug!(
        "User data retrieved - Username: {}, Email: {}", 
        user.get_str("username").unwrap_or_default(),
        user.get_str("email").unwrap_or_default()
    );

    info!("Successfully returning user data for ID: {}", user_id);
    (StatusCode::OK, Json(ApiResponse {
        success: true,
        data: Some(UserResponse {
            id: user_id.to_string(),     // Convert ObjectId to string and include it
            username: user.get_str("username").unwrap_or_default().to_string(),
            email: user.get_str("email").unwrap_or_default().to_string(),
        }),
        error: None,
    }))
}

pub async fn auth_register_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    info!("Registration attempt - Username: {}, Email: {}", payload.username, payload.email);
    let state = state.lock().await;
    let mongodb_state = &state.mongodb_state;
    
    match register_user(mongodb_state, &payload.username, &payload.email, &payload.password).await {
        Ok(()) => {
            info!("Successful registration for username: {}", payload.username);
            (StatusCode::CREATED, Json(ApiResponse {
                success: true,
                data: None,
                error: None,
            }))
        },
        Err(e) => {
            error!("Registration failed for {}: {}", payload.username, e);
            error_response::<()>(StatusCode::BAD_REQUEST, e)
        },
    }
}

pub async fn auth_check_session_handler(
    State(state): State<Arc<Mutex<ApiServerState>>>,
    Json(payload): Json<SessionCheckPayload>,
) -> impl IntoResponse {
    let token_snippet = payload.token.chars().take(6).collect::<String>();
    debug!("Session check request for token: {}...", token_snippet);
    let state = state.lock().await;
    let session_manager = &state.session_manager;
    
    let valid = session_manager.lock().await.validate_session(&payload.token).await;
    info!("Session validation result for {}...: {}", token_snippet, valid);
    
    (StatusCode::OK, Json(ApiResponse {
        success: true,
        data: Some(SessionCheckResponse { valid }),
        error: None,
    }))
}
