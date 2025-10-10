#!/bin/bash
# Script to demonstrate different logging levels

echo "==================================================================="
echo "Testing Structured Logging with Tracing Framework"
echo "==================================================================="
echo ""

echo "1. Testing CLI mode (user output should remain unchanged):"
echo "-------------------------------------------------------------------"
echo -e "50\n" | cargo run --quiet -- --min 1 --max 100 --limit 5 2>&1 | head -15
echo ""

echo "==================================================================="
echo "2. Testing Web Server with INFO level (default):"
echo "-------------------------------------------------------------------"
echo "Starting database..."
make dev-db > /dev/null 2>&1
sleep 2

echo ""
echo "Starting web server with RUST_LOG=info..."
timeout 2 env RUST_LOG=number_guessing_game=info cargo run --quiet -- --server 2>&1 | grep -E "INFO|ERROR|WARN" || true
echo ""

echo "==================================================================="
echo "3. Testing Web Server with ERROR level (minimal output):"
echo "-------------------------------------------------------------------"
echo "Starting web server with RUST_LOG=error..."
timeout 2 env RUST_LOG=error cargo run --quiet -- --server 2>&1 | grep -E "INFO|ERROR|WARN" || true
echo "(Notice: No INFO logs shown, only errors would appear)"
echo ""

echo "==================================================================="
echo "Cleaning up..."
make dev-down > /dev/null 2>&1
echo "Done!"
echo "==================================================================="
