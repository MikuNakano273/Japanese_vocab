# Database Integration - Implementation Notes

## Overview
This document describes the migration from SQLite to MySQL and bug fixes made to the database integration.

## Changes Made

### 1. Bug Fix: Column Name Mismatch
**Problem**: The database schema defined a `prompt` column, but the application code was trying to query/insert using `text` column.

**Files Affected**: `backend/src/routes/mod.rs`

**Changes**:
- Changed `SELECT id, text, options, correct_answer` to `SELECT id, prompt, options, correct_answer` in `list_quizzes()` function (line 30)
- Changed `r.try_get("text")` to `r.try_get("prompt")` in `list_quizzes()` function (line 44)
- Changed `SELECT id, text, options, correct_answer` to `SELECT id, prompt, options, correct_answer` in `get_quiz()` function (line 88)
- Changed `r.try_get("text")` to `r.try_get("prompt")` in `get_quiz()` function (line 104)
- Changed `INSERT INTO questions (quiz_id, text, ...)` to `INSERT INTO questions (quiz_id, prompt, ...)` in `create_quiz()` function (line 164)

**Impact**: This bug would have prevented the application from properly retrieving and creating quiz questions.

### 2. Database Migration from SQLite to MySQL

#### Cargo.toml Changes
- Changed sqlx features from `["sqlite", ...]` to `["mysql", ...]`

#### Backend Database Module (`backend/src/db/mod.rs`)
**Changes**:
- Changed `SqlitePool` to `MySqlPool` throughout
- Updated table creation syntax for MySQL:
  - `INTEGER PRIMARY KEY AUTOINCREMENT` → `INT AUTO_INCREMENT PRIMARY KEY`
  - `TEXT DEFAULT (datetime('now'))` → `DATETIME DEFAULT CURRENT_TIMESTAMP`
  - Added `ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci` to all tables
  - Added explicit `FOREIGN KEY` constraints for MySQL
- Updated `INSERT OR IGNORE` to `INSERT IGNORE` (MySQL syntax)
- Changed integer bindings from `i64` to `i32` for n_level inserts
- Rewrote `column_exists()` helper function:
  - Removed SQLite's `PRAGMA table_info()` query
  - Added MySQL's `INFORMATION_SCHEMA.COLUMNS` query
- Updated ALTER TABLE statements for MySQL foreign key syntax

#### Backend Main File (`backend/src/main.rs`)
**Changes**:
- Removed SQLite file-based connection logic
- Added MySQL connection using `DATABASE_URL` environment variable
- Changed from `SqlitePool::connect()` to `MySqlPool::connect()`
- Removed file system operations for database file creation
- Added connection status logging

#### Backend Routes (`backend/src/routes/mod.rs`)
**Changes**:
- Changed all `web::Data<SqlitePool>` to `web::Data<MySqlPool>`
- Changed `SqliteRow` to `MySqlRow`
- Changed `last_insert_rowid()` to `last_insert_id()` for MySQL compatibility

### 3. Database Initialization Script (`init_database_mysql.sh`)
Created a new bash script to initialize the MySQL database from the SQL file:
- **Location**: `init_database_mysql.sh` (project root)
- **Purpose**: Creates and populates MySQL `japanese_vocab` database from `mimikara_n3_questions.db.sql`
- **Features**:
  - Checks if MySQL is accessible
  - Creates database with utf8mb4 charset
  - Checks if database already exists and has data
  - Converts SQLite SQL syntax to MySQL syntax:
    - `BEGIN TRANSACTION` → `START TRANSACTION`
    - `SERIAL PRIMARY KEY` → `INT AUTO_INCREMENT PRIMARY KEY`
    - `TIMESTAMP` → `DATETIME`
  - Only initializes if needed (idempotent)
  - Provides user-friendly status messages
  - Validates successful initialization

**Usage**:
```bash
./init_database_mysql.sh
```

**Environment Variables**:
- `MYSQL_USER` (default: root)
- `MYSQL_PASSWORD` (default: password)
- `MYSQL_HOST` (default: localhost)
- `MYSQL_PORT` (default: 3306)

### 4. Environment Configuration Updates
Updated default configurations to use MySQL:

**`backend/.env.example`**:
- Changed from SQLite to MySQL connection string
- Format: `mysql://username:password@host:port/database`
- Default: `mysql://root:password@localhost:3306/japanese_vocab`

### 5. Documentation Updates (`README.md`)
Enhanced the README with MySQL setup instructions:
- Updated architecture section to mention MySQL
- Added MySQL to prerequisites
- Detailed MySQL setup steps:
  - Starting MySQL server
  - Configuring database connection
  - Running initialization script
- Updated manual database inspection with MySQL CLI
- Updated all configuration examples
- Updated database management section

## Database Schema

### Tables in MySQL Database
1. **entries** - Vocabulary entries
   - id: INT AUTO_INCREMENT PRIMARY KEY
   - list_index: INT
   - kanji: TEXT
   - kana: TEXT
   - meaning: TEXT

2. **quizzes** - User-created quizzes
   - id: INT AUTO_INCREMENT PRIMARY KEY
   - title: VARCHAR(255) NOT NULL
   - description: TEXT
   - created_at: DATETIME DEFAULT CURRENT_TIMESTAMP

3. **n_level** - JLPT level mapping
   - id: INT PRIMARY KEY (1→n5, 2→n4, 3→n3, 4→n2, 5→n1)
   - level: VARCHAR(10) NOT NULL

