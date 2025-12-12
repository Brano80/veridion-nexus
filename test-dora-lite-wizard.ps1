# Test DORA Lite Wizard Integration
# Tests wizard flow with DORA Lite auto-enable

$ErrorActionPreference = "Continue"
$API_URL = "http://localhost:8080/api/v1"

Write-Host "🧪 Testing DORA Lite Wizard Integration" -ForegroundColor Cyan
Write-Host ""

# Test 1: Create company profile with FINANCIAL_SERVICES industry
Write-Host "Test 1: Create company profile (FINANCIAL_SERVICES)" -ForegroundColor Yellow
$profileBody = @{
    company_name = "Test Fintech Startup"
    industry = "FINANCIAL_SERVICES"
    company_size = "SMALL"
    country = "SK"
    regulatory_requirements = @("DORA", "GDPR")
    ai_use_cases = @("CREDIT_SCORING")
    deployment_preference = "CLOUD"
    estimated_ai_systems = 2
} | ConvertTo-Json

try {
    $profileResponse = Invoke-RestMethod -Uri "$API_URL/wizard/company-profile" `
        -Method POST `
        -Body $profileBody `
        -ContentType "application/json" `
        -ErrorAction Stop
    
    Write-Host "✅ Company profile created: $($profileResponse.id)" -ForegroundColor Green
    $companyId = $profileResponse.id
    
    # Test 2: Get module recommendations
    Write-Host ""
    Write-Host "Test 2: Get module recommendations" -ForegroundColor Yellow
    $recommendBody = @{
        industry = "FINANCIAL_SERVICES"
        regulatory_requirements = @("DORA", "GDPR")
        ai_use_cases = @("CREDIT_SCORING")
    } | ConvertTo-Json
    
    $recommendResponse = Invoke-RestMethod -Uri "$API_URL/wizard/recommend-modules" `
        -Method POST `
        -Body $recommendBody `
        -ContentType "application/json" `
        -ErrorAction Stop
    
    $doraLiteRecommended = $recommendResponse.recommended_modules | Where-Object { $_.module_name -eq "module_dora_lite" }
    if ($doraLiteRecommended) {
        Write-Host "✅ DORA Lite is recommended: $($doraLiteRecommended.recommendation_reason)" -ForegroundColor Green
    } else {
        Write-Host "⚠️  DORA Lite not found in recommendations" -ForegroundColor Yellow
    }
    
    # Test 3: Check if DORA Lite was auto-enabled
    Write-Host ""
    Write-Host "Test 3: Check if DORA Lite was auto-enabled" -ForegroundColor Yellow
    Start-Sleep -Seconds 2
    
    # Check module status via modules API
    $modulesResponse = Invoke-RestMethod -Uri "$API_URL/modules" `
        -Method GET `
        -ErrorAction Stop
    
    $doraLiteModule = $modulesResponse | Where-Object { $_.name -eq "module_dora_lite" }
    if ($doraLiteModule) {
        Write-Host "✅ DORA Lite module found in modules list" -ForegroundColor Green
    } else {
        Write-Host "⚠️  DORA Lite module not found in modules list" -ForegroundColor Yellow
    }
    
    # Test 4: Test DORA Lite API endpoints
    Write-Host ""
    Write-Host "Test 4: Test DORA Lite API endpoints" -ForegroundColor Yellow
    
    # Get compliance status (should work even without auth for testing)
    try {
        $complianceResponse = Invoke-RestMethod -Uri "$API_URL/dora-lite/compliance-status" `
            -Method GET `
            -ErrorAction Stop
        Write-Host "✅ DORA Lite compliance status endpoint works" -ForegroundColor Green
        Write-Host "   Compliance Score: $($complianceResponse.compliance_score)%" -ForegroundColor Cyan
        Write-Host "   Article 9 (Vendors): $($complianceResponse.article9_compliant)" -ForegroundColor Cyan
        Write-Host "   Article 10 (Incidents): $($complianceResponse.article10_compliant)" -ForegroundColor Cyan
        Write-Host "   Article 11 (SLA): $($complianceResponse.article11_compliant)" -ForegroundColor Cyan
    } catch {
        Write-Host "⚠️  DORA Lite compliance status endpoint: $($_.Exception.Message)" -ForegroundColor Yellow
    }
    
    Write-Host ""
    Write-Host "✅ Wizard integration test completed!" -ForegroundColor Green
    
} catch {
    Write-Host "❌ Error: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $responseBody = $reader.ReadToEnd()
        Write-Host "Response: $responseBody" -ForegroundColor Red
    }
}

