# Quick script to check if backend is running
Write-Host "Checking backend status..." -ForegroundColor Yellow

$maxAttempts = 12
$attempt = 0
$backendReady = $false

while ($attempt -lt $maxAttempts -and -not $backendReady) {
    $attempt++
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:8080/health" -TimeoutSec 2 -ErrorAction Stop
        $backendReady = $true
        Write-Host "`n✅ Backend is RUNNING!" -ForegroundColor Green
        Write-Host "   Status: $($response.StatusCode)" -ForegroundColor White
        Write-Host "   API: http://localhost:8080" -ForegroundColor Cyan
        Write-Host "   Swagger: http://localhost:8080/swagger-ui/" -ForegroundColor Cyan
        break
    } catch {
        Write-Host "." -NoNewline -ForegroundColor Gray
        Start-Sleep -Seconds 5
    }
}

if (-not $backendReady) {
    Write-Host "`n❌ Backend is NOT running yet" -ForegroundColor Red
    Write-Host "`nPlease check:" -ForegroundColor Yellow
    Write-Host "  1. Is the backend window open?" -ForegroundColor White
    Write-Host "  2. Is it still compiling? (first run takes 1-2 minutes)" -ForegroundColor White
    Write-Host "  3. Are there any error messages in the backend window?" -ForegroundColor White
    Write-Host "`nTo start backend manually:" -ForegroundColor Cyan
    Write-Host "  `$env:DATABASE_URL = 'postgresql://veridion:veridion_secure_pass_2024@localhost:5432/veridion_nexus'" -ForegroundColor Yellow
    Write-Host "  cargo run" -ForegroundColor Yellow
}

