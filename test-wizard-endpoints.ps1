# Test all Wizard API endpoints
Write-Host "Testing Wizard API Endpoints..." -ForegroundColor Cyan
Write-Host ""

$API_BASE = "http://localhost:8080/api/v1"

# Test 1: Health check
Write-Host "[1/7] Testing health endpoint..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "http://localhost:8080/health" -TimeoutSec 5
    Write-Host "  ✅ Health check: $($response.StatusCode)" -ForegroundColor Green
} catch {
    Write-Host "  ❌ Health check failed: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "`nBackend is not running. Please start it first:" -ForegroundColor Yellow
    Write-Host "  .\start-backend.ps1" -ForegroundColor Cyan
    exit 1
}

# Test 2: Recommend modules
Write-Host "`n[2/7] Testing recommend-modules endpoint..." -ForegroundColor Yellow
try {
    $body = @{
        industry = "FINANCIAL_SERVICES"
        regulatory_requirements = @("GDPR", "DORA")
        ai_use_cases = @("CREDIT_SCORING")
    } | ConvertTo-Json
    
    $response = Invoke-WebRequest -Uri "$API_BASE/wizard/recommend-modules" `
        -Method POST `
        -ContentType "application/json" `
        -Body $body `
        -TimeoutSec 10
    
    Write-Host "  ✅ Recommend modules: $($response.StatusCode)" -ForegroundColor Green
    $data = $response.Content | ConvertFrom-Json
    Write-Host "  📦 Found $($data.recommended_modules.Count) recommended modules" -ForegroundColor Gray
} catch {
    Write-Host "  ❌ Recommend modules failed: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $errorBody = $reader.ReadToEnd()
        Write-Host "  Error details: $errorBody" -ForegroundColor Gray
    }
}

# Test 3: Calculate price
Write-Host "`n[3/7] Testing calculate-price endpoint..." -ForegroundColor Yellow
try {
    $body = @{
        selected_modules = @("module_risk_assessment", "module_human_oversight")
        num_systems = 2
    } | ConvertTo-Json
    
    $response = Invoke-WebRequest -Uri "$API_BASE/wizard/calculate-price" `
        -Method POST `
        -ContentType "application/json" `
        -Body $body `
        -TimeoutSec 10
    
    Write-Host "  ✅ Calculate price: $($response.StatusCode)" -ForegroundColor Green
    $data = $response.Content | ConvertFrom-Json
    Write-Host "  💰 Total monthly: €$($data.total_monthly)" -ForegroundColor Gray
} catch {
    Write-Host "  ❌ Calculate price failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 4: Create company profile
Write-Host "`n[4/7] Testing create-company-profile endpoint..." -ForegroundColor Yellow
$companyId = $null
try {
    $body = @{
        company_name = "Test Company"
        industry = "FINANCIAL_SERVICES"
        company_size = "SME"
        country = "Slovakia"
        regulatory_requirements = @("GDPR", "DORA")
        ai_use_cases = @("CREDIT_SCORING")
        deployment_preference = "SDK"
        estimated_ai_systems = 2
    } | ConvertTo-Json
    
    $response = Invoke-WebRequest -Uri "$API_BASE/wizard/company-profile" `
        -Method POST `
        -ContentType "application/json" `
        -Body $body `
        -TimeoutSec 10
    
    Write-Host "  ✅ Create company profile: $($response.StatusCode)" -ForegroundColor Green
    $data = $response.Content | ConvertFrom-Json
    $companyId = $data.id
    Write-Host "  🏢 Company ID: $companyId" -ForegroundColor Gray
} catch {
    Write-Host "  ❌ Create company profile failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 5: Start trial
if ($companyId) {
    Write-Host "`n[5/7] Testing start-trial endpoint..." -ForegroundColor Yellow
    try {
        $body = @{
            company_id = $companyId
            selected_modules = @("module_risk_assessment", "module_human_oversight")
            estimated_ai_systems = 2
        } | ConvertTo-Json
        
        $response = Invoke-WebRequest -Uri "$API_BASE/wizard/start-trial" `
            -Method POST `
            -ContentType "application/json" `
            -Body $body `
            -TimeoutSec 10
        
        Write-Host "  ✅ Start trial: $($response.StatusCode)" -ForegroundColor Green
        $data = $response.Content | ConvertFrom-Json
        Write-Host "  🎁 Trial status: $($data.status)" -ForegroundColor Gray
        Write-Host "  📅 Days remaining: $($data.days_remaining)" -ForegroundColor Gray
    } catch {
        Write-Host "  ❌ Start trial failed: $($_.Exception.Message)" -ForegroundColor Red
    }
} else {
    Write-Host "`n[5/7] Skipping start-trial (no company ID)" -ForegroundColor Gray
}

# Test 6: Get subscription
if ($companyId) {
    Write-Host "`n[6/7] Testing get-subscription endpoint..." -ForegroundColor Yellow
    try {
        $response = Invoke-WebRequest -Uri "$API_BASE/wizard/subscription/$companyId" `
            -Method GET `
            -TimeoutSec 10
        
        Write-Host "  ✅ Get subscription: $($response.StatusCode)" -ForegroundColor Green
        $data = $response.Content | ConvertFrom-Json
        Write-Host "  📋 Subscription type: $($data.subscription_type)" -ForegroundColor Gray
    } catch {
        Write-Host "  ❌ Get subscription failed: $($_.Exception.Message)" -ForegroundColor Red
    }
} else {
    Write-Host "`n[6/7] Skipping get-subscription (no company ID)" -ForegroundColor Gray
}

# Test 7: Get company profile
if ($companyId) {
    Write-Host "`n[7/7] Testing get-company-profile endpoint..." -ForegroundColor Yellow
    try {
        $response = Invoke-WebRequest -Uri "$API_BASE/wizard/company-profile/$companyId" `
            -Method GET `
            -TimeoutSec 10
        
        Write-Host "  ✅ Get company profile: $($response.StatusCode)" -ForegroundColor Green
        $data = $response.Content | ConvertFrom-Json
        Write-Host "  🏢 Company: $($data.company_name)" -ForegroundColor Gray
    } catch {
        Write-Host "  ❌ Get company profile failed: $($_.Exception.Message)" -ForegroundColor Red
    }
} else {
    Write-Host "`n[7/7] Skipping get-company-profile (no company ID)" -ForegroundColor Gray
}

Write-Host "`n✅ Wizard API testing complete!" -ForegroundColor Green
Write-Host "`nIf all tests passed, the Wizard should work in the browser." -ForegroundColor Cyan

