mod db;
mod models;
mod routes;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
/* SqlitePool is Clone and can be cloned directly when passed into actix-web App */
use sqlx::SqlitePool;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // Get SQLite database URL from environment variable
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://mimikara_n3_questions.db".to_string());

    println!("Connecting to SQLite database...");

    // Create a connection pool to the SQLite database
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to SQLite database");

    println!("Connected to SQLite database successfully");
    
    db::init_db(&pool)
        .await
        .expect("Failed to initialize database");

    println!("Database initialized successfully");

    // SqlitePool is cloneable and safe to share between threads (SqlitePool implements Clone)
    // No Arc wrapper is needed; we'll clone the pool directly when providing it to App.

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            // provide the shared SQLite pool to handlers
            .app_data(web::Data::new(pool.clone()))
            // allow larger JSON payloads for test creation (adjust limit as needed)
            .app_data(web::JsonConfig::default().limit(10 * 1024 * 1024))
            // configure routes (includes quizzes and tests)
            .configure(routes::config)
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
