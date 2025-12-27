mod db;
mod models;
mod routes;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    
    println!("Connecting to database...");
    let client = db::connect().await.expect("Failed to connect to database");
    
    println!("Initializing database tables...");
    db::init_db(&client).await.expect("Failed to initialize database");
    
    let client = Arc::new(Mutex::new(client));
    
    println!("Starting server at http://localhost:8080");
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(client.clone()))
            .configure(routes::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

