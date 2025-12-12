# Veridion Nexus Platform Startup Script
# This script starts all components of the Veridion Nexus platform

Write-Host "[START] Starting Veridion Nexus Platform..." -ForegroundColor Cyan
Write-Host ""

# Navigate to project root
$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $projectRoot

# Set environment variables
Write-Host "[CONFIG] Setting environment variables..." -ForegroundColor Yellow
$env:POSTGRES_USER = "veridion"
$env:POSTGRES_PASSWORD = "veridion_secure_pass_2024"
$env:POSTGRES_DB = "veridion_nexus"
$env:VERIDION_MASTER_KEY = "veridion_master_key_change_in_production_2024"
$env:JWT_SECRET = "jwt_secret_change_in_production_2024"
$env:RUST_LOG = "info"

# Check if Docker is running
Write-Host "`n[DOCKER] Checking Docker..." -ForegroundColor Yellow
try {
    docker ps | Out-Null
    Write-Host "[OK] Docker is running" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Docker is not running. Please start Docker Desktop first." -ForegroundColor Red
    exit 1
}

# Start Docker containers (only PostgreSQL, not API)
Write-Host "`n[DOCKER] Starting Docker containers (PostgreSQL only)..." -ForegroundColor Yellow
# Stop API container if running
docker-compose stop veridion-nexus-api 2>$null
# Start only PostgreSQL
docker-compose up -d postgres

if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Failed to start Docker containers" -ForegroundColor Red
    exit 1
}

# Wait for database to be ready
Write-Host "[DATABASE] Waiting for database to be ready..." -ForegroundColor Yellow
$maxAttempts = 30
$attempt = 0
$dbReady = $false

while ($attempt -lt $maxAttempts -and -not $dbReady) {
    Start-Sleep -Seconds 2
    $attempt++
    try {
        $result = docker-compose exec -T postgres pg_isready -U veridion -d veridion_nexus 2>&1
        if ($result -match "accepting connections") {
            $dbReady = $true
            Write-Host "`n[OK] Database is ready!" -ForegroundColor Green
        }
    } catch {
        # Continue waiting
    }
    Write-Host "." -NoNewline -ForegroundColor Gray
}

if (-not $dbReady) {
    Write-Host "`n[ERROR] Database did not become ready in time" -ForegroundColor Red
    exit 1
}

# Check containers status
Write-Host "`n[STATUS] Docker containers status:" -ForegroundColor Green
docker-compose ps

# Set DATABASE_URL for backend (using the same password as Docker)
$env:DATABASE_URL = "postgresql://veridion:veridion_secure_pass_2024@localhost:5432/veridion_nexus"
$env:RUST_LOG = "info"
$env:VERIDION_MASTER_KEY = "veridion_master_key_change_in_production_2024"
$env:JWT_SECRET = "jwt_secret_change_in_production_2024"

# Start Backend (in new window)
Write-Host "`n[BACKEND] Starting Backend API in new window..." -ForegroundColor Yellow
$backendScript = "cd '$projectRoot'; `$env:DATABASE_URL = 'postgresql://veridion:veridion_secure_pass_2024@localhost:5432/veridion_nexus'; `$env:RUST_LOG = 'info'; `$env:VERIDION_MASTER_KEY = 'veridion_master_key_change_in_production_2024'; `$env:JWT_SECRET = 'jwt_secret_change_in_production_2024'; Write-Host '[BACKEND] Backend API starting...' -ForegroundColor Green; Write-Host 'API will be available at: http://localhost:8080' -ForegroundColor Cyan; Write-Host 'Swagger UI: http://localhost:8080/swagger-ui/' -ForegroundColor Cyan; cargo run"

Start-Process powershell -ArgumentList @("-NoExit", "-Command", $backendScript)

# Wait a bit for backend to start
Start-Sleep -Seconds 5

# Start Frontend (in new window)
Write-Host "[FRONTEND] Starting Frontend in new window..." -ForegroundColor Yellow
$frontendScript = "cd '$projectRoot\dashboard'; Write-Host '[FRONTEND] Frontend starting...' -ForegroundColor Green; Write-Host 'Frontend will be available at: http://localhost:3000' -ForegroundColor Cyan; npm run dev"

Start-Process powershell -ArgumentList @("-NoExit", "-Command", $frontendScript)

# Summary
Write-Host ""
Write-Host ("=" * 60) -ForegroundColor Cyan
Write-Host "[SUCCESS] Platform starting!" -ForegroundColor Green
Write-Host ("=" * 60) -ForegroundColor Cyan
Write-Host "`nAccess URLs:" -ForegroundColor Cyan
Write-Host "  Backend API:     http://localhost:8080" -ForegroundColor White
Write-Host "  Swagger UI:      http://localhost:8080/swagger-ui/" -ForegroundColor White
Write-Host "  Health Check:    http://localhost:8080/health" -ForegroundColor White
Write-Host "  Frontend:        http://localhost:3000" -ForegroundColor White
Write-Host "  Wizard:          http://localhost:3000/wizard" -ForegroundColor White
Write-Host "`nTips:" -ForegroundColor Magenta
Write-Host "  - Backend and Frontend are running in separate windows" -ForegroundColor Gray
Write-Host "  - Wait 30-60 seconds for everything to fully start" -ForegroundColor Gray
Write-Host "  - Check Docker containers: docker-compose ps" -ForegroundColor Gray
Write-Host "  - Stop everything: docker-compose down" -ForegroundColor Gray
Write-Host ""

