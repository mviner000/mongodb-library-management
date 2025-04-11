// src/lib_mongodb_schema.rs

use mongodb::{
    options::IndexOptions,
    Database,
    IndexModel,
};
use mongodb::bson::{doc, Document};
use anyhow::Result;
use crate::mongodb_schema::{create_archive_index, merge_with_archive_properties};

// Library-specific collection for school accounts
pub async fn create_lib_school_accounts_collection(db: &Database) -> Result<()> {
    let collection = db.collection::<Document>("school_accounts");
    
    // Create indexes
    let indexes = vec![
        IndexModel::builder()
            .keys(doc! { "school_name": 1 })
            .options(Some(IndexOptions::builder()
                .unique(true)
                .build()))
            .build(),
        create_archive_index(),
    ];
    
    collection.create_indexes(indexes, None).await?;
    
    // Define base properties for this collection
    let base_properties = doc! {
        "school_name": { 
            "bsonType": "string", 
            "description": "School name (required, unique)" 
        },
        "address": { 
            "bsonType": "string", 
            "description": "School address (required)" 
        },
        "contact_person": { 
            "bsonType": "string", 
            "description": "Contact person name (required)" 
        },
        "contact_email": { 
            "bsonType": "string", 
            "description": "Contact email (required)" 
        },
        "contact_phone": { 
            "bsonType": "string", 
            "description": "Contact phone number (required)" 
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
            "collMod": "school_accounts",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["school_name", "address", "contact_person", "contact_email", "contact_phone", "created_at", "updated_at"],
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

// Library-specific collection for attendance records
pub async fn create_lib_attendance_collection(db: &Database) -> Result<()> {
    let collection = db.collection::<Document>("attendance");
    
    // Create indexes
    let indexes = vec![
        IndexModel::builder()
            .keys(doc! { "student_id": 1, "date": 1 })
            .options(Some(IndexOptions::builder()
                .unique(true)
                .build()))
            .build(),
        IndexModel::builder()
            .keys(doc! { "school_id": 1 })
            .build(),
        create_archive_index(),
    ];
    
    collection.create_indexes(indexes, None).await?;
    
    // Define base properties for this collection
    let base_properties = doc! {
        "student_id": { 
            "bsonType": "objectId", 
            "description": "Student ID reference (required)" 
        },
        "school_id": { 
            "bsonType": "objectId", 
            "description": "REF:school_accounts | School ID reference (required)" 
        },
        "date": { 
            "bsonType": "date", 
            "description": "Attendance date (required)" 
        },
        "status": { 
            "bsonType": "string", 
            "enum": ["present", "absent", "late", "excused"], 
            "description": "Attendance status (required)" 
        },
        "notes": { 
            "bsonType": "string", 
            "description": "Additional notes" 
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
            "collMod": "attendance",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["student_id", "school_id", "date", "status", "created_at", "updated_at"],
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

// Library-specific collection for purposes
pub async fn create_lib_purposes_collection(db: &Database) -> Result<()> {
    let collection = db.collection::<Document>("purposes");
    
    // Create indexes
    let indexes = vec![
        IndexModel::builder()
            .keys(doc! { "name": 1 })
            .options(Some(IndexOptions::builder()
                .unique(true)
                .build()))
            .build(),
        create_archive_index(),
    ];
    
    collection.create_indexes(indexes, None).await?;
    
    // Define base properties for this collection
    let base_properties = doc! {
        "name": { 
            "bsonType": "string", 
            "description": "Purpose name (required, unique)" 
        },
        "description": { 
            "bsonType": "string", 
            "description": "Purpose description (required)" 
        },
        "category": { 
            "bsonType": "string", 
            "enum": ["academic", "administrative", "extracurricular", "other"], 
            "description": "Purpose category (required)" 
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
            "collMod": "purposes",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["name", "description", "category", "created_at", "updated_at"],
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

// Library-specific collection for semesters
pub async fn create_lib_semesters_collection(db: &Database) -> Result<()> {
    let collection = db.collection::<Document>("semesters");
    
    // Create indexes
    let indexes = vec![
        IndexModel::builder()
            .keys(doc! { "name": 1, "school_year": 1 })
            .options(Some(IndexOptions::builder()
                .unique(true)
                .build()))
            .build(),
        create_archive_index(),
    ];
    
    collection.create_indexes(indexes, None).await?;
    
    // Define base properties for this collection
    let base_properties = doc! {
        "name": { 
            "bsonType": "string", 
            "description": "Semester name (required, unique with school_year)" 
        },
        "school_year": { 
            "bsonType": "string", 
            "description": "School year (required, unique with name)" 
        },
        "start_date": { 
            "bsonType": "date", 
            "description": "Start date (required)" 
        },
        "end_date": { 
            "bsonType": "date", 
            "description": "End date (required)" 
        },
        "is_active": { 
            "bsonType": "bool", 
            "description": "Whether the semester is currently active (required)" 
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
            "collMod": "semesters",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["name", "school_year", "start_date", "end_date", "is_active", "created_at", "updated_at"],
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

// Library-specific collection for settings styles
pub async fn create_lib_settings_styles_collection(db: &Database) -> Result<()> {
    let collection = db.collection::<Document>("settings_styles");
    
    // Create indexes
    let indexes = vec![
        IndexModel::builder()
            .keys(doc! { "name": 1 })
            .options(Some(IndexOptions::builder()
                .unique(true)
                .build()))
            .build(),
        create_archive_index(),
    ];
    
    collection.create_indexes(indexes, None).await?;
    
    // Define base properties for this collection
    let base_properties = doc! {
        "name": { 
            "bsonType": "string", 
            "description": "Style name (required, unique)" 
        },
        "description": { 
            "bsonType": "string", 
            "description": "Style description" 
        },
        "theme": { 
            "bsonType": "string", 
            "enum": ["light", "dark", "system", "custom"], 
            "description": "UI theme setting (required)" 
        },
        "primary_color": { 
            "bsonType": "string", 
            "description": "Primary color hex code (required)" 
        },
        "secondary_color": { 
            "bsonType": "string", 
            "description": "Secondary color hex code (required)" 
        },
        "font_family": { 
            "bsonType": "string", 
            "description": "Font family (required)" 
        },
        "is_default": { 
            "bsonType": "bool", 
            "description": "Whether this is the default style (required)" 
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
            "collMod": "settings_styles",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["name", "theme", "primary_color", "secondary_color", "font_family", "is_default", "created_at", "updated_at"],
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

// Additional helper function for UI metadata for library collections
pub fn get_default_lib_column_widths(collection_name: &str) -> Document {
    let default_column_width = 200;
    
    match collection_name {
        "school_accounts" => doc! {
            "school_name": default_column_width,
            "address": default_column_width,
            "contact_person": default_column_width,
            "contact_email": default_column_width,
            "contact_phone": default_column_width,
            "is_archive": default_column_width,
            "created_at": default_column_width,
            "updated_at": default_column_width
        },
        "attendance" => doc! {
            "student_id": default_column_width,
            "school_id": default_column_width,
            "date": default_column_width,
            "status": default_column_width,
            "notes": default_column_width,
            "is_archive": default_column_width,
            "created_at": default_column_width,
            "updated_at": default_column_width
        },
        "purposes" => doc! {
            "name": default_column_width,
            "description": default_column_width,
            "category": default_column_width,
            "is_archive": default_column_width,
            "created_at": default_column_width,
            "updated_at": default_column_width
        },
        "semesters" => doc! {
            "name": default_column_width,
            "school_year": default_column_width,
            "start_date": default_column_width,
            "end_date": default_column_width,
            "is_active": default_column_width,
            "is_archive": default_column_width,
            "created_at": default_column_width,
            "updated_at": default_column_width
        },
        "settings_styles" => doc! {
            "name": default_column_width,
            "description": default_column_width,
            "theme": default_column_width,
            "primary_color": default_column_width,
            "secondary_color": default_column_width,
            "font_family": default_column_width,
            "is_default": default_column_width,
            "is_archive": default_column_width,
            "created_at": default_column_width,
            "updated_at": default_column_width
        },
        _ => doc! {}
    }
}

// Helper function to get default sort field for library collections
pub fn get_default_lib_sort_field(collection_name: &str) -> &str {
    match collection_name {
        "school_accounts" => "school_name",
        "attendance" => "date",
        "purposes" => "name",
        "semesters" => "school_year",
        "settings_styles" => "name",
        _ => "created_at",
    }
}

// Initialize all library collections
pub async fn initialize_all_library_collections(db: &Database) -> Result<()> {
    create_lib_school_accounts_collection(db).await?;
    create_lib_attendance_collection(db).await?;
    create_lib_purposes_collection(db).await?;
    create_lib_semesters_collection(db).await?;
    create_lib_settings_styles_collection(db).await?;
    
    // Create UI metadata for these collections
    create_lib_ui_metadata(db).await?;
    
    Ok(())
}

// Helper function to create UI metadata for library collections
async fn create_lib_ui_metadata(db: &Database) -> Result<()> {
    let collection = db.collection::<Document>("ui_metadata");
    let now = mongodb::bson::DateTime::now();
    
    // Library collections
    let lib_collections = vec![
        "school_accounts",
        "attendance",
        "purposes",
        "semesters",
        "settings_styles",
    ];
    
    for collection_name in lib_collections {
        // Get column widths for this collection
        let column_widths = get_default_lib_column_widths(collection_name);
        
        // Create column order
        let column_order = mongodb::bson::to_bson(&column_widths.keys().collect::<Vec<_>>())
            .unwrap_or(mongodb::bson::Bson::Array(Vec::new()));
        
        let default_settings = doc! {
            "collection": collection_name,
            "ui": {
                "columnWidths": column_widths,
                "columnOrder": column_order,
                "hiddenColumns": [],
                "sortSettings": {
                    "field": get_default_lib_sort_field(collection_name),
                    "direction": "asc"
                },
                "filterSettings": {},
            },
            "created_at": now,
            "updated_at": now
        };
        
        // Use upsert to create only if not exists
        let filter = doc! {
            "collection": collection_name,
            "user_id": { "$exists": false }  // Global defaults have no user_id
        };
        
        collection.update_one(
            filter,
            doc! { "$setOnInsert": default_settings },
            mongodb::options::UpdateOptions::builder()
                .upsert(true)
                .build(),
        ).await?;
    }
    
    Ok(())
}