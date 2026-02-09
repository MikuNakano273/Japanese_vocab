@echo off
title Japanese Vocab Launcher

echo ===============================
echo   Japanese Vocab Launcher
echo ===============================

:: Check Rust
rustc --version >nul 2>&1 || (
    echo ❌ Rust not found
    pause
    exit /b
)

:: Check Node
node --version >nul 2>&1 || (
    echo ❌ Node.js not found
    pause
    exit /b
)

echo.
echo 🚀 Launching backend...
start "Backend" "%~dp0run_backend.bat"

timeout /t 4 >nul

echo 🌐 Launching frontend...
start "Frontend" "%~dp0run_frontend.bat"

timeout /t 5 >nul

echo 🌍 Opening browser...
start http://localhost:3000

echo.
echo ✅ All services launched
echo ❗ Do not close backend/frontend windows
echo.

pause
