// src/api_server/routes.rs

use axum::{
    routing::{get, post, put, delete},
    http::Method,
    Router,
    middleware::map_request,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use crate::api_server::{
    state::ApiServerState,
    handlers::{
        auth_handlers::{
            auth_login_handler,
            auth_get_me_handler,
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
            find_empty_or_recovered_documents_handler,
            find_empty_archive_history_handler,
            find_archived_documents_handler,
            find_recovered_documents_handler,
            find_pinned_documents_handler,
            insert_document_handler,
            update_document_handler,
            delete_document_handler,
            batch_delete_documents_handler,
            archive_document_handler,
            batch_archive_documents_handler,
            recover_document_handler,
            batch_recover_documents_handler,
            pin_document_handler,
            unpin_document_handler,
            download_collection_csv_handler,

        },
        system_handlers::{
            health_check_handler,
            initialize_library_collections_handler,
        },
        csv_temp_handlers::{
            load_csv_temp,
            save_csv_temp,
            delete_csv_temp,
            validate_csv_temp_handler
        },
        csv_download_handler::{
            download_temp_csv,
            download_temp_csv_post
        },
        csv_import_handler::import_valid_csv_data_handler,
    },
};
use axum::extract::Request;

// Add middleware to log all route calls
async fn log_request(request: Request) -> Request {
    println!("Route called: {} {}", request.method(), request.uri().path());
    request
}

// Create the API router and return both the router and a list of routes
pub fn create_api_router() -> (Router<Arc<Mutex<ApiServerState>>>, Vec<String>) {
    let mut routes = Vec::new();
    let mut router = Router::new();
    
    // Setup CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin(Any);
    
    // Macro to add routes and track them
    macro_rules! add_route {
        ($method:expr, $path:expr, $handler:expr) => {
            router = match $method {
                Method::GET => router.route($path, get($handler)),
                Method::POST => router.route($path, post($handler)),
                Method::PUT => router.route($path, put($handler)),
                Method::DELETE => router.route($path, delete($handler)),
                _ => panic!("Unsupported method: {}. Update the router implementation.", $method),
            };
            routes.push(format!("{} {}", $method, $path));
        };
    }
    
    // Collection routes
    add_route!(Method::GET, "/collections", list_collections_handler);
    add_route!(Method::GET, "/collections/:collection_name/schema", get_collection_schema_handler);
    add_route!(Method::PUT, "/collections/:collection_name/ui-metadata", update_ui_metadata_handler);
    
    // route for temp sqlite3 csv temporary storage
    add_route!(Method::POST, "/api/csv-temp/:collection", save_csv_temp);
    add_route!(Method::GET, "/api/csv-temp/:collection", load_csv_temp);
    add_route!(Method::DELETE, "/api/csv-temp/:collection", delete_csv_temp);

    add_route!(Method::POST, "/api/csv-validate/:collection", validate_csv_temp_handler);

    // Download csv from sqlite routes
    add_route!(Method::GET, "/api/csv-temp/:collection/download-csv", download_temp_csv);
    add_route!(Method::POST, "/api/csv-temp/:collection/download-csv", download_temp_csv_post);


    // Import validated CSV into MongoDB
    add_route!(Method::POST,   "/api/csv-import/:collection", import_valid_csv_data_handler);

    // Document routes
    add_route!(Method::GET, "/collections/:collection_name/documents", find_documents_handler);
    add_route!(
        Method::GET, 
        "/collections/:collection_name/empty-or-recovered", 
        find_empty_or_recovered_documents_handler
    );
    add_route!(
        Method::GET, 
        "/collections/:collection_name/empty-archive-history", 
        find_empty_archive_history_handler
    );
    add_route!(Method::GET, "/collections/:collection_name/archives", find_archived_documents_handler);
    add_route!(Method::GET, "/collections/:collection_name/recoveries", find_recovered_documents_handler);
    add_route!(Method::GET, "/collections/:collection_name/pins", find_pinned_documents_handler);
    add_route!(Method::POST, "/collections/:collection_name/documents", insert_document_handler);
    add_route!(Method::PUT, "/collections/:collection_name/documents/:id", update_document_handler);
    add_route!(Method::DELETE, "/collections/:collection_name/documents/:id", delete_document_handler);
    add_route!(
        Method::POST, 
        "/collections/:collection_name/documents/batch-delete", 
        batch_delete_documents_handler
    );
    add_route!(Method::PUT, "/collections/:collection_name/documents/:id/archive", archive_document_handler);
    add_route!(
        Method::POST, 
        "/collections/:collection_name/documents/batch-archive", 
        batch_archive_documents_handler
    );
    add_route!(Method::PUT, "/collections/:collection_name/documents/:id/recover", recover_document_handler);
    add_route!(
        Method::POST, 
        "/collections/:collection_name/documents/batch-recover", 
        batch_recover_documents_handler
    );
    add_route!(Method::PUT, "/collections/:collection_name/documents/:id/pin", pin_document_handler);
    add_route!(Method::PUT, "/collections/:collection_name/documents/:id/unpin", unpin_document_handler);

    add_route!(Method::GET, "/collections/:collection_name/download-csv", download_collection_csv_handler);
    
    // Auth routes
    add_route!(Method::POST, "/api/auth/login", auth_login_handler);
    add_route!(Method::GET, "/api/auth/me", auth_get_me_handler);
    add_route!(Method::POST, "/api/auth/register", auth_register_handler);
    add_route!(Method::POST, "/api/auth/check-session", auth_check_session_handler);
    // System routes
    add_route!(Method::POST, "/api/initialize-library-collections", initialize_library_collections_handler);
    add_route!(Method::GET, "/api/health", health_check_handler);
    
    // Print startup routes information
    println!("Available routes:");
    for route in &routes {
        println!("  {}", route);
    }
    
    // Apply request logging middleware
    router = router.layer(map_request(log_request));
    
    // Apply CORS middleware
    router = router.layer(cors);
    
    (router, routes)
}