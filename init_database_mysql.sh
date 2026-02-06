#!/usr/bin/env bash

# Database initialization script for Japanese Vocabulary Quiz Application (MySQL version)
# This script creates the MySQL database and populates it from the SQL file

set -e

DB_NAME="japanese_vocab"
SQL_FILE="mimikara_n3_questions.db.sql"
MYSQL_USER="${MYSQL_USER:-root}"
MYSQL_PASSWORD="${MYSQL_PASSWORD:-password}"
MYSQL_HOST="${MYSQL_HOST:-localhost}"
MYSQL_PORT="${MYSQL_PORT:-3306}"

echo "🗄️  Initializing MySQL database..."

# Check if MySQL is accessible
if ! command -v mysql &> /dev/null; then
    echo "❌ Error: mysql client not found!"
    echo "   Please install MySQL client first."
    exit 1
fi

# Check if we can connect to MySQL
if ! mysql -h"$MYSQL_HOST" -P"$MYSQL_PORT" -u"$MYSQL_USER" -p"$MYSQL_PASSWORD" -e "SELECT 1;" &> /dev/null; then
    echo "❌ Error: Cannot connect to MySQL server!"
    echo "   Please check your MySQL credentials and ensure MySQL is running."
    exit 1
fi

# Create database if it doesn't exist
echo "📁 Creating database '$DB_NAME' if it doesn't exist..."
mysql -h"$MYSQL_HOST" -P"$MYSQL_PORT" -u"$MYSQL_USER" -p"$MYSQL_PASSWORD" -e "CREATE DATABASE IF NOT EXISTS $DB_NAME CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"

# Check if database has data
ENTRY_COUNT=$(mysql -h"$MYSQL_HOST" -P"$MYSQL_PORT" -u"$MYSQL_USER" -p"$MYSQL_PASSWORD" -D"$DB_NAME" -s -N -e "SELECT COUNT(*) FROM entries;" 2>/dev/null || echo "0")
QUESTION_COUNT=$(mysql -h"$MYSQL_HOST" -P"$MYSQL_PORT" -u"$MYSQL_USER" -p"$MYSQL_PASSWORD" -D"$DB_NAME" -s -N -e "SELECT COUNT(*) FROM questions;" 2>/dev/null || echo "0")

if [ "$ENTRY_COUNT" -gt "0" ] && [ "$QUESTION_COUNT" -gt "0" ]; then
    echo "✅ Database already initialized with $ENTRY_COUNT entries and $QUESTION_COUNT questions."
    exit 0
fi

# Check if SQL file exists
if [ ! -f "$SQL_FILE" ]; then
    echo "❌ Error: SQL file '$SQL_FILE' not found!"
    echo "   Please ensure mimikara_n3_questions.db.sql is in the project root directory."
    exit 1
fi

# Convert SQLite SQL to MySQL SQL and import
echo "📝 Converting and importing data from SQL file..."

# Create a temporary MySQL-compatible SQL file
TEMP_SQL="/tmp/mysql_import_$$.sql"

# Convert the SQL file from SQLite to MySQL format
cat "$SQL_FILE" | \
    sed 's/BEGIN TRANSACTION;/START TRANSACTION;/g' | \
    sed 's/SERIAL PRIMARY KEY/INT AUTO_INCREMENT PRIMARY KEY/g' | \
    sed 's/TEXT/TEXT/g' | \
    sed 's/INTEGER/INT/g' | \
    sed 's/TIMESTAMP/DATETIME/g' | \
    sed 's/COMMIT;/COMMIT;/g' | \
    grep -v "^PRAGMA" > "$TEMP_SQL"

# Import the converted SQL
mysql -h"$MYSQL_HOST" -P"$MYSQL_PORT" -u"$MYSQL_USER" -p"$MYSQL_PASSWORD" "$DB_NAME" < "$TEMP_SQL"

# Clean up temporary file
rm -f "$TEMP_SQL"

# Verify database creation
ENTRY_COUNT=$(mysql -h"$MYSQL_HOST" -P"$MYSQL_PORT" -u"$MYSQL_USER" -p"$MYSQL_PASSWORD" -D"$DB_NAME" -s -N -e "SELECT COUNT(*) FROM entries;" 2>/dev/null || echo "0")
QUESTION_COUNT=$(mysql -h"$MYSQL_HOST" -P"$MYSQL_PORT" -u"$MYSQL_USER" -p"$MYSQL_PASSWORD" -D"$DB_NAME" -s -N -e "SELECT COUNT(*) FROM questions;" 2>/dev/null || echo "0")

if [ "$ENTRY_COUNT" -gt "0" ] && [ "$QUESTION_COUNT" -gt "0" ]; then
    echo "✅ Database successfully created!"
    echo "   📊 Stats: $ENTRY_COUNT entries, $QUESTION_COUNT questions"
else
    echo "❌ Error: Database creation failed or database is empty."
    exit 1
fi
