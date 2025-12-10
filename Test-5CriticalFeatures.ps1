# Test script for 5 Critical Features
# 1. Veridion TPRM Integration
# 2. Executive Assurance Reporting
# 3. AI Explainability & Observability
# 4. Configuration Drift Detection
# 5. Multi-Cloud Native Integrations

$ErrorActionPreference = "Stop"
$API_BASE = "http://127.0.0.1:8080/api/v1"

# Colors for output
function Write-Success { Write-Host $args -ForegroundColor Green }
function Write-Error { Write-Host $args -ForegroundColor Red }
function Write-Info { Write-Host $args -ForegroundColor Cyan }
function Write-Warning { Write-Host $args -ForegroundColor Yellow }

# Get auth token (assuming default admin user)
function Get-AuthToken {
    Write-Info "🔐 Authenticating..."
    $loginBody = @{
        username = "admin"
        password = "admin123"
    } | ConvertTo-Json

    try {
        $response = Invoke-RestMethod -Uri "$API_BASE/auth/login" `
            -Method POST `
            -Body $loginBody `
            -ContentType "application/json"
        
        return $response.token
    } catch {
        Write-Error "❌ Authentication failed: $_"
        return $null
    }
}

$token = Get-AuthToken
if (-not $token) {
    Write-Error "Cannot proceed without authentication token"
    exit 1
}

$headers = @{
    "Authorization" = "Bearer $token"
    "Content-Type" = "application/json"
}

Write-Success "✅ Authentication successful"
Write-Host ""

# ========== TEST 1: Veridion TPRM Integration ==========
Write-Info "=" * 60
Write-Info "TEST 1: Veridion TPRM Integration"
Write-Info "=" * 60

Write-Info "📊 Testing: GET /vendors/openai.com/risk-score"
try {
    $response = Invoke-RestMethod -Uri "$API_BASE/vendors/openai.com/risk-score" `
        -Method GET `
        -Headers $headers
    
    Write-Success "✅ Vendor risk score retrieved:"
    Write-Host "   Domain: $($response.vendor_domain)"
    Write-Host "   Risk Score: $($response.risk_score)"
    Write-Host "   Risk Level: $($response.risk_level)"
    Write-Host "   Compliance Status: $($response.compliance_status)"
} catch {
    Write-Error "❌ Failed: $_"
}

Write-Host ""

# ========== TEST 2: Executive Assurance Reporting ==========
Write-Info "=" * 60
Write-Info "TEST 2: Executive Assurance Reporting"
Write-Info "=" * 60

Write-Info "📊 Testing: GET /reports/executive-assurance"
try {
    $response = Invoke-RestMethod -Uri "$API_BASE/reports/executive-assurance" `
        -Method GET `
        -Headers $headers
    
    Write-Success "✅ Executive scorecard generated:"
    Write-Host "   Compliance Score: $($response.compliance_score)%"
    Write-Host "   Risk Level: $($response.risk_level)"
    Write-Host "   Liability Protection: $($response.liability_protection_status)"
    Write-Host "   NIS2 Readiness: $($response.nis2_readiness)%"
    Write-Host "   DORA Compliance: $($response.dora_compliance)"
    Write-Host "   Total Assets: $($response.total_assets)"
    Write-Host "   Critical Issues: $($response.critical_issues_count)"
} catch {
    Write-Error "❌ Failed: $_"
}

Write-Info "📊 Testing: GET /reports/compliance-kpis"
try {
    $response = Invoke-RestMethod -Uri "$API_BASE/reports/compliance-kpis" `
        -Method GET `
        -Headers $headers
    
    Write-Success "✅ Compliance KPIs retrieved: $($response.Count) KPIs"
} catch {
    Write-Error "❌ Failed: $_"
}

Write-Host ""

# ========== TEST 3: AI Explainability & Observability ==========
Write-Info "=" * 60
Write-Info "TEST 3: AI Explainability & Observability"
Write-Info "=" * 60

Write-Info "📊 Testing: GET /models/test-model/feature-importance"
try {
    $response = Invoke-RestMethod -Uri "$API_BASE/models/test-model/feature-importance" `
        -Method GET `
        -Headers $headers
    
    Write-Success "✅ Feature importance retrieved: $($response.Count) features"
} catch {
    Write-Error "❌ Failed: $_"
}

Write-Info "📊 Testing: GET /models/test-model/drift"
try {
    $response = Invoke-RestMethod -Uri "$API_BASE/models/test-model/drift" `
        -Method GET `
        -Headers $headers
    
    if ($response.status -eq "NO_DRIFT_DETECTED") {
        Write-Success "✅ No drift detected (expected for test model)"
    } else {
        Write-Success "✅ Drift detection result: $($response.drift_severity)"
    }
} catch {
    Write-Error "❌ Failed: $_"
}

Write-Host ""

# ========== TEST 4: Configuration Drift Detection ==========
Write-Info "=" * 60
Write-Info "TEST 4: Configuration Drift Detection"
Write-Info "=" * 60

Write-Info "📊 Testing: POST /configuration/baselines (create baseline)"
$baselineBody = @{
    baseline_name = "test-baseline-$(Get-Date -Format 'yyyyMMddHHmmss')"
    baseline_type = "POLICY"
    baseline_config = @{
        policy_type = "SOVEREIGN_LOCK"
        enabled = $true
    } | ConvertTo-Json -Compress
    is_golden_image = $false
    description = "Test baseline for drift detection"
} | ConvertTo-Json

try {
    $baselineResponse = Invoke-RestMethod -Uri "$API_BASE/configuration/baselines" `
        -Method POST `
        -Headers $headers `
        -Body $baselineBody
    
    $baselineId = $baselineResponse.id
    Write-Success "✅ Baseline created: $baselineId"
    
    Write-Info "📊 Testing: POST /configuration/baselines/$baselineId/detect-drift"
    $driftBody = @{
        current_config = @{
            policy_type = "SOVEREIGN_LOCK"
            enabled = $true
            modified_field = "test"
        } | ConvertTo-Json -Compress
    } | ConvertTo-Json
    
    $driftResponse = Invoke-RestMethod -Uri "$API_BASE/configuration/baselines/$baselineId/detect-drift" `
        -Method POST `
        -Headers $headers `
        -Body $driftBody
    
    Write-Success "✅ Drift detection completed: $($driftResponse.total_count) drift(s) found"
} catch {
    Write-Error "❌ Failed: $_"
}

Write-Host ""

# ========== TEST 5: Multi-Cloud Native Integrations ==========
Write-Info "=" * 60
Write-Info "TEST 5: Multi-Cloud Native Integrations"
Write-Info "=" * 60

Write-Info "📊 Testing: POST /cloud/providers (register AWS)"
$cloudProviderBody = @{
    provider = "AWS"
    account_id = "123456789012"
    region = "eu-west-1"
    credentials_encrypted = "encrypted-credentials-placeholder"
} | ConvertTo-Json

try {
    $providerResponse = Invoke-RestMethod -Uri "$API_BASE/cloud/providers" `
        -Method POST `
        -Headers $headers `
        -Body $cloudProviderBody
    
    Write-Success "✅ Cloud provider registered: $($providerResponse.provider)"
    
    Write-Info "📊 Testing: POST /cloud/providers/AWS/sync?account_id=123456789012"
    $syncResponse = Invoke-RestMethod -Uri "$API_BASE/cloud/providers/AWS/sync?account_id=123456789012" `
        -Method POST `
        -Headers $headers
    
    Write-Success "✅ Cloud sync started: $($syncResponse.sync_id)"
    
    Write-Info "📊 Testing: GET /cloud/providers/AWS/compliance"
    Start-Sleep -Seconds 2  # Wait a bit for sync
    $complianceResponse = Invoke-RestMethod -Uri "$API_BASE/cloud/providers/AWS/compliance" `
        -Method GET `
        -Headers $headers
    
    Write-Success "✅ Compliance summary:"
    Write-Host "   Total Resources: $($complianceResponse.total_resources)"
    Write-Host "   Compliance: $($complianceResponse.compliance_percentage)%"
} catch {
    Write-Error "❌ Failed: $_"
}

Write-Host ""
Write-Success ("=" * 60)
Write-Success "✅ All 5 Critical Features Tests Completed!"
Write-Success ("=" * 60)

