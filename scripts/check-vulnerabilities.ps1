# Dependency Vulnerability Scanning Script (PowerShell)
# This script uses cargo-audit to check for known vulnerabilities in dependencies

Write-Host "🔍 Checking for dependency vulnerabilities..." -ForegroundColor Cyan

# Check if cargo-audit is installed
$cargoAuditInstalled = Get-Command cargo-audit -ErrorAction SilentlyContinue

if (-not $cargoAuditInstalled) {
    Write-Host "⚠️  cargo-audit not found. Installing..." -ForegroundColor Yellow
    cargo install cargo-audit --locked
}

# Run cargo audit
Write-Host "📊 Running cargo audit..." -ForegroundColor Cyan
cargo audit

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ No known vulnerabilities found!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "❌ Vulnerabilities found! Please review the output above." -ForegroundColor Red
    exit 1
}

