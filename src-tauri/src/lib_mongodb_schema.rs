// src/lib_mongodb_schema.rs

use mongodb::{
    options::IndexOptions,
    Database,
    IndexModel,
};
use mongodb::bson::{doc, Document};
use anyhow::Result;
use crate::mongodb_schema::{
    create_archive_index, 
    create_pinned_index,
    merge_with_archive_pinned_and_row_height_properties
};

// Library-specific collection for school accounts
pub async fn create_lib_school_accounts_collection(db: &Database) -> Result<()> {
    let collection = db.collection::<Document>("school_accounts");
    
    // Create indexes
    let indexes = vec![
        IndexModel::builder()
            .keys(doc! { "school_id": 1 })
            .options(Some(mongodb::options::IndexOptions::builder()
                .unique(true)
                .build()))
            .build(),
        IndexModel::builder()
            .keys(doc! { "last_updated_semester_id": 1 })
            .options(Some(IndexOptions::builder()
                .name("semester_ref_idx".to_string())
                .build()))
            .build(),
        create_archive_index(),
        create_pinned_index(),
    ];
    
    collection.create_indexes(indexes, None).await?;
    
    // Define base properties for this collection
    let base_properties = doc! {
        "school_id": { "bsonType": "string", "description": "Unique school ID (required)" },
        "first_name": { "bsonType": "string", "description": "First name" },
        "middle_name": { "bsonType": "string", "description": "Middle name" },
        "last_name": { "bsonType": "string", "description": "Last name" },
        "gender": { "bsonType": "int", "description": "Gender (integer code)" },
        "course": { "bsonType": "string", "description": "Course" },
        "department": { "bsonType": "string", "description": "Department" },
        "position": { "bsonType": "string", "description": "Position" },
        "major": { "bsonType": "string", "description": "Major" },
        "year_level": { "bsonType": "string", "description": "Year level" },
        "is_active": { "bsonType": "bool", "description": "Active status flag (required)" },
        "last_updated_semester_id": { "bsonType": "string", "description": "REF:semesters | Reference to last updated semester" },
        "created_at": { "bsonType": "date", "description": "Creation timestamp (required)" },
        "updated_at": { "bsonType": "date", "description": "Last update timestamp (required)" }
    };
    
    // Merge with both archive and pinned properties
    let properties = merge_with_archive_pinned_and_row_height_properties(base_properties);
    
    // Apply validator schema using collMod
    db.run_command(
        doc! {
            "collMod": "school_accounts",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["school_id", "is_active", "created_at"],
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
            .keys(doc! { "school_id": 1, "time_in_date": 1 })
            .options(Some(IndexOptions::builder()
                .unique(true)
                .build()))
            .build(),
        IndexModel::builder()
            .keys(doc! { "time_in_date": 1 })
            .build(),
        create_archive_index(),
        create_pinned_index(),
    ];
    
    collection.create_indexes(indexes, None).await?;
    
    // Define base properties for this collection
    let base_properties = doc! {
        "school_id": { "bsonType": "string", "description": "School ID (required)" },
        "full_name": { "bsonType": "string", "description": "Full name of the person (required)" },
        "time_in_date": { "bsonType": "date", "description": "Date and time of entry (required)" },
        "classification": { "bsonType": "string", "description": "Classification (required)" },
        "purpose_label": { "bsonType": "string", "description": "Purpose label" },
        "created_at": { "bsonType": "date", "description": "Creation timestamp (required)" },
        "updated_at": { "bsonType": "date", "description": "Last update timestamp (required)" }
    };
    
    // Merge with both archive and pinned properties
    let properties = merge_with_archive_pinned_and_row_height_properties(base_properties);
    
    // Apply validator schema using collMod
    db.run_command(
        doc! {
            "collMod": "attendance",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["school_id", "full_name", "classification", "created_at"],
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
            .keys(doc! { "label": 1 })
            .options(Some(mongodb::options::IndexOptions::builder()
                .unique(true)
                .build()))
            .build(),
        create_archive_index(),
        create_pinned_index(),
    ];
    
    collection.create_indexes(indexes, None).await?;
    
    // Define base properties for this collection
    let base_properties = doc! {
        "label": { "bsonType": "string", "description": "Purpose label (required, unique)" },
        "icon_name": { "bsonType": "string", "description": "Icon name (required)" },
        "is_deleted": { "bsonType": "bool", "description": "Deletion flag (required)" },
        "created_at": { "bsonType": "date", "description": "Creation timestamp (required)" },
        "updated_at": { "bsonType": "date", "description": "Last update timestamp (required)" }
    };
    
    // Merge with both archive and pinned properties
    let properties = merge_with_archive_pinned_and_row_height_properties(base_properties);
    
    // Apply validator schema using collMod
    db.run_command(
        doc! {
            "collMod": "purposes",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["label", "icon_name", "is_deleted", "created_at"],
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
            .keys(doc! { "label": 1 })
            .options(Some(IndexOptions::builder()
                .unique(true)
                .build()))
            .build(),
        create_archive_index(),
        create_pinned_index(),
    ];
    
    collection.create_indexes(indexes, None).await?;
    
    // Define base properties for this collection
    let base_properties = doc! {
        "label": { "bsonType": "string", "description": "Unique label for the semester (required)" },
        "is_active": { "bsonType": "bool", "description": "Indicates if this is the active semester (required)" },
        "created_at": { "bsonType": "date", "description": "Creation timestamp (required)" },
        "updated_at": { "bsonType": "date", "description": "Last update timestamp (required)" }
    };
    
    // Merge with both archive and pinned properties
    let properties = merge_with_archive_pinned_and_row_height_properties(base_properties);
    
    // Apply validator schema using collMod
    db.run_command(
        doc! {
            "collMod": "semesters",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["label", "is_active", "created_at"],
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
            .keys(doc! { "component_name": 1 })
            .options(Some(IndexOptions::builder()
                .unique(true)  // Add unique constraint here
                .build()))
            .build(),
        create_archive_index(),
        create_pinned_index(),
    ];
    
    collection.create_indexes(indexes, None).await?;
    
    // Define base properties for this collection
    let base_properties = doc! {
        "component_name": { "bsonType": "string", "description": "Component name (required)" },
        "tailwind_classes": { "bsonType": "string", "description": "Tailwind CSS classes (required)" },
        "label": { "bsonType": "string", "description": "Optional label" },
        "created_at": { "bsonType": "date", "description": "Creation timestamp (required)" },
        "updated_at": { "bsonType": "date", "description": "Last update timestamp (required)" }
    };
    
    // Merge with both archive and pinned properties
    let properties = merge_with_archive_pinned_and_row_height_properties(base_properties);
    
    // Apply validator schema using collMod
    db.run_command(
        doc! {
            "collMod": "settings_styles",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["component_name", "tailwind_classes", "created_at"],
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
            "school_id": default_column_width,
            "first_name": default_column_width,
            "middle_name": default_column_width,
            "last_name": default_column_width,
            "gender": default_column_width,
            "course": default_column_width,
            "department": default_column_width,
            "position": default_column_width,
            "major": default_column_width,
            "year_level": default_column_width,
            "is_active": default_column_width,
            "last_updated_semester_id": default_column_width,
            "is_archive": default_column_width,
            "pinned_by": default_column_width,
            "row_height": default_column_width,
            "created_at": default_column_width,
            "updated_at": default_column_width
        },
        "attendance" => doc! {
            "school_id": default_column_width,
            "full_name": default_column_width,
            "time_in_date": default_column_width,
            "classification": default_column_width,
            "purpose_label": default_column_width,
            "is_archive": default_column_width,
            "pinned_by": default_column_width,
            "row_height": default_column_width,
            "created_at": default_column_width,
            "updated_at": default_column_width
        },
        "purposes" => doc! {
            "label": default_column_width,
            "icon_name": default_column_width,
            "is_deleted": default_column_width,
            "is_archive": default_column_width,
            "pinned_by": default_column_width,
            "row_height": default_column_width,
            "created_at": default_column_width,
            "updated_at": default_column_width
        },
        "semesters" => doc! {
            "label": default_column_width,
            "is_active": default_column_width,
            "is_archive": default_column_width,
            "pinned_by": default_column_width,
            "row_height": default_column_width,
            "created_at": default_column_width,
            "updated_at": default_column_width
        },
        "settings_styles" => doc! {
            "component_name": default_column_width,
            "tailwind_classes": default_column_width,
            "label": default_column_width,
            "is_archive": default_column_width,
            "pinned_by": default_column_width,
            "row_height": default_column_width,
            "created_at": default_column_width,
            "updated_at": default_column_width
        },
        _ => doc! {}
    }
}

// Helper function to get default sort field for library collections
pub fn get_default_lib_sort_field(collection_name: &str) -> &str {
    match collection_name {
        "school_accounts" => "school_id",
        "attendance" => "time_in_date",
        "purposes" => "label",
        "semesters" => "label",
        "settings_styles" => "component_name",
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