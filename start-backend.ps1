# Simple script to start backend
Write-Host "Starting Veridion Nexus Backend..." -ForegroundColor Cyan
Write-Host ""

# Set environment variables
$env:DATABASE_URL = "postgresql://veridion:veridion_secure_pass_2024@localhost:5432/veridion_nexus"
$env:RUST_LOG = "info"
$env:VERIDION_MASTER_KEY = "veridion_master_key_change_in_production_2024"
$env:JWT_SECRET = "jwt_secret_change_in_production_2024"

# Check if database is running
Write-Host "Checking database..." -ForegroundColor Yellow
try {
    docker-compose ps postgres | Select-String "Up" | Out-Null
    Write-Host "✅ Database is running" -ForegroundColor Green
} catch {
    Write-Host "❌ Database is NOT running" -ForegroundColor Red
    Write-Host "Starting database..." -ForegroundColor Yellow
    docker-compose up -d postgres
    Start-Sleep -Seconds 5
}

# Start backend
Write-Host "`nStarting backend..." -ForegroundColor Yellow
Write-Host "This will open a new window." -ForegroundColor Gray
Write-Host "Wait for: 'Veridion Nexus API starting on port 8080'" -ForegroundColor Cyan
Write-Host ""

Start-Process powershell -ArgumentList @("-NoExit", "-Command", "cd '$PWD'; `$env:DATABASE_URL = 'postgresql://veridion:veridion_secure_pass_2024@localhost:5432/veridion_nexus'; `$env:RUST_LOG = 'info'; `$env:VERIDION_MASTER_KEY = 'veridion_master_key_change_in_production_2024'; `$env:JWT_SECRET = 'jwt_secret_change_in_production_2024'; Write-Host '[BACKEND] Starting Veridion Nexus API...' -ForegroundColor Green; Write-Host ''; cargo run")

Write-Host "✅ Backend is starting in a new window" -ForegroundColor Green
Write-Host "`nPlease wait 30-60 seconds for compilation and startup" -ForegroundColor Yellow
Write-Host "`nOnce you see 'Veridion Nexus API starting on port 8080' in the backend window:" -ForegroundColor Cyan
Write-Host "  1. Refresh the Wizard page in your browser (F5)" -ForegroundColor White
Write-Host "  2. The 'Failed to fetch' error should disappear" -ForegroundColor White
Write-Host "`nTo check if backend is ready, run: .\check-backend.ps1" -ForegroundColor Gray

