#!/bin/bash

# FFmpeg Rust Converter - Run Script
# This script builds and runs the FFmpeg converter application

set -e

echo "FFmpeg Rust Converter"
echo "====================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo is not installed. Please install Rust first:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if FFmpeg is available
if ! command -v ffmpeg &> /dev/null; then
    echo "Warning: FFmpeg is not found in PATH."
    echo "Please install FFmpeg before using the converter:"
    echo ""
    echo "Ubuntu/Debian: sudo apt install ffmpeg"
    echo "macOS:         brew install ffmpeg"
    echo "Windows:       Download from https://ffmpeg.org/"
    echo ""
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Build and run the application
echo "Building application..."
cargo build --release

echo "Starting FFmpeg Converter..."
./target/release/ffmpegrust