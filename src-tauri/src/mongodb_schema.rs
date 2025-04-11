// src/mongodb_schema.rs
use mongodb::{
    options::IndexOptions,
    Database,
    IndexModel,
};
use std::time::Duration;
use mongodb::bson::{doc, Document};
use anyhow::Result;
use crate::lib_mongodb_schema;


pub async fn initialize_database(db: &Database) -> Result<()> {
    // Create only essential collections by default
    create_users_collection(db).await?;
    create_sessions_collection(db).await?;
    create_ui_metadata_collection(db).await?;

    // Note: library-specific collections are now moved to lib_mongodb_schema.rs
    // and can be created separately when needed

    Ok(())
}

// Helper function to get archive properties schema to be reused - made public
pub fn get_archive_properties() -> Document {
    doc! {
        "is_archive": { 
            "bsonType": "bool", 
            "description": "Flag indicating if the document is archived (true) or active (false)" 
        },
        "archive_history": {
            "bsonType": "array",
            "description": "Log of archive and recovery actions",
            "items": {
                "bsonType": "object",
                "required": ["action", "user_id", "timestamp"],
                "properties": {
                    "action": { 
                        "bsonType": "string", 
                        "enum": ["archive", "recover"], 
                        "description": "The action performed" 
                    },
                    "user_id": { 
                        "bsonType": "objectId", 
                        "description": "REF:users | ID of the user performing the action" 
                    },
                    "timestamp": { 
                        "bsonType": "date", 
                        "description": "Timestamp of the action" 
                    }
                }
            }
        }
    }
}

// Helper function to create archive index - made public
pub fn create_archive_index() -> IndexModel {
    IndexModel::builder()
        .keys(doc! { "is_archive": 1 })
        .build()
}

// Helper function to merge document properties with archive properties - made public
pub fn merge_with_archive_properties(properties: Document) -> Document {
    let mut merged = properties;
    for (key, value) in get_archive_properties() {
        merged.insert(key, value);
    }
    merged
}

async fn create_users_collection(db: &Database) -> Result<()> {
    let collection = db.collection::<Document>("users");

    // Create a unique index on "username"
    let username_index = IndexModel::builder()
        .keys(doc! { "username": 1 })
        .options(Some(mongodb::options::IndexOptions::builder()
            .unique(true)
            .build()))
        .build();

    // Create a unique index on "email"
    let email_index = IndexModel::builder()
        .keys(doc! { "email": 1 })
        .options(Some(mongodb::options::IndexOptions::builder()
            .unique(true)
            .build()))
        .build();

    // Add archive index
    let archive_index = create_archive_index();

    collection.create_index(username_index, None).await?;
    collection.create_index(email_index, None).await?;
    collection.create_index(archive_index, None).await?;

    // Define base properties for this collection
    let base_properties = doc! {
        "username": {
            "bsonType": "string",
            "description": "Username (required, unique)"
        },
        "email": {
            "bsonType": "string",
            "pattern": "^[\\w.-]+@[\\w.-]+\\.[a-zA-Z]{2,}$",
            "description": "Email (required, unique, must match pattern)"
        },
        "password": {
            "bsonType": "string",
            "description": "Password (hashed, required)"
        },
        "created_at": {
            "bsonType": "date",
            "description": "Creation timestamp (required)"
        },
        "updated_at": {
            "bsonType": "date", 
            "description": "Last update timestamp (required)"
        }
    };
    
    // Merge with archive properties
    let properties = merge_with_archive_properties(base_properties);

    // Apply validator schema using collMod
    db.run_command(
        doc! {
            "collMod": "users",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["username", "email", "password", "created_at", "updated_at"],
                    "properties": properties
                }
            },
            "validationLevel": "moderate",
            "validationAction": "error"
        },
        None
    ).await?;

    Ok(())
}