4. **questions** - Quiz questions
   - id: INT AUTO_INCREMENT PRIMARY KEY
   - entry_id: INT (FK → entries.id)
   - quiz_id: INT (FK → quizzes.id, nullable)
   - q_type: VARCHAR(50)
   - prompt: TEXT (Note: Previously incorrectly referenced as "text" in code)
   - correct_answer: TEXT
   - options: TEXT (JSON string)
   - correct_index: INT
   - level: INT (FK → n_level.id)
   - chapter: INT
   - created_at: DATETIME DEFAULT CURRENT_TIMESTAMP

5. **tests** - Generated tests
   - id: INT AUTO_INCREMENT PRIMARY KEY
   - title: VARCHAR(255)
   - questions: TEXT (JSON string)
   - created_at: DATETIME DEFAULT CURRENT_TIMESTAMP

### Indexes
- `idx_entry_id` on questions(entry_id) for faster lookups

### Foreign Key Constraints
- questions.entry_id → entries.id (ON DELETE CASCADE)
- questions.quiz_id → quizzes.id (ON DELETE SET NULL)
- questions.level → n_level.id

All tables use:
- Engine: InnoDB
- Character Set: utf8mb4
- Collation: utf8mb4_unicode_ci

### Backend Schema Migration
The backend automatically adds missing columns to the questions table on first run if they don't exist:
- **quiz_id**: INT (FK to quizzes table)
- **level**: INT (FK to n_level table)
- **chapter**: INT (for chapter grouping)

This ensures backward compatibility with databases created from the SQL file.

## Error Handling

### Missing Database
- Backend fails to connect with appropriate error message
- `init_database_mysql.sh` creates the database if it doesn't exist

### Empty Database
- `init_database_mysql.sh` detects empty database and populates it

### Invalid Schema
- Backend automatically adds missing columns using `column_exists()` helper
- Uses INFORMATION_SCHEMA to detect existing columns
- Foreign key constraints are properly handled

### Database Connection Errors
- Backend returns appropriate HTTP error codes
- Frontend displays user-friendly error messages

## Test Generation Flow

1. User clicks "Create Test" in frontend
2. Frontend sends POST request to `/api/tests` with:
   - level (n5, n4, n3, n2, n1)
   - mode (chapter or range)
   - chapters or range specification
   - numQuestions (optional)

3. Backend:
   - Builds SQL query with WHERE clauses based on filters
   - Executes query with RANDOM ordering
   - If no results, falls back to global selection
   - Stores test in `tests` table
   - Returns test ID and redirect URL

4. Frontend redirects to `/test/:id`
5. User takes the test with immediate feedback

## Known Limitations

### Chapter and Level Filtering
The questions in the SQL file don't have `level` or `chapter` values populated. This means:
- Level filtering won't work until data is populated
- Chapter filtering won't work until data is populated
- Tests will be generated from all available questions regardless of filters

**Workaround**: The backend falls back to random selection from all questions if no matches found.

### Future Enhancements
To fully utilize the filtering features:
1. Populate the `level` column based on question difficulty
2. Populate the `chapter` column based on vocabulary grouping
3. Update existing questions with appropriate values

## Testing

### Manual Testing Steps
1. Ensure MySQL is running
2. Initialize database: `./init_database_mysql.sh`
3. Configure backend: Update `backend/.env` with MySQL credentials
4. Start backend: `cd backend && cargo run`
5. Start frontend: `cd frontend && npm start`
6. Navigate to `http://localhost:3000/test/create`
7. Select options and create test
8. Verify test is generated and displayed

### API Testing
```bash
# Test test creation
curl -X POST http://localhost:8081/api/tests \
  -H "Content-Type: application/json" \
  -d '{"level":"n3","mode":"range","range":{"start":1,"end":100},"numQuestions":10}'

# Test test retrieval
curl http://localhost:8081/api/tests/1 | jq '.'
```

## Security Considerations

### SQL Injection Prevention
- All queries use parameterized bindings with sqlx
- User input is validated before query building
- MySQL connection uses safe default settings
- Table names validated against allowlist in `column_exists()` function

### Input Validation
- Backend validates level, mode, chapters, range values
- Frontend validates form inputs before submission
- Error messages don't expose internal details

## Performance

### Database Size
- ~2.2 MB of data (1760 entries, 10232 questions)
- Fast query performance with InnoDB engine
- Index on entry_id for faster lookups
- RANDOM ordering is efficient for small result sets

### Connection Pooling
- MySQL connection pooling via sqlx
- Configurable pool size through sqlx settings

## Deployment Notes

When deploying:
1. Ensure MySQL server is running and accessible
2. Run `init_database_mysql.sh` to create and populate database
3. Configure `DATABASE_URL` in backend `.env` file
4. Ensure backend has network access to MySQL server
5. Configure MySQL user with appropriate permissions:
   ```sql
   CREATE USER 'appuser'@'localhost' IDENTIFIED BY 'password';
   GRANT ALL PRIVILEGES ON japanese_vocab.* TO 'appuser'@'localhost';
   FLUSH PRIVILEGES;
   ```

## Migration from SQLite

If migrating from an existing SQLite installation:
1. Export data from SQLite database (if needed)
2. Install and configure MySQL
3. Run `init_database_mysql.sh` to set up MySQL database
4. Update `backend/.env` with MySQL connection string
5. Rebuild backend: `cd backend && cargo build --release`
6. Start backend and verify connection

## Summary of Changes

### Bug Fixes
- ✅ Fixed column name mismatch ("text" vs "prompt") in 5 locations
- ✅ Queries now correctly reference the "prompt" column

### Database Migration
- ✅ Migrated from SQLite to MySQL
- ✅ Updated all table schemas for MySQL
- ✅ Updated all SQL queries for MySQL compatibility
- ✅ Created MySQL initialization script
- ✅ Updated documentation for MySQL setup

### Security Improvements
- ✅ Table name validation in column_exists function
- ✅ Parameterized queries throughout
- ✅ Proper foreign key constraints in MySQL
