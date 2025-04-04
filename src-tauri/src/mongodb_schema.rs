// src/mongodb_schema.rs
use mongodb::{
    options::IndexOptions,
    Database,
    IndexModel,
};
use mongodb::bson::{doc, Document};
use anyhow::Result;

pub async fn initialize_database(db: &Database) -> Result<()> {
    create_school_accounts_collection(db).await?;
    create_attendance_collection(db).await?;
    create_users_collection(db).await?;
    create_purposes_collection(db).await?;
    create_semesters_collection(db).await?;
    create_settings_styles_collection(db).await?;
    Ok(())
}

async fn create_school_accounts_collection(db: &Database) -> Result<()> {
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
    ];

    collection.create_indexes(indexes, None).await?;
    
    // Apply validator schema using collMod
    db.run_command(
        doc! {
            "collMod": "school_accounts",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["school_id", "is_active", "created_at", "updated_at"],
                    "properties": {
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
                    }
                }
            },
            "validationLevel": "moderate",
            "validationAction": "error"
        },
        None
    ).await?;
    
    Ok(())
}

async fn create_attendance_collection(db: &Database) -> Result<()> {
    let collection = db.collection::<Document>("attendance");
    
    let indexes = vec![
        IndexModel::builder()
            .keys(doc! { "school_id": 1 })
            .build(),
        IndexModel::builder()
            .keys(doc! { "time_in_date": 1 })
            .build(),
    ];

    collection.create_indexes(indexes, None).await?;
    
    // Apply validator schema using collMod
    db.run_command(
        doc! {
            "collMod": "attendance",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["school_id", "full_name", "time_in_date", "classification", "created_at", "updated_at"],
                    "properties": {
                        "school_id": { "bsonType": "string", "description": "School ID (required)" },
                        "full_name": { "bsonType": "string", "description": "Full name of the person (required)" },
                        "time_in_date": { "bsonType": "date", "description": "Date and time of entry (required)" },
                        "classification": { "bsonType": "string", "description": "Classification (required)" },
                        "purpose_label": { "bsonType": "string", "description": "Purpose label" },
                        "created_at": { "bsonType": "date", "description": "Creation timestamp (required)" },
                        "updated_at": { "bsonType": "date", "description": "Last update timestamp (required)" }
                    }
                }
            },
            "validationLevel": "moderate",
            "validationAction": "error"
        },
        None
    ).await?;
    
    Ok(())
}

async fn create_users_collection(db: &Database) -> Result<()> {
    let collection = db.collection::<Document>("users");
    
    let index = IndexModel::builder()
        .keys(doc! { "username": 1 })
        .options(Some(mongodb::options::IndexOptions::builder()
            .unique(true)
            .build()))
        .build();

    collection.create_index(index, None).await?;
    
    // Apply validator schema using collMod
    db.run_command(
        doc! {
            "collMod": "users",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["username", "password", "created_at", "updated_at"],
                    "properties": {
                        "username": { "bsonType": "string", "description": "Username (required, unique)" },
                        "password": { "bsonType": "string", "description": "Password (hashed, required)" },
                        "created_at": { "bsonType": "date", "description": "Creation timestamp (required)" },
                        "updated_at": { "bsonType": "date", "description": "Last update timestamp (required)" }
                    }
                }
            },
            "validationLevel": "moderate",
            "validationAction": "error"
        },
        None
    ).await?;
    
    Ok(())
}

async fn create_purposes_collection(db: &Database) -> Result<()> {
    let collection = db.collection::<Document>("purposes");
    
    let index = IndexModel::builder()
        .keys(doc! { "label": 1 })
        .options(Some(mongodb::options::IndexOptions::builder()
            .unique(true)
            .build()))
        .build();

    collection.create_index(index, None).await?;
    
    // Apply validator schema using collMod
    db.run_command(
        doc! {
            "collMod": "purposes",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["label", "icon_name", "is_deleted", "created_at", "updated_at"],
                    "properties": {
                        "label": { "bsonType": "string", "description": "Purpose label (required, unique)" },
                        "icon_name": { "bsonType": "string", "description": "Icon name (required)" },
                        "is_deleted": { "bsonType": "bool", "description": "Deletion flag (required)" },
                        "created_at": { "bsonType": "date", "description": "Creation timestamp (required)" },
                        "updated_at": { "bsonType": "date", "description": "Last update timestamp (required)" }
                    }
                }
            },
            "validationLevel": "moderate",
            "validationAction": "error"
        },
        None
    ).await?;
    
    Ok(())
}

async fn create_semesters_collection(db: &Database) -> Result<()> {
    // First create index on the collection (this will create the collection if it doesn't exist)
    let collection = db.collection::<Document>("semesters");
    let index = IndexModel::builder()
        .keys(doc! { "label": 1 })
        .options(Some(IndexOptions::builder()
            .unique(true)
            .build()))
        .build();

    collection.create_index(index, None).await?;
    
    // Then apply or update the validator schema using collMod
    db.run_command(
        doc! {
            "collMod": "semesters",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["label", "is_active", "created_at", "updated_at"],
                    "properties": {
                        "label": { "bsonType": "string", "description": "Unique label for the semester (required)" },
                        "is_active": { "bsonType": "bool", "description": "Indicates if this is the active semester (required)" },
                        "created_at": { "bsonType": "date", "description": "Creation timestamp (required)" },
                        "updated_at": { "bsonType": "date", "description": "Last update timestamp (required)" }
                    }
                }
            },
            "validationLevel": "moderate",
            "validationAction": "error"
        },
        None
    ).await?;
    
    Ok(())
}

pub async fn create_settings_styles_collection(db: &Database) -> Result<()> {
    // First create index on the collection (this will create the collection if it doesn't exist)
    let collection = db.collection::<Document>("settings_styles");
    
    // Create index on component_name field
    collection.create_index(
        IndexModel::builder()
            .keys(doc! { "component_name": 1 })
            .build(),
        None
    ).await?;
    
    // Then apply or update the validator schema using collMod
    db.run_command(
        doc! {
            "collMod": "settings_styles",
            "validator": {
                "$jsonSchema": {
                    "bsonType": "object",
                    "required": ["component_name", "tailwind_classes", "created_at", "updated_at"],
                    "properties": {
                        "component_name": { "bsonType": "string", "description": "Component name (required)" },
                        "tailwind_classes": { "bsonType": "string", "description": "Tailwind CSS classes (required)" },
                        "label": { "bsonType": "string", "description": "Optional label" },
                        "created_at": { "bsonType": "date", "description": "Creation timestamp (required)" },
                        "updated_at": { "bsonType": "date", "description": "Last update timestamp (required)" }
                    }
                }
            },
            "validationLevel": "moderate",
            "validationAction": "error"
        },
        None
    ).await?;
    
    Ok(())
}