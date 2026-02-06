use sqlx::mysql::MySqlPool;

// Note: the previous `connect()` helper was removed because the application
// uses `MySqlPool::connect` directly in `main.rs`. If a centralized helper
// is needed later, it can be reintroduced here.

/// Initialize required tables for the application in MySQL.
///
/// Notes about type differences for MySQL:
/// - `INT AUTO_INCREMENT` is used for id with PRIMARY KEY
/// - `VARCHAR` and `TEXT` are used for text fields
/// - `DATETIME` is used for timestamps with DEFAULT CURRENT_TIMESTAMP
/// - `options` is stored as JSON text in a `TEXT` column
pub async fn init_db(pool: &MySqlPool) -> Result<(), sqlx::Error> {
    // Create entries table (source data)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS entries (
            id INT AUTO_INCREMENT PRIMARY KEY,
            list_index INT,
            kanji TEXT,
            kana TEXT,
            meaning TEXT
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
        "#,
    )
    .execute(pool)
    .await?;

    // Create quizzes table (optional container for grouping questions)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS quizzes (
            id INT AUTO_INCREMENT PRIMARY KEY,
            title VARCHAR(255) NOT NULL,
            description TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
        "#,
    )
    .execute(pool)
    .await?;

    // Create n_level mapping table (1 -> n5, 2 -> n4, ..., 5 -> n1)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS n_level (
            id INT PRIMARY KEY,
            level VARCHAR(10) NOT NULL
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
        "#,
    )
    .execute(pool)
    .await?;

    // Populate canonical n_level rows (id 1..5)
    // Use INSERT IGNORE so this is idempotent.
    sqlx::query("INSERT IGNORE INTO n_level (id, level) VALUES (?, ?)")
        .bind(1i32)
        .bind("n5")
        .execute(pool)
        .await?;
    sqlx::query("INSERT IGNORE INTO n_level (id, level) VALUES (?, ?)")
        .bind(2i32)
        .bind("n4")
        .execute(pool)
        .await?;
    sqlx::query("INSERT IGNORE INTO n_level (id, level) VALUES (?, ?)")
        .bind(3i32)
        .bind("n3")
        .execute(pool)
        .await?;
    sqlx::query("INSERT IGNORE INTO n_level (id, level) VALUES (?, ?)")
        .bind(4i32)
        .bind("n2")
        .execute(pool)
        .await?;
    sqlx::query("INSERT IGNORE INTO n_level (id, level) VALUES (?, ?)")
        .bind(5i32)
        .bind("n1")
        .execute(pool)
        .await?;

    // Create questions table (keeps original columns and extends with level & chapter)
    // - entry_id references entries.id
    // - quiz_id is optional (can be NULL) and references quizzes.id
    // - options stored as JSON text in a TEXT column
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS questions (
            id INT AUTO_INCREMENT PRIMARY KEY,
            entry_id INT,
            quiz_id INT,
            q_type VARCHAR(50),
            prompt TEXT,
            correct_answer TEXT,
            options TEXT,
            correct_index INT,
            level INT,
            chapter INT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
            FOREIGN KEY (quiz_id) REFERENCES quizzes(id) ON DELETE SET NULL,
            FOREIGN KEY (level) REFERENCES n_level(id)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
        "#,
    )
    .execute(pool)
    .await?;

    // Create tests table to store generated tests (questions JSON)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tests (
            id INT AUTO_INCREMENT PRIMARY KEY,
            title VARCHAR(255),
            questions TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
        "#,
    )
    .execute(pool)
    .await?;

    // Helpful index for lookups by entry_id
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_entry_id ON questions(entry_id);")
        .execute(pool)
        .await?;

    // Add missing columns to questions table if they don't exist
    // This handles the case where database was created from SQL file
    // Note: MySQL doesn't have "ALTER TABLE ADD COLUMN IF NOT EXISTS"
    // so we check if column exists first
    
    // Helper function to check if column exists in a table
    // Note: This function is only called with hardcoded table names during initialization,
    // not with user input. Table names are validated via allowlist.
    async fn column_exists(pool: &MySqlPool, table: &str, column: &str) -> bool {
        // Validate table name against allowlist to prevent SQL injection
        const ALLOWED_TABLES: &[&str] = &["questions", "entries", "quizzes", "tests", "n_level"];
        if !ALLOWED_TABLES.contains(&table) {
            return false;
        }
        
        // For MySQL, we query information_schema
        let query = "SELECT COLUMN_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? AND COLUMN_NAME = ?";
        if let Ok(rows) = sqlx::query(query)
            .bind(table)
            .bind(column)
            .fetch_all(pool)
            .await 
        {
            return !rows.is_empty();
        }
        false
    }

    // Add quiz_id column if missing
    if !column_exists(pool, "questions", "quiz_id").await {
        // Add column first
        sqlx::query("ALTER TABLE questions ADD COLUMN quiz_id INT")
            .execute(pool)
            .await?;
        // Then add foreign key constraint
        // Note: Silently ignore errors if constraint already exists or for invalid data
        // This is acceptable as the column is the important part for functionality
        if let Err(e) = sqlx::query("ALTER TABLE questions ADD FOREIGN KEY (quiz_id) REFERENCES quizzes(id) ON DELETE SET NULL")
            .execute(pool)
            .await {
            // Log error for debugging but don't fail initialization
            eprintln!("Note: Could not add foreign key constraint for quiz_id (may already exist): {}", e);
        }
    }

    // Add level column if missing
    if !column_exists(pool, "questions", "level").await {
        // Add column first
        sqlx::query("ALTER TABLE questions ADD COLUMN level INT")
            .execute(pool)
            .await?;
        // Then add foreign key constraint
        // Note: Silently ignore errors if constraint already exists or for invalid data
        // This is acceptable as the column is the important part for functionality
        if let Err(e) = sqlx::query("ALTER TABLE questions ADD FOREIGN KEY (level) REFERENCES n_level(id)")
            .execute(pool)
            .await {
            // Log error for debugging but don't fail initialization
            eprintln!("Note: Could not add foreign key constraint for level (may already exist): {}", e);
        }
    }

    // Add chapter column if missing
    if !column_exists(pool, "questions", "chapter").await {
        sqlx::query("ALTER TABLE questions ADD COLUMN chapter INT")
            .execute(pool)
            .await?;
    }

    Ok(())
}
