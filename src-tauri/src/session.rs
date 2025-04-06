// src/session.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub user_id: String,
    pub token: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_session(&self, user_id: &str) -> Session {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + chrono::Duration::hours(9);
        
        let session = Session {
            user_id: user_id.to_string(),
            token: token.clone(),
            expires_at,
        };

        self.sessions.lock().await.insert(token.clone(), session.clone());
        session
    }

    pub async fn validate_session(&self, token: &str) -> bool {
        let sessions = self.sessions.lock().await;
        match sessions.get(token) {
            Some(session) => session.expires_at > Utc::now(),
            None => false,
        }
    }

    pub async fn cleanup_expired(&self) {
        let mut sessions = self.sessions.lock().await;
        sessions.retain(|_, session| session.expires_at > Utc::now());
    }
}