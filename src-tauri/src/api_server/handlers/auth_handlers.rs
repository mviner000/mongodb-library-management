// src/api_server/handlers/auth_handlers.rs

use axum::{
    http::StatusCode,
    Json,
    extract::State,
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error, debug};

use crate::api_server::services::{login_user, register_user};
use crate::api_server::state::ApiServerState;
use crate::api_server::models::{
    LoginPayload, RegisterPayload, SessionCheckPayload,
    ApiResponse, LoginResponse, SessionCheckResponse,
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
