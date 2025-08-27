#!/bin/bash
# Integration test runner for the Number Guessing Game

set -e

# Build the application first
echo "Building application..."
cargo build

# Run the API integration tests
echo "Running API integration tests..."
cargo test --test integration_test

# Run the Web UI integration tests
echo "Running Web UI integration tests..."
echo "(Note: These tests require Docker with Selenium container)"
cargo test --test web_ui_test

echo "Integration tests completed!"