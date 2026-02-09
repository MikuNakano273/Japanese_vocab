@echo off
title Japanese Vocab Backend

cd /d "%~dp0backend"

echo Starting Rust backend...
cargo run

pause
