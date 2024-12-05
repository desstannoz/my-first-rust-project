use sea_orm::ConnectionTrait;
use sea_orm::DatabaseBackend;
use sea_orm::{Database, DatabaseConnection, Schema, Statement};
use sea_orm_migration::prelude::SqliteQueryBuilder;

mod controllers;
mod middleware;
mod models;
mod routes;

type DB = DatabaseConnection;

#[tokio::main]
async fn main() {
    // Veritabani olusturma
    let db_path = "master.db";
    if !std::path::Path::new(db_path).exists() {
        std::fs::File::create(db_path).expect("Could not create database file");
        println!("Database file created: {}", db_path);
    }

    let db = Database::connect(format!("sqlite:{}", db_path))
        .await
        .expect("Failed to connect to database");

    // Model'e göre tablo oluştur
    let schema = Schema::new(DatabaseBackend::Sqlite);
    let sql = schema.create_table_from_entity(models::user::Entity)
        .if_not_exists()
        .build(SqliteQueryBuilder);

    db.execute(Statement::from_string(DatabaseBackend::Sqlite, sql))
        .await
        .expect("Could not create table");

    println!("Database initialized successfully");

    let app = routes::routes(db);

    println!("Listening on http://0.0.0.0:3000");
    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
