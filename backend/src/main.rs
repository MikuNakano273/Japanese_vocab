mod db;
mod models;
mod routes;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::fs::OpenOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    println!("Connecting to SQLite database...");

    // Force use SQLite with a relative path so everyone can run the server the same way.
    // Database file: ./data/mimikara_n3_questions.db
    let mut db_path = PathBuf::from("data");

    if let Err(e) = std::fs::create_dir_all(&db_path) {
        eprintln!("Warning: failed to create DB directory '{}': {}", db_path.display(), e);
    }

    db_path.push("mimikara_n3_questions.db");

    // Attempt to create the file if it doesn't exist (touch). Keep path relative.
    if !db_path.exists() {
        if let Err(e) = OpenOptions::new().create(true).write(true).open(&db_path) {
            eprintln!("Warning: failed to create DB file '{}': {}", db_path.display(), e);
        }
    }

    // Use an explicit relative path in the URI (do not canonicalize to an absolute path).
    let path_str = format!("./{}", db_path.to_string_lossy().replace('\\', "/"));
    let database_url = format!("sqlite://{}", path_str);

    println!("Using DB URI: {}", database_url);

    // Create a connection pool to the SQLite database
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to SQLite database");

    println!("Initializing database tables...");
    db::init_db(&pool).await.expect("Failed to initialize database");

    // SqlitePool is cloneable and safe to share between threads
    let pool = Arc::new(pool);

    println!("Starting server at http://localhost:8081");

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
