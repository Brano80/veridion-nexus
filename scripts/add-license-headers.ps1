# Add AGPL-3.0 license headers to all Rust and Python source files
# Usage: .\scripts\add-license-headers.ps1

$rustHeader = @"
// Copyright (c) 2025 Veridion Nexus.
// Licensed under the AGPL-3.0 license.

"@

$pythonHeader = @"
# Copyright (c) 2025 Veridion Nexus.
# Licensed under the AGPL-3.0 license.

"@

function Add-LicenseHeader {
    param (
        [string]$FilePath,
        [string]$Header
    )
    
    # Check if file already has a license header
    $content = Get-Content -Path $FilePath -Raw
    if ($content -match "(?s)^(//|#)\s*Copyright.*AGPL") {
        Write-Host "Skipping $FilePath (already has license header)" -ForegroundColor Yellow
        return
    }
    
    # Read existing content
    $existingContent = Get-Content -Path $FilePath -Raw
    
    # Prepend header
    $newContent = $Header + $existingContent
    
    # Write back to file
    Set-Content -Path $FilePath -Value $newContent -NoNewline
    Write-Host "Added license header to $FilePath" -ForegroundColor Green
}

# Find all Rust files
$rustFiles = Get-ChildItem -Path . -Include *.rs -Recurse -File | Where-Object {
    $_.FullName -notmatch "\\target\\" -and
    $_.FullName -notmatch "\\.git\\"
}

# Find all Python files
$pythonFiles = Get-ChildItem -Path . -Include *.py -Recurse -File | Where-Object {
    $_.FullName -notmatch "\\__pycache__\\" -and
    $_.FullName -notmatch "\\.git\\" -and
    $_.FullName -notmatch "\\venv\\" -and
    $_.FullName -notmatch "\\env\\"
}

Write-Host "Found $($rustFiles.Count) Rust files and $($pythonFiles.Count) Python files" -ForegroundColor Cyan

# Add headers to Rust files
foreach ($file in $rustFiles) {
    Add-LicenseHeader -FilePath $file.FullName -Header $rustHeader
}

# Add headers to Python files
foreach ($file in $pythonFiles) {
    Add-LicenseHeader -FilePath $file.FullName -Header $pythonHeader
}

Write-Host "`nLicense header addition complete!" -ForegroundColor Green

