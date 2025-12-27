#!/bin/bash

# Start script for Japanese Vocab Quiz Application

echo "🌸 Starting Japanese Vocab Quiz Application"
echo "==========================================="
echo ""

# Function to check if a port is in use
check_port() {
    if lsof -Pi :$1 -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Check if backend is already running
if check_port 8080; then
    echo "⚠️  Backend server is already running on port 8080"
else
    echo "🚀 Starting backend server..."
    cd backend
    cargo run &
    BACKEND_PID=$!
    cd ..
    echo "✅ Backend server started (PID: $BACKEND_PID)"
fi

# Wait a bit for backend to start
sleep 2

# Check if frontend is already running
if check_port 3000; then
    echo "⚠️  Frontend server is already running on port 3000"
else
    echo "🚀 Starting frontend server..."
    cd frontend
    npm start &
    FRONTEND_PID=$!
    cd ..
    echo "✅ Frontend server started (PID: $FRONTEND_PID)"
fi

echo ""
echo "✅ Application is running!"
echo "   - Frontend: http://localhost:3000"
echo "   - Backend API: http://localhost:8080"
echo ""
echo "Press Ctrl+C to stop all servers"
echo ""

# Wait for both processes
wait
