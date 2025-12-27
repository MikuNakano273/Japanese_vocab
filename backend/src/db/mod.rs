use tokio_postgres::{Client, Error, NoTls};
use std::env;

pub async fn connect() -> Result<Client, Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "host=localhost user=postgres password=postgres dbname=japanese_vocab".to_string());
    
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Database connection error: {}", e);
        }
    });
    
    Ok(client)
}

pub async fn init_db(client: &Client) -> Result<(), Error> {
    // Create quizzes table
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS quizzes (
                id SERIAL PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                description TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            &[],
        )
        .await?;
    
    // Create questions table
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS questions (
                id SERIAL PRIMARY KEY,
                quiz_id INTEGER NOT NULL REFERENCES quizzes(id) ON DELETE CASCADE,
                text TEXT NOT NULL,
                options JSONB NOT NULL,
                correct_answer INTEGER NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            &[],
        )
        .await?;
    
    println!("Database tables initialized successfully");
    Ok(())
}
