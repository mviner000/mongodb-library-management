// src/api_server/state.rs
use crate::mongodb_manager::MongoDbState;
use crate::session::SessionManager;
use std::sync::Arc;
use tokio::sync::Mutex;

// API server state
pub struct ApiServerState {
    pub mongodb_state: Arc<Mutex<MongoDbState>>,
    pub session_manager: Arc<Mutex<SessionManager>>,
    pub server_handle: Option<tokio::task::JoinHandle<()>>,
    pub routes: Vec<String>, // Added routes field
}

impl ApiServerState {
    pub fn new(mongodb_state: MongoDbState, session_manager: SessionManager) -> Self {
        Self {
            mongodb_state: Arc::new(Mutex::new(mongodb_state)),
            session_manager: Arc::new(Mutex::new(session_manager)),
            server_handle: None,
            routes: Vec::new(), // Initialize with empty vector
        }
    }
}