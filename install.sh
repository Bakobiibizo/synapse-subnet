#!/bin/bash

# Exit on error
set -e

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "Docker is not installed. Please install Docker first."
    exit 1
fi

# Check required environment variables
required_vars=("OPENAI_API_KEY" "ANTHROPIC_API_KEY" "HUGGINGFACE_API_KEY" "VALIDATOR_KEY")
for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        echo "Error: $var is not set"
        exit 1
    fi
done

# Build zangief Docker image
echo "Building zangief Docker image..."
cd .zangief
docker build -t zangief:latest .
cd ..

# Start the registrar
echo "Starting registrar..."
cargo run --bin registrar &
REGISTRAR_PID=$!

# Wait for registrar to start
echo "Waiting for registrar to start..."
sleep 5

# Register zangief module
echo "Registering zangief module..."
cargo run --bin validator -- register --config config.yaml

# Start the validator
echo "Starting validator..."
cargo run --bin validator -- start --config config.yaml

# Cleanup function
cleanup() {
    echo "Cleaning up..."
    kill $REGISTRAR_PID
    docker stop zangief || true
    docker rm zangief || true
}

# Set up cleanup on script exit
trap cleanup EXIT

# Keep running until interrupted
echo "Setup complete! Press Ctrl+C to stop..."
wait
