// src/api_server/routes.rs

use axum::{
    routing::{get, post, put, delete},
    http::Method,
    Router,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

use crate::api_server::{
    state::ApiServerState,
    handlers::{
        auth_handlers::{
            auth_login_handler,
            auth_register_handler,
            auth_check_session_handler,
        },
        collection_handlers::{
            list_collections_handler,
            get_collection_schema_handler,
            update_ui_metadata_handler,
        },
        document_handlers::{
            find_documents_handler,
            insert_document_handler,
            update_document_handler,
            delete_document_handler,
        },
        system_handlers::{
            health_check_handler,
            initialize_library_collections_handler,
        },
    },
};

// Create the API router
pub fn create_api_router() -> Router<Arc<Mutex<ApiServerState>>> {
    // Setup CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin(Any);
    
    Router::new()
        // Collection routes
        .route("/collections", get(list_collections_handler))
        .route("/collections/:collection_name/schema", get(get_collection_schema_handler))
        .route("/collections/:collection_name/ui-metadata", put(update_ui_metadata_handler))
        
        // Document routes
        .route("/collections/:collection_name/documents", get(find_documents_handler))
        .route("/collections/:collection_name/documents", post(insert_document_handler))
        .route("/collections/:collection_name/documents/:id", put(update_document_handler))
        .route("/collections/:collection_name/documents/:id", delete(delete_document_handler))
        
        // Auth routes
        .route("/api/auth/login", post(auth_login_handler))
        .route("/api/auth/register", post(auth_register_handler))
        .route("/api/auth/check-session", post(auth_check_session_handler))
        
        // System routes
        .route("/api/initialize-library-collections", post(initialize_library_collections_handler))
        .route("/api/health", get(health_check_handler))
        
        // Apply CORS middleware
        .layer(cors)
}