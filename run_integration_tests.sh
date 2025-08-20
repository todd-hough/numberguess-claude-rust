#!/bin/bash
# Simple integration test runner for the Number Guessing Game

set -e

# Build the application first
echo "Building application..."
cargo build

# Run the integration tests
echo "Running integration tests..."
cargo test --test integration_test

echo "Integration tests completed!"