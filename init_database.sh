#!/usr/bin/env bash

# Database initialization script for Japanese Vocabulary Quiz Application
# This script creates and populates the SQLite database from the SQL file if needed

set -e

DB_DIR="backend/data"
DB_FILE="$DB_DIR/mimikara_n3_questions.db"
SQL_FILE="mimikara_n3_questions.db.sql"

echo "🗄️  Initializing database..."

# Create backend data directory if it doesn't exist
if [ ! -d "$DB_DIR" ]; then
    echo "📁 Creating backend data directory..."
    mkdir -p "$DB_DIR"
fi

# Check if database file exists and has data
if [ -f "$DB_FILE" ]; then
    # Check if database has entries table with data
    ENTRY_COUNT=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM entries;" 2>/dev/null || echo "0")
    QUESTION_COUNT=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM questions;" 2>/dev/null || echo "0")
    
    if [ "$ENTRY_COUNT" -gt "0" ] && [ "$QUESTION_COUNT" -gt "0" ]; then
        echo "✅ Database already initialized with $ENTRY_COUNT entries and $QUESTION_COUNT questions."
        exit 0
    else
        echo "⚠️  Database exists but is empty. Reinitializing..."
        rm -f "$DB_FILE"
    fi
fi

# Check if SQL file exists
if [ ! -f "$SQL_FILE" ]; then
    echo "❌ Error: SQL file '$SQL_FILE' not found!"
    echo "   Please ensure mimikara_n3_questions.db.sql is in the project root directory."
    exit 1
fi

# Create database from SQL file
echo "📝 Creating database from SQL file..."
sqlite3 "$DB_FILE" < "$SQL_FILE"

# Verify database creation
ENTRY_COUNT=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM entries;" 2>/dev/null || echo "0")
QUESTION_COUNT=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM questions;" 2>/dev/null || echo "0")

if [ "$ENTRY_COUNT" -gt "0" ] && [ "$QUESTION_COUNT" -gt "0" ]; then
    echo "✅ Database successfully created in $DB_FILE!"
    echo "   📊 Stats: $ENTRY_COUNT entries, $QUESTION_COUNT questions"
else
    echo "❌ Error: Database creation failed or database is empty."
    exit 1
fi
