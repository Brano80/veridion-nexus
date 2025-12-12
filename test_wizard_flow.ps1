# Test Wizard Flow Script
# Tests the complete wizard flow from start to finish

$ErrorActionPreference = "Continue"
$API_BASE = "http://127.0.0.1:8080/api/v1"

Write-Host "`nTESTING WIZARD FLOW" -ForegroundColor Cyan
Write-Host ("=" * 70) -ForegroundColor Gray

# Step 1: Health Check
Write-Host "`n[1/6] Testing Backend Health..." -ForegroundColor Yellow
try {
    $health = Invoke-WebRequest -Uri "http://127.0.0.1:8080/health" -TimeoutSec 5 -UseBasicParsing
    Write-Host "   [OK] Backend is running (Status: $($health.StatusCode))" -ForegroundColor Green
} catch {
    Write-Host "   [FAIL] Backend is NOT running!" -ForegroundColor Red
    Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Yellow
    exit 1
}

# Step 2: Test recommend-modules
Write-Host "`n[2/6] Testing recommend-modules API..." -ForegroundColor Yellow
$recRequest = @{
    industry = "FINANCIAL_SERVICES"
    regulatory_requirements = @("GDPR", "DORA")
    ai_use_cases = @("CREDIT_SCORING")
} | ConvertTo-Json

try {
    $recResponse = Invoke-WebRequest -Uri "$API_BASE/wizard/recommend-modules" `
        -Method POST -Body $recRequest -ContentType "application/json" -TimeoutSec 10 -UseBasicParsing
    $recData = $recResponse.Content | ConvertFrom-Json
    Write-Host "   [OK] recommend-modules: OK" -ForegroundColor Green
    Write-Host "   Found $($recData.recommended_modules.Count) modules" -ForegroundColor White
    Write-Host "   Required: $($recData.required_count), Recommended: $($recData.recommended_count), Optional: $($recData.optional_count)" -ForegroundColor Gray
} catch {
    Write-Host "   [FAIL] recommend-modules: FAILED" -ForegroundColor Red
    Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Yellow
}

# Step 3: Test calculate-price
Write-Host "`n[3/6] Testing calculate-price API..." -ForegroundColor Yellow
$priceRequest = @{
    selected_modules = @("gdpr_article_12", "module_human_oversight")
    num_systems = 2
} | ConvertTo-Json

try {
    $priceResponse = Invoke-WebRequest -Uri "$API_BASE/wizard/calculate-price" `
        -Method POST -Body $priceRequest -ContentType "application/json" -TimeoutSec 10 -UseBasicParsing
    $priceData = $priceResponse.Content | ConvertFrom-Json
    Write-Host "   [OK] calculate-price: OK" -ForegroundColor Green
    Write-Host "   Base: €$($priceData.base_price), Per System: €$($priceData.per_system_price)" -ForegroundColor White
    Write-Host "   Total Monthly: €$([math]::Round($priceData.total_monthly, 2))" -ForegroundColor White
    Write-Host "   Total Annual: €$([math]::Round($priceData.total_annual, 2))" -ForegroundColor White
} catch {
    Write-Host "   [FAIL] calculate-price: FAILED" -ForegroundColor Red
    Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Yellow
}

# Step 4: Test company-profile creation
Write-Host "`n[4/6] Testing company-profile API..." -ForegroundColor Yellow
$profileRequest = @{
    company_name = "Test Company $(Get-Date -Format 'HHmmss')"
    industry = "FINANCIAL_SERVICES"
    company_size = "SME"
    country = "SK"
    regulatory_requirements = @("GDPR", "DORA")
    ai_use_cases = @("CREDIT_SCORING", "FRAUD_DETECTION")
    deployment_preference = "SDK"
    estimated_ai_systems = 2
} | ConvertTo-Json

$companyId = $null
try {
    $profileResponse = Invoke-WebRequest -Uri "$API_BASE/wizard/company-profile" `
        -Method POST -Body $profileRequest -ContentType "application/json" -TimeoutSec 10 -UseBasicParsing
    $profileData = $profileResponse.Content | ConvertFrom-Json
    $companyId = $profileData.id
    Write-Host "   [OK] company-profile: OK" -ForegroundColor Green
    Write-Host "   Company ID: $companyId" -ForegroundColor White
    Write-Host "   Company Name: $($profileData.company_name)" -ForegroundColor White
} catch {
    Write-Host "   [FAIL] company-profile: FAILED" -ForegroundColor Red
    Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Yellow
}

# Step 5: Test start-trial
if ($companyId) {
    Write-Host "`n[5/6] Testing start-trial API..." -ForegroundColor Yellow
    $trialRequest = @{
        company_id = $companyId
        selected_modules = @("gdpr_article_12")
        estimated_ai_systems = 2
    } | ConvertTo-Json

    try {
        $trialResponse = Invoke-WebRequest -Uri "$API_BASE/wizard/start-trial" `
            -Method POST -Body $trialRequest -ContentType "application/json" -TimeoutSec 10 -UseBasicParsing
        $trialData = $trialResponse.Content | ConvertFrom-Json
        Write-Host "   [OK] start-trial: OK" -ForegroundColor Green
        Write-Host "   Subscription ID: $($trialData.id)" -ForegroundColor White
        Write-Host "   Status: $($trialData.status)" -ForegroundColor White
        if ($trialData.days_remaining) {
            Write-Host "   Days Remaining: $($trialData.days_remaining)" -ForegroundColor White
        }
    } catch {
        Write-Host "   [FAIL] start-trial: FAILED" -ForegroundColor Red
        Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Yellow
    }
} else {
    Write-Host "`n[5/6] Skipping start-trial (no company ID)" -ForegroundColor Yellow
}

# Step 6: Verify module activation
if ($companyId) {
    Write-Host "`n[6/6] Verifying module activation..." -ForegroundColor Yellow
    $modules = docker-compose exec -T postgres psql -U veridion -d veridion_nexus `
        -c "SELECT m.name, cmc.enabled FROM company_module_configs cmc JOIN modules m ON cmc.module_id = m.id WHERE cmc.company_id = '$companyId'::uuid;" 2>&1
    
    if ($modules -match "gdpr_article_12.*true") {
        Write-Host "   [OK] Module gdpr_article_12 is activated!" -ForegroundColor Green
    } else {
        Write-Host "   ⚠️  Module activation check:" -ForegroundColor Yellow
        $modules | Select-String -Pattern "name|enabled|gdpr" | ForEach-Object { Write-Host "      $_" -ForegroundColor Gray }
    }
} else {
    Write-Host "`n[6/6] Skipping module verification (no company ID)" -ForegroundColor Yellow
}

# Summary
Write-Host "`nTEST SUMMARY" -ForegroundColor Cyan
Write-Host ("=" * 70) -ForegroundColor Gray
Write-Host "`nAll API endpoints tested!" -ForegroundColor Green
Write-Host "`nNext: Test in browser at http://localhost:3000/wizard" -ForegroundColor Cyan

