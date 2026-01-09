mod db;
mod models;
mod routes;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
/* Removed unnecessary Arc wrapper: SqlitePool is Clone and can be cloned directly when passed into actix-web App */
use sqlx::SqlitePool;
use std::fs::OpenOptions;
use std::path::PathBuf;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // debug output removed

    // Force use SQLite with a relative path so everyone can run the server the same way.
    // Database file: ./data/mimikara_n3_questions.db
    let mut db_path = PathBuf::from("data");

    let _ = std::fs::create_dir_all(&db_path);

    db_path.push("mimikara_n3_questions.db");

    // Attempt to create the file if it doesn't exist (touch). Keep path relative.
    if !db_path.exists() {
        let _ = OpenOptions::new().create(true).write(true).open(&db_path);
    }

    // Use an explicit relative path in the URI (do not canonicalize to an absolute path).
    let path_str = format!("./{}", db_path.to_string_lossy().replace('\\', "/"));
    let database_url = format!("sqlite://{}", path_str);

    // debug output removed

    // Create a connection pool to the SQLite database
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("Failed to connect to SQLite database");

    // debug output removed
    db::init_db(&pool)
        .await
        .expect("Failed to initialize database");

    // SqlitePool is cloneable and safe to share between threads (SqlitePool implements Clone)
    // No Arc wrapper is needed; we'll clone the pool directly when providing it to App.
    // Keep using the existing `pool` (SqlitePool) variable here.

    // debug output removed

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
