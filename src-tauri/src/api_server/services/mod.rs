// src/api_server/services/mod.rs

// Re-export services
pub mod database_service;
pub mod auth_service;
pub mod schema_service;

pub use auth_service::{
    login_user,
    register_user,
};

pub use schema_service::{
    get_collection_schema_with_ui,
    update_ui_metadata,
};