async fn create_sessions_collection(db: &Database) -> Result<()> {
    let collection = db.collection::<Document>("sessions");
    
    // Create indexes
    let indexes = vec![
        IndexModel::builder()
            .keys(doc! { "session_token": 1 })
            .options(Some(IndexOptions::builder()
                .unique(true)
                .build()))
            .build(),
        IndexModel::builder()
            .keys(doc! { "user_id": 1 })
            .build(),
        IndexModel::builder()
            .keys(doc! { "expires_at": 1 })
            .options(Some(IndexOptions::builder()
                .expire_after(Some(Duration::from_secs(0))) // TTL index (expire after 0 seconds)
                .build()))
            .build(),
        // Add archive index
        create_archive_index(),
    ];

    collection.create_indexes(indexes, None).await?;
    
    // Define base properties for this collection
    let base_properties = doc! {
        "session_token": { 
            "bsonType": "string", 
            "description": "Unique session token (required)" 
        },
        "user_id": { 
            "bsonType": "string", 
            "description": "REF:users | Reference to user (required)" 
        },
        "expires_at": { 
            "bsonType": "date", 
            "description": "Expiration timestamp (required)" 
        },
        "is_valid": { 
            "bsonType": "bool", 
            "description": "Validity status (required)" 
        },
        "created_at": { 
            "bsonType": "date", 
            "description": "Creation timestamp (required)" 
        },
        "label": {
            "bsonType": "string",
            "description": "Session label (required)"
        }
    };
    
    // Merge with archive properties
    let properties = merge_with_archive_properties(base_properties);
    
    // Apply validator schema using collMod
    db.run_command(
        doc! {
            "collMod": "sessions",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["session_token", "user_id", "expires_at", "is_valid", "created_at", "label"],
                    "properties": properties
                }
            },
            "validationLevel": "moderate",
            "validationAction": "error"
        },
        None
    ).await?;
    
    Ok(())
}

// Keep the ui_metadata collection as is - no changes per requirements
async fn create_ui_metadata_collection(db: &Database) -> Result<()> {
    let collection = db.collection::<Document>("ui_metadata");
    
    // Create indexes for faster lookups
    let indexes = vec![
        IndexModel::builder()
            .keys(doc! { "collection": 1, "user_id": 1 })
            .options(Some(IndexOptions::builder()
                .unique(true)
                .build()))
            .build(),
    ];

    collection.create_indexes(indexes, None).await?;
    
    // Apply validator schema using collMod
    db.run_command(
        doc! {
            "collMod": "ui_metadata",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["collection", "ui", "created_at", "updated_at"],
                    "properties": {
                        "collection": { "bsonType": "string", "description": "Collection name" },
                        "ui": { 
                            "bsonType": "object", 
                            "properties": {
                                "columnWidths": { "bsonType": "object", "description": "Width for each column" },
                                "columnOrder": { "bsonType": "array", "description": "Order of columns in the view" },
                                "hiddenColumns": { "bsonType": "array", "description": "List of hidden columns" },
                                "sortSettings": { 
                                    "bsonType": "object", 
                                    "properties": {
                                        "field": { "bsonType": "string" },
                                        "direction": { "bsonType": "string", "enum": ["asc", "desc"] }
                                    }
                                },
                                "filterSettings": { "bsonType": "object", "description": "Filter configurations" },
                            }
                        },
                        "created_at": { "bsonType": "date", "description": "Creation timestamp" },
                        "updated_at": { "bsonType": "date", "description": "Last update timestamp" }
                    }
                }
            },
            "validationLevel": "moderate",
            "validationAction": "error"
        },
        None
    ).await?;

    // Insert default global settings for each collection
    let default_settings = create_default_ui_settings();
    
    for setting in default_settings {
        // Use upsert with $setOnInsert to create only if not exists
        let filter = doc! {
            "collection": setting.get("collection").unwrap(),
            "user_id": { "$exists": false }  // Global defaults have no user_id
        };
        
        collection.update_one(
            filter,
            doc! { "$setOnInsert": setting },  // Only apply settings when inserting a new document
            mongodb::options::UpdateOptions::builder()
                .upsert(true)
                .build(),
        ).await?;
    }

    Ok(())
}

