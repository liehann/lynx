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
    
    # Load environment variables to show correct URLs
    source .env 2>/dev/null || true
    PORT=${PORT:-3000}
    ADMIN_HOST=${ADMIN_HOST:-lynx}
    DEFAULT_REDIRECT_HOST=${DEFAULT_REDIRECT_HOST:-go}
    
    echo "Admin UI will be available at: http://${ADMIN_HOST}:${PORT}"
    echo "Redirector will handle requests to: http://${DEFAULT_REDIRECT_HOST}:${PORT}"
    echo ""
    echo "Press Ctrl+C to stop the server"
    echo ""
    
    # Run the server
    cargo run --release
else
    echo "❌ Build failed"
    exit 1
fi
