// src/api_server/state.rs
use crate::mongodb_manager::MongoDbState;
use crate::session::SessionManager;
use std::sync::Arc;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::Mutex as AsyncMutex;

// API server state
pub struct ApiServerState {
    pub mongodb_state: Arc<AsyncMutex<MongoDbState>>,
    pub session_manager: Arc<AsyncMutex<SessionManager>>,
    pub server_handle: Option<tokio::task::JoinHandle<()>>,
    pub routes: Vec<String>,
    pub temp_dirs: Arc<AsyncMutex<HashMap<String, PathBuf>>>,
}

impl ApiServerState {
    pub fn new(mongodb_state: MongoDbState, session_manager: SessionManager) -> Self {
        Self {
            mongodb_state: Arc::new(AsyncMutex::new(mongodb_state)),
            session_manager: Arc::new(AsyncMutex::new(session_manager)),
            server_handle: None,
            routes: Vec::new(),
            temp_dirs: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }
}