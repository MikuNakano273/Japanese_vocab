# Database Integration - Implementation Notes

## Overview
This document describes the changes made to integrate the `mimikara_n3_questions.db` SQLite database for quiz/test generation.

## Changes Made

### 1. Database Initialization Script (`init_database.sh`)
Created a bash script to initialize the SQLite database from the SQL file:
- **Location**: `init_database.sh` (project root)
- **Purpose**: Creates and populates `backend/data/mimikara_n3_questions.db` from `mimikara_n3_questions.db.sql`
- **Features**:
  - Checks if database already exists and has data
  - Only initializes if needed (idempotent)
  - Provides user-friendly status messages
  - Validates successful initialization

**Usage**:
```bash
./init_database.sh
```

### 2. Backend Database Module Updates (`backend/src/db/mod.rs`)
Enhanced the database initialization to handle pre-existing databases:

**Changes**:
- Added `use sqlx::Row;` import for row operations
- Implemented `column_exists()` helper function to check for missing columns
- Added schema migration logic to add missing columns:
  - `quiz_id`: Links questions to quizzes (nullable)
  - `level`: JLPT level reference (n5=1, n4=2, n3=3, n2=4, n1=5)
  - `chapter`: Chapter number for grouping questions

**Why This Matters**:
The SQL file (`mimikara_n3_questions.db.sql`) creates a questions table without these columns. The backend expects them for filtering and test generation. The migration ensures backward compatibility with pre-existing databases.

### 3. Backend Routes Fix (`backend/src/routes/mod.rs`)
Fixed a compiler warning:
- Removed unnecessary `mut` keyword from `q_any` variable

### 4. Environment Configuration Updates
Updated default configurations to use SQLite:

**`backend/.env.example`**:
- Changed from PostgreSQL to SQLite connection string
- Updated port to 8081 (to match running backend)

**`frontend/.env.example`**:
- Updated BACKEND_URL to point to port 8081

### 5. Documentation Updates (`README.md`)
Enhanced the README with:
- Database initialization instructions
- Database statistics (1760 entries, 10232 questions)
- "Create Test" feature documentation
- Updated usage instructions

## Database Schema

### Tables Created by SQL File
1. **entries** - Vocabulary entries
   - id, list_index, kanji, kana, meaning

2. **questions** - Quiz questions (initially)
   - id, entry_id, q_type, prompt, correct_answer, options, correct_index, created_at

### Columns Added by Backend
The backend adds these columns on first run if they don't exist:
- **quiz_id**: INTEGER (references quizzes table)
- **level**: INTEGER (references n_level table)
- **chapter**: INTEGER (chapter grouping)

### Additional Tables Created by Backend
- **quizzes**: User-created quizzes
- **n_level**: JLPT level mapping (1→n5, 2→n4, 3→n3, 4→n2, 5→n1)
- **tests**: Generated test storage

## Error Handling

### Missing Database
- Backend creates empty database file if missing
- `init_database.sh` populates it from SQL file

### Empty Database
- `init_database.sh` detects empty database and repopulates

### Invalid Schema
- Backend automatically adds missing columns
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
1. Initialize database: `./init_database.sh`
2. Start backend: `cd backend && cargo run`
3. Start frontend: `cd frontend && npm start`
4. Navigate to `http://localhost:3000/test/create`
5. Select options and create test
6. Verify test is generated and displayed

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
- All queries use parameterized bindings
- User input is validated before query building
- SQLite connection uses safe default settings

### Input Validation
- Backend validates level, mode, chapters, range values
- Frontend validates form inputs before submission
- Error messages don't expose internal details

## Performance

### Database Size
- ~2.2 MB database file
- Fast query performance with indexes on entry_id and q_type
- RANDOM ordering is efficient for small result sets

### Caching
- No caching currently implemented
- SQLite connection pooling via sqlx

## Deployment Notes

When deploying:
1. Ensure `init_database.sh` is run before first backend start
2. Database file is in `backend/data/` directory
3. Backend has write permissions for the database file
4. SQLite library is available on the system
