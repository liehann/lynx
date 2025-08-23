#!/bin/bash

echo "Starting Lynx Server"
echo "==================="

# Check if .env file exists
if [ ! -f ".env" ]; then
    echo "❌ .env file not found!"
    echo ""
    echo "Please create a .env file with your configuration:"
    echo "  cp env.example .env"
    echo ""
    echo "Then edit .env with your actual database credentials:"
    echo "  DATABASE_URL=postgresql://username:password@goblin/lynx_prod"
    echo "  ADMIN_HOST=lynx"
    echo "  DEFAULT_REDIRECT_HOST=go"
    echo ""
    exit 1
fi

echo "✓ Found .env file"
echo ""

# Build the project
echo "Building project..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✓ Build successful"
    echo ""
    echo "Starting server..."
    echo "Admin UI will be available at: http://lynx:3000"
    echo "Redirector will handle requests to: http://go:3000"
    echo ""
    echo "Press Ctrl+C to stop the server"
    echo ""
    
    # Run the server
    cargo run --release
else
    echo "❌ Build failed"
    exit 1
fi
