#!/bin/bash

# Quick Start Script for Japanese Vocab Quiz Application

echo "🌸 Japanese Vocab Quiz - Quick Start"
echo "===================================="
echo ""

# Check if PostgreSQL is running
if ! command -v psql &> /dev/null; then
    echo "❌ PostgreSQL is not installed or not in PATH"
    echo "Please install PostgreSQL first."
    exit 1
fi

echo "✅ PostgreSQL found"

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
echo "1. Start PostgreSQL and run: psql -U postgres -f database/init.sql"
echo "2. Start backend: cd backend && cargo run"
echo "3. Start frontend (in new terminal): cd frontend && npm start"
echo "4. Open browser: http://localhost:3000"
echo ""
echo "Happy learning! 🌸"
