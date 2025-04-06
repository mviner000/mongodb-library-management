// src/session.rs
use crate::mongodb_manager::MongoDbState;
use chrono::{DateTime, Utc};
use mongodb::{bson::{self, doc}, Collection};
use serde::{Deserialize, Serialize};
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
    mongodb_state: MongoDbState,
}

impl SessionManager {
    pub fn new(mongodb_state: MongoDbState) -> Self {
        Self { mongodb_state }
    }

    pub async fn create_session(&self, user_id: &str) -> Result<Session, String> {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + chrono::Duration::minutes(30);
        let created_at = Utc::now();

        let expires_at_millis = expires_at.timestamp_millis();
        let session_doc = doc! {
            "session_token": &token,
            "user_id": user_id,
            "expires_at": bson::DateTime::from_millis(expires_at_millis),
            "is_valid": true,
            "created_at": bson::DateTime::from_millis(created_at.timestamp_millis()),
            "label": "auth_login"
        };

        let db = self.mongodb_state.get_database().await?;
        let collection: Collection<bson::Document> = db.collection("sessions");
        
        collection.insert_one(session_doc, None)
            .await
            .map_err(|e| e.to_string())?;

        Ok(Session {
            user_id: user_id.to_string(),
            token,
            expires_at,
        })
    }

    pub async fn validate_session(&self, token: &str) -> bool {
        let db = match self.mongodb_state.get_database().await {
            Ok(db) => db,
            Err(_) => return false,
        };
        let collection: Collection<bson::Document> = db.collection("sessions");

        let now = Utc::now();
        let filter = doc! {
            "session_token": token,
            "is_valid": true,
            "expires_at": { "$gt": bson::DateTime::from_millis(now.timestamp_millis()) }
        };

        match collection.find_one(filter, None).await {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(_) => false,
        }
    }
}