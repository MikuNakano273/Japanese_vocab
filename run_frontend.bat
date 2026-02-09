@echo off
title Japanese Vocab Frontend

cd /d "%~dp0frontend"

if not exist node_modules (
    echo Installing frontend dependencies...
    npm install
)

echo Starting frontend UI...
npm start

pause
