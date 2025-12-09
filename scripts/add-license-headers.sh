#!/bin/bash
# Add AGPL-3.0 license headers to all Rust and Python source files
# Usage: ./scripts/add-license-headers.sh

RUST_HEADER="// Copyright (c) 2025 Veridion Nexus.
// Licensed under the AGPL-3.0 license.
"

PYTHON_HEADER="# Copyright (c) 2025 Veridion Nexus.
# Licensed under the AGPL-3.0 license.
"

add_license_header() {
    local file="$1"
    local header="$2"
    
    # Check if file already has a license header
    if head -n 5 "$file" | grep -qE "^(//|#)\s*Copyright.*AGPL"; then
        echo "Skipping $file (already has license header)"
        return
    fi
    
    # Create temporary file with header
    local temp_file=$(mktemp)
    echo -n "$header" > "$temp_file"
    cat "$file" >> "$temp_file"
    
    # Replace original file
    mv "$temp_file" "$file"
    echo "Added license header to $file"
}

# Find all Rust files (excluding target and .git directories)
find . -type f -name "*.rs" \
    ! -path "*/target/*" \
    ! -path "*/.git/*" \
    -print0 | while IFS= read -r -d '' file; do
    add_license_header "$file" "$RUST_HEADER"
done

# Find all Python files (excluding common virtualenv directories)
find . -type f -name "*.py" \
    ! -path "*/__pycache__/*" \
    ! -path "*/.git/*" \
    ! -path "*/venv/*" \
    ! -path "*/env/*" \
    -print0 | while IFS= read -r -d '' file; do
    add_license_header "$file" "$PYTHON_HEADER"
done

echo ""
echo "License header addition complete!"

