#!/bin/bash

echo "Running Lynx Tests"
echo "=================="

echo "1. Running unit tests (no database required)..."
cargo test --lib

echo ""
echo "2. Running integration tests (requires test database)..."
echo "   Make sure your test database is set up and env.test has correct credentials"

if [ -f ".env.test" ]; then
    echo "   Found .env.test file ✓"
    cargo test --test integration_tests
else
    echo "   ❌ env.test file not found. Please create it with your test database credentials."
    echo "   Example:"
    echo "   DATABASE_URL=postgresql://username:password@goblin/lynx_test"
    echo "   ADMIN_HOST=lynx"
    echo "   DEFAULT_REDIRECT_HOST=go"
fi