// metadata for collections start here
// Keep the UI metadata helper functions as is, but update collection list to include only essential collections
const DEFAULT_COLUMN_WIDTH: i32 = 200;

fn create_default_ui_settings() -> Vec<Document> {
    // Include only essential collections
    let collections = vec![
        "users",
        "sessions",
    ];
    
    let now = mongodb::bson::DateTime::now();
    
    collections.iter().map(|&collection_name| {
        // Get field names for this collection to create column width settings
        let column_widths = get_default_column_widths(collection_name);

        // Ensure the backend only stores data column widths (excluding the _actions and _row_number columns)
        let mut filtered_column_widths = Document::new();
        for (key, value) in column_widths.iter() {
            if !["_row_number", "_actions"].contains(&key.as_str()) {
                filtered_column_widths.insert(key.clone(), value.clone());
            }
        }

        // Create column order based on fields (same order as in schema)
        let column_order = mongodb::bson::to_bson(&column_widths.keys().collect::<Vec<_>>())
            .unwrap_or(mongodb::bson::Bson::Array(Vec::new()));
        
        doc! {
            "collection": collection_name,
            "ui": {
                "columnWidths": column_widths,
                "columnOrder": column_order,
                "hiddenColumns": [],
                "sortSettings": {
                    "field": get_default_sort_field(collection_name),
                    "direction": "asc"
                },
                "filterSettings": {},
            },
            "created_at": now,
            "updated_at": now
        }
    }).collect()
}

fn get_default_column_widths(collection_name: &str) -> Document {
    match collection_name {
        "users" => doc! {
            "username": DEFAULT_COLUMN_WIDTH, 
            "email": DEFAULT_COLUMN_WIDTH, 
            "password": DEFAULT_COLUMN_WIDTH,
            "is_archive": DEFAULT_COLUMN_WIDTH,
            "created_at": DEFAULT_COLUMN_WIDTH, 
            "updated_at": DEFAULT_COLUMN_WIDTH
        },
        "sessions" => doc! {
            "session_token": DEFAULT_COLUMN_WIDTH, 
            "user_id": DEFAULT_COLUMN_WIDTH, 
            "expires_at": DEFAULT_COLUMN_WIDTH,
            "is_valid": DEFAULT_COLUMN_WIDTH,
            "is_archive": DEFAULT_COLUMN_WIDTH,
            "created_at": DEFAULT_COLUMN_WIDTH, 
            "label": DEFAULT_COLUMN_WIDTH
        },
        _ => {
            let field_names = get_field_names_for_collection(collection_name);
            let mut widths = Document::new();
            for field in field_names {
                widths.insert(field, DEFAULT_COLUMN_WIDTH);
            }
            widths
        }
    }
}

fn get_default_sort_field(collection_name: &str) -> &str {
    match collection_name {
        "users" => "username",
        "sessions" => "created_at",
        _ => "created_at",
    }
}

// Helper function to get field names for a collection - reduced to essential collections
fn get_field_names_for_collection(collection_name: &str) -> Vec<&str> {
    // Add is_archive to all collection field lists
    let mut fields = match collection_name {
        "users" => vec![
            "username", "email", "password", "created_at", "updated_at"
        ],
        "sessions" => vec![
            "session_token", "user_id", "expires_at", "is_valid", "created_at", "label"
        ],
        _ => vec!["created_at", "updated_at"] // Fallback for unknown collections
    };
    
    // Add is_archive field to each collection's fields
    fields.push("is_archive");
    fields
}
// metadata for collections ends here

// Public function to initialize all library collections if needed
pub async fn initialize_library_collections(db: &Database) -> Result<()> {
    // Call the library-specific initialization
    lib_mongodb_schema::initialize_all_library_collections(db).await?;
    Ok(())
}