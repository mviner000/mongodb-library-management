// src/api_server/models/requests.rs

use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginPayload {
    pub identifier: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SessionCheckPayload {
    pub token: String,
}