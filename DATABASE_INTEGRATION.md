# Database Integration - Implementation Notes

## Overview
This document describes the database integration using SQLite and bug fixes made to the database integration.

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

### 2. Database System: SQLite

#### Cargo.toml Configuration
- Uses sqlx with `["sqlite", ...]` features

#### Backend Database Module (`backend/src/db/mod.rs`)
**Key Features**:
- Uses `SqlitePool` for connection pooling
- Table creation syntax for SQLite:
  - `INTEGER PRIMARY KEY AUTOINCREMENT` for auto-incrementing IDs
  - `TEXT DEFAULT (datetime('now'))` for timestamps
  - Foreign key constraints supported
- Uses `INSERT OR IGNORE` for idempotent inserts
- Column existence checks using `PRAGMA table_info()`

#### Backend Main File (`backend/src/main.rs`)
**Features**:
- SQLite file-based connection using `DATABASE_URL` environment variable
- Default database file: `mimikara_n3_questions.db`
- Connection using `SqlitePool::connect()`
- Automatic database file creation

#### Backend Routes (`backend/src/routes/mod.rs`)
**Key Details**:
- All handlers use `web::Data<SqlitePool>`
- Uses `SqliteRow` for row types
- Uses `last_insert_rowid()` for SQLite compatibility
- `ORDER BY RANDOM()` for random question selection

### 3. Database Initialization Script (`init_database.sh`)
A bash script to initialize the SQLite database from the SQL file:
- **Location**: `init_database.sh` (project root)
- **Purpose**: Creates and populates SQLite `mimikara_n3_questions.db` file from `mimikara_n3_questions.db.sql`
- **Features**:
  - Creates backend data directory if needed
  - Checks if database already exists and has data
  - Only initializes if needed (idempotent)
  - Provides user-friendly status messages
  - Validates successful initialization

**Usage**:
```bash
./init_database.sh
```

### 4. Environment Configuration
SQLite configuration using file-based database:

**`backend/.env.example`**:
- Uses SQLite connection string
- Format: `sqlite://filename.db`
- Default: `sqlite://mimikara_n3_questions.db`

### 5. Documentation Updates (`README.md`)
Enhanced the README with SQLite setup instructions:
- Updated architecture section to mention SQLite
- Removed MySQL from prerequisites
- Simplified setup steps (no database server needed)
- Updated manual database inspection with SQLite CLI
- Updated all configuration examples
- Updated database management section

## Database Schema

### Tables in SQLite Database
1. **entries** - Vocabulary entries
   - id: INTEGER PRIMARY KEY AUTOINCREMENT
   - list_index: INTEGER
   - kanji: TEXT
   - kana: TEXT
   - meaning: TEXT

2. **quizzes** - User-created quizzes
   - id: INTEGER PRIMARY KEY AUTOINCREMENT
   - title: TEXT NOT NULL
   - description: TEXT
   - created_at: TEXT DEFAULT (datetime('now'))

3. **n_level** - JLPT level mapping
   - id: INTEGER PRIMARY KEY (1→n5, 2→n4, 3→n3, 4→n2, 5→n1)
   - level: TEXT NOT NULL

4. **questions** - Quiz questions
   - id: INTEGER PRIMARY KEY AUTOINCREMENT
   - entry_id: INTEGER (FK → entries.id)
   - quiz_id: INTEGER (FK → quizzes.id, nullable)
   - q_type: TEXT
   - prompt: TEXT (Note: Previously incorrectly referenced as "text" in code)
   - correct_answer: TEXT
   - options: TEXT (JSON string)
   - correct_index: INTEGER
   - level: INTEGER (FK → n_level.id)
   - chapter: INTEGER
   - created_at: TEXT DEFAULT (datetime('now'))

5. **tests** - Generated tests
   - id: INTEGER PRIMARY KEY AUTOINCREMENT
   - title: TEXT
   - questions: TEXT (JSON string)
   - created_at: TEXT DEFAULT (datetime('now'))

### Indexes
- `idx_entry_id` on questions(entry_id) for faster lookups

### Foreign Key Constraints
- questions.entry_id → entries.id (ON DELETE CASCADE)
- questions.quiz_id → quizzes.id (ON DELETE SET NULL)
- questions.level → n_level.id

All tables use SQLite's default storage with UTF-8 encoding.

### Backend Schema Migration
The backend automatically adds missing columns to the questions table on first run if they don't exist:
- **quiz_id**: INTEGER (FK to quizzes table)
- **level**: INTEGER (FK to n_level table)
- **chapter**: INTEGER (for chapter grouping)

This ensures backward compatibility with databases created from the SQL file.

## Error Handling

### Missing Database
- Backend creates database file automatically if it doesn't exist
- `init_database.sh` can pre-populate the database from SQL file

### Empty Database
- `init_database.sh` detects empty database and populates it
- Backend automatically initializes required tables

### Invalid Schema
- Backend automatically adds missing columns using `column_exists()` helper
- Uses PRAGMA table_info to detect existing columns
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
1. Optionally initialize database: `./init_database.sh` (or let backend create it)
2. Configure backend: Update `backend/.env` if needed (default SQLite settings work out of box)
3. Start backend: `cd backend && cargo run`
4. Start frontend: `cd frontend && npm start`
5. Navigate to `http://localhost:3000/test/create`
6. Select options and create test
7. Verify test is generated and displayed

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
- SQLite connection uses safe default settings
- Table names validated against allowlist in `column_exists()` function

### Input Validation
- Backend validates level, mode, chapters, range values
- Frontend validates form inputs before submission
- Error messages don't expose internal details

## Performance

### Database Size
- ~2.2 MB of data (1760 entries, 10232 questions)
- Fast query performance with SQLite
- Index on entry_id for faster lookups
- RANDOM ordering is efficient for small result sets

### Connection Pooling
- SQLite connection pooling via sqlx
- Configurable pool size through sqlx settings

## Deployment Notes

When deploying:
1. Ensure the backend has write permissions to create the database file
2. Run `init_database.sh` to pre-populate database (optional)
3. Configure `DATABASE_URL` in backend `.env` file
4. The database file will be created automatically on first run if it doesn't exist

## Summary of Changes

### Bug Fixes
- ✅ Fixed column name mismatch ("text" vs "prompt") in 5 locations
- ✅ Queries now correctly reference the "prompt" column

### Database System
- ✅ Using SQLite for file-based database storage
- ✅ All table schemas optimized for SQLite
- ✅ All SQL queries use SQLite-compatible syntax
- ✅ Initialization script supports SQLite
- ✅ Documentation updated for SQLite setup

### Security Improvements
- ✅ Table name validation in column_exists function
- ✅ Parameterized queries throughout
- ✅ Proper foreign key constraints in SQLite
