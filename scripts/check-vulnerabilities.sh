#!/bin/bash

# Dependency Vulnerability Scanning Script
# This script uses cargo-audit to check for known vulnerabilities in dependencies

set -e

echo "🔍 Checking for dependency vulnerabilities..."

# Check if cargo-audit is installed
if ! command -v cargo-audit &> /dev/null; then
    echo "⚠️  cargo-audit not found. Installing..."
    cargo install cargo-audit --locked
fi

# Run cargo audit
echo "📊 Running cargo audit..."
cargo audit

if [ $? -eq 0 ]; then
    echo "✅ No known vulnerabilities found!"
    exit 0
else
    echo "❌ Vulnerabilities found! Please review the output above."
    exit 1
fi

