use sqlx::sqlite::SqlitePool;
use sqlx::Row;

// Note: the previous `connect()` helper was removed because the application
// uses `SqlitePool::connect` directly in `main.rs`. If a centralized helper
// is needed later, it can be reintroduced here.

/// Initialize required tables for the application in SQLite.
///
/// Notes about type differences for SQLite:
/// - `INTEGER PRIMARY KEY AUTOINCREMENT` is used for id
/// - `TEXT` is used for text fields and timestamps (using `datetime('now')`)
/// - `options` is stored as JSON text in a `TEXT` column
pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Enable foreign key enforcement in SQLite
    // (Must be set per-connection for older SQLite versions; this is safe to run repeatedly.)
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(pool)
        .await?;

    // Create entries table (source data)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            list_index INTEGER,
            kanji TEXT,
            kana TEXT,
            meaning TEXT
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Create quizzes table (optional container for grouping questions)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS quizzes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Create n_level mapping table (1 -> n5, 2 -> n4, ..., 5 -> n1)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS n_level (
            id INTEGER PRIMARY KEY,
            level TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Populate canonical n_level rows (id 1..5)
    // Use INSERT OR IGNORE so this is idempotent.
    sqlx::query("INSERT OR IGNORE INTO n_level (id, level) VALUES (?, ?)")
        .bind(1i64)
        .bind("n5")
        .execute(pool)
        .await?;
    sqlx::query("INSERT OR IGNORE INTO n_level (id, level) VALUES (?, ?)")
        .bind(2i64)
        .bind("n4")
        .execute(pool)
        .await?;
    sqlx::query("INSERT OR IGNORE INTO n_level (id, level) VALUES (?, ?)")
        .bind(3i64)
        .bind("n3")
        .execute(pool)
        .await?;
    sqlx::query("INSERT OR IGNORE INTO n_level (id, level) VALUES (?, ?)")
        .bind(4i64)
        .bind("n2")
        .execute(pool)
        .await?;
    sqlx::query("INSERT OR IGNORE INTO n_level (id, level) VALUES (?, ?)")
        .bind(5i64)
        .bind("n1")
        .execute(pool)
        .await?;

    // Create questions table (keeps original columns and extends with level & chapter)
    // - entry_id references entries.id
    // - quiz_id is optional (can be NULL) and references quizzes.id
    // - options stored as JSON text in TEXT column
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS questions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_id INTEGER REFERENCES entries(id) ON DELETE CASCADE,
            quiz_id INTEGER REFERENCES quizzes(id) ON DELETE SET NULL,
            q_type TEXT,
            prompt TEXT,
            correct_answer TEXT,
            options TEXT,
            correct_index INTEGER,
            level INTEGER REFERENCES n_level(id),
            chapter INTEGER,
            created_at TEXT DEFAULT (datetime('now'))
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Create tests table to store generated tests (questions JSON)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT,
            questions TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );
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
    // Note: SQLite doesn't have "ALTER TABLE ADD COLUMN IF NOT EXISTS"
    // so we check if column exists first
    
    // Helper function to check if column exists
    async fn column_exists(pool: &SqlitePool, table: &str, column: &str) -> bool {
        let query = format!("PRAGMA table_info({})", table);
        if let Ok(rows) = sqlx::query(&query).fetch_all(pool).await {
            for row in rows {
                if let Ok(name) = row.try_get::<String, _>("name") {
                    if name == column {
                        return true;
                    }
                }
            }
        }
        false
    }

    // Add quiz_id column if missing
    if !column_exists(pool, "questions", "quiz_id").await {
        sqlx::query("ALTER TABLE questions ADD COLUMN quiz_id INTEGER REFERENCES quizzes(id) ON DELETE SET NULL")
            .execute(pool)
            .await?;
    }

    // Add level column if missing
    if !column_exists(pool, "questions", "level").await {
        sqlx::query("ALTER TABLE questions ADD COLUMN level INTEGER REFERENCES n_level(id)")
            .execute(pool)
            .await?;
    }

    // Add chapter column if missing
    if !column_exists(pool, "questions", "chapter").await {
        sqlx::query("ALTER TABLE questions ADD COLUMN chapter INTEGER")
            .execute(pool)
            .await?;
    }

    Ok(())
}
