#!/bin/bash
# Bash script to build and run Veridion Nexus Docker container

echo "🐳 Building Veridion Nexus Docker image..."

# Build the Docker image
docker build -t veridion-nexus:latest .

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "To run the container:"
    echo "  docker run -p 8080:8080 veridion-nexus:latest"
    echo ""
    echo "Or use docker-compose:"
    echo "  docker-compose up"
else
    echo "❌ Build failed!"
    exit 1
fi

