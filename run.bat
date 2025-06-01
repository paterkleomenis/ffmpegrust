@echo off
rem FFmpeg Rust Converter - Windows Run Script
rem This script builds and runs the FFmpeg converter application

echo FFmpeg Rust Converter
echo =====================

rem Check if Rust is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: Rust/Cargo is not installed. Please install Rust first:
    echo https://rustup.rs/
    pause
    exit /b 1
)

rem Check if FFmpeg is available
where ffmpeg >nul 2>nul
if %errorlevel% neq 0 (
    echo Warning: FFmpeg is not found in PATH.
    echo Please install FFmpeg before using the converter:
    echo.
    echo Download from https://ffmpeg.org/download.html
    echo Extract to a folder and add to your system PATH
    echo.
    set /p continue="Continue anyway? (y/N): "
    if /i not "%continue%"=="y" (
        exit /b 1
    )
)

rem Build and run the application
echo Building application...
cargo build --release

if %errorlevel% neq 0 (
    echo Build failed!
    pause
    exit /b 1
)

echo Starting FFmpeg Converter...
target\release\ffmpegrust.exe

pause