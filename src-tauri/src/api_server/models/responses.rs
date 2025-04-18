// src/api_server/models/responses.rs

use axum::{http::StatusCode, Json};
use mongodb::bson::Document;
use serde::{Deserialize, Serialize};

// Document response types
#[cfg_attr(debug_assertions, derive(Debug))] // Only in debug builds
#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct InsertResponse {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateResponse {
    pub success: bool,
    pub modified_count: u64,
    pub document: Option<Document>, 
}

#[derive(Serialize, Deserialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub deleted_count: u64,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
}

#[derive(Serialize)]
pub struct SessionCheckResponse {
    pub valid: bool,
}

// Helper for creating error responses
pub fn error_response<T: Serialize>(status: StatusCode, message: String) -> (StatusCode, Json<ApiResponse<T>>) {
    (status, Json(ApiResponse {
        success: false,
        data: None,
        error: Some(message),
    }))
}