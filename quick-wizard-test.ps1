# Quick test to check if backend is running and test wizard endpoint
Write-Host "Quick Wizard Test" -ForegroundColor Cyan
Write-Host ""

# Check backend
Write-Host "Checking backend..." -ForegroundColor Yellow
try {
    $health = Invoke-WebRequest -Uri "http://localhost:8080/health" -TimeoutSec 3
    Write-Host "Backend is RUNNING" -ForegroundColor Green
} catch {
    Write-Host "Backend is NOT running" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please:" -ForegroundColor Yellow
    Write-Host "1. Open a new PowerShell window" -ForegroundColor White
    Write-Host "2. Run: cd C:\Users\Brano\Projects\veridion-nexus" -ForegroundColor Cyan
    Write-Host "3. Run: `$env:DATABASE_URL = 'postgresql://veridion:veridion_secure_pass_2024@localhost:5432/veridion_nexus'" -ForegroundColor Cyan
    Write-Host "4. Run: cargo run" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Wait for: 'Veridion Nexus API starting on port 8080'" -ForegroundColor Gray
    Write-Host "Then refresh the Wizard page in browser (F5)" -ForegroundColor Gray
    exit 1
}

# Test wizard endpoint
Write-Host "`nTesting wizard/recommend-modules endpoint..." -ForegroundColor Yellow
try {
    $body = '{"industry":"FINANCIAL_SERVICES","regulatory_requirements":["GDPR"],"ai_use_cases":["CREDIT_SCORING"]}'
    $response = Invoke-WebRequest -Uri "http://localhost:8080/api/v1/wizard/recommend-modules" `
        -Method POST `
        -ContentType "application/json" `
        -Body $body `
        -TimeoutSec 10
    
    Write-Host "Wizard endpoint is WORKING" -ForegroundColor Green
    Write-Host "Status: $($response.StatusCode)" -ForegroundColor Gray
    $data = $response.Content | ConvertFrom-Json
    Write-Host "Recommended modules: $($data.recommended_modules.Count)" -ForegroundColor Gray
    Write-Host ""
    Write-Host "If you see this, the Wizard should work in browser!" -ForegroundColor Cyan
    Write-Host "Refresh the Wizard page (F5) and try again." -ForegroundColor Yellow
} catch {
    Write-Host "Wizard endpoint FAILED" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $errorBody = $reader.ReadToEnd()
        Write-Host "Details: $errorBody" -ForegroundColor Gray
    }
}

