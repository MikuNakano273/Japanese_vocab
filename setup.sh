#!/bin/bash

# Quick Start Script for Japanese Vocab Quiz Application

echo "🌸 Japanese Vocab Quiz - Quick Start"
echo "===================================="
echo ""

# Using SQLite (file-based) — no separate DB server required
# Check for sqlite3 CLI for convenience (optional; backend will create DB file even if sqlite3 isn't installed)
if ! command -v sqlite3 &> /dev/null; then
    echo "⚠️ sqlite3 CLI not found — that's okay; the backend will create the database file automatically. Install sqlite3 if you want CLI access."
else
    echo "✅ sqlite3 CLI found"
fi

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed or not in PATH"
    echo "Please install Node.js first."
    exit 1
fi

echo "✅ Node.js found ($(node --version))"

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed or not in PATH"
    echo "Please install Rust first: https://rustup.rs/"
    exit 1
fi

echo "✅ Rust found ($(rustc --version))"

# Setup environment files
echo ""
echo "📝 Setting up environment files..."

if [ ! -f backend/.env ]; then
    cp backend/.env.example backend/.env
    echo "✅ Backend .env created"
else
    echo "⚠️  Backend .env already exists"
fi

if [ ! -f frontend/.env ]; then
    cp frontend/.env.example frontend/.env
    echo "✅ Frontend .env created"
else
    echo "⚠️  Frontend .env already exists"
fi

# Install frontend dependencies
echo ""
echo "📦 Installing frontend dependencies..."
cd frontend && npm install
cd ..
echo "✅ Frontend dependencies installed"

# Build Rust backend
echo ""
echo "🔨 Building Rust backend..."
cd backend && cargo build --release
cd ..
echo "✅ Backend built successfully"

echo ""
echo "✅ Setup complete!"
echo ""
echo "To start the application:"
echo "1. Start backend (it will create/init the SQLite DB file `mimikara_n3_question.db` if needed): cd backend && cargo run"
echo "2. Start frontend (in new terminal): cd frontend && npm start"
echo "3. Open browser: http://localhost:3000"
echo ""
echo "Happy learning! 🌸"
