// src/mongodb_schema.rs
use mongodb::{Database, IndexModel};
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
            .build(),
    ];

    collection.create_indexes(indexes, None).await?;
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
    Ok(())
}

async fn create_semesters_collection(db: &Database) -> Result<()> {
    let collection = db.collection::<Document>("semesters");
    
    let index = IndexModel::builder()
        .keys(doc! { "label": 1 })
        .options(Some(mongodb::options::IndexOptions::builder()
            .unique(true)
            .build()))
        .build();

    collection.create_index(index, None).await?;
    Ok(())
}

async fn create_settings_styles_collection(db: &Database) -> Result<()> {
    // No unique indexes needed for this collection
    let collection = db.collection::<Document>("settings_styles");
    collection.create_index(
        IndexModel::builder()
            .keys(doc! { "component_name": 1 })
            .build(),
        None
    ).await?;
    Ok(())
}