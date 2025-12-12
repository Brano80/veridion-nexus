# Phase 1 MVP Success Criteria Verification Script
# Tests all critical metrics for Phase 1 launch readiness

$ErrorActionPreference = "Continue"
$API_URL = "http://localhost:8080/api/v1"

# Test Results
$script:TEST_RESULTS = @()
$script:ALL_PASSED = $true

# Colors
function Write-Success { param($msg) Write-Host "[PASS] $msg" -ForegroundColor Green }
function Write-Failure { param($msg) Write-Host "[FAIL] $msg" -ForegroundColor Red }
function Write-Info { param($msg) Write-Host "[INFO] $msg" -ForegroundColor Cyan }
function Write-Warning { param($msg) Write-Host "[WARN] $msg" -ForegroundColor Yellow }

# Test result tracking
function Add-TestResult {
    param(
        [string]$Category,
        [string]$TestName,
        [bool]$Passed,
        [string]$Details = "",
        [string]$Value = ""
    )
    
    $script:TEST_RESULTS += [PSCustomObject]@{
        Category = $Category
        TestName = $TestName
        Passed = $Passed
        Details = $Details
        Value = $Value
    }
    
    if (-not $Passed) {
        $script:ALL_PASSED = $false
    }
}

# Authentication
function Get-AuthToken {
    try {
        $loginBody = @{
            username = "admin"
            password = "admin"
        } | ConvertTo-Json
        
        $response = Invoke-RestMethod -Uri "$API_URL/auth/login" `
            -Method POST `
            -Body $loginBody `
            -ContentType "application/json" `
            -ErrorAction Stop
        
        return $response.token
    } catch {
        Write-Warning "Failed to authenticate. Trying without auth..."
        return $null
    }
}

function Get-AuthHeaders {
    $token = Get-AuthToken
    if ($token) {
        return @{
            "Authorization" = "Bearer $token"
            "Content-Type" = "application/json"
        }
    }
    return @{
        "Content-Type" = "application/json"
    }
}

# ============================================================================
# TEST 1: API Responsiveness (Time to first policy test)
# ============================================================================
function Test-APIResponsiveness {
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "TEST 1: API Responsiveness (Time to first policy test)" -ForegroundColor Yellow
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    
    try {
        $headers = Get-AuthHeaders
        
        # Create a simple policy for testing
        $policyBody = @{
            policy_type = "LOCATION"
            policy_config = @{
                allowed_regions = @("EU", "US")
                block_non_compliant = $true
            }
            notes = "MVP Test Policy"
        } | ConvertTo-Json
        
        # Measure time to create policy
        $createStart = Get-Date
        $createResponse = Invoke-RestMethod -Uri "$API_URL/policies" `
            -Method POST `
            -Body $policyBody `
            -Headers $headers `
            -ErrorAction Stop
        $createTime = ((Get-Date) - $createStart).TotalMilliseconds
        
        Write-Info "Policy created in $([math]::Round($createTime, 2))ms"
        
        # Measure time to simulate policy
        $simulateStart = Get-Date
        $simulateBody = @{
            policy_type = "LOCATION"
            policy_config = @{
                allowed_regions = @("EU", "US")
                block_non_compliant = $true
            }
            time_range_days = 7
        } | ConvertTo-Json
        
        $simulateResponse = Invoke-RestMethod -Uri "$API_URL/policies/simulate" `
            -Method POST `
            -Body $simulateBody `
            -Headers $headers `
            -ErrorAction Stop
        $simulateTime = ((Get-Date) - $simulateStart).TotalMilliseconds
        
        Write-Info "Policy simulation completed in $([math]::Round($simulateTime, 2))ms"
        
        $totalTime = $createTime + $simulateTime
        $threshold = 5000 # 5 seconds threshold
        
        $totalTimeRounded = [math]::Round($totalTime, 2)
        $thresholdMs = "$threshold ms"
        
        if ($totalTime -lt $threshold) {
            $timeStr = "$totalTimeRounded" + "ms"
            $msg = 'API Responsiveness: PASS (' + $timeStr + ' less than ' + $thresholdMs + ')'
            Write-Success $msg
            $details = "Total time: $timeStr"
            $value = $timeStr
            Add-TestResult -Category "API Responsiveness" -TestName "Time to first policy test" `
                -Passed $true -Details $details -Value $value
        } else {
            $timeStr = "$totalTimeRounded" + "ms"
            $msg = 'API Responsiveness: FAIL (' + $timeStr + ' greater than or equal to ' + $thresholdMs + ')'
            Write-Failure $msg
            $details = "Total time: $timeStr (threshold: $thresholdMs)"
            $value = $timeStr
            Add-TestResult -Category "API Responsiveness" -TestName "Time to first policy test" `
                -Passed $false -Details $details -Value $value
        }
        
        return $createResponse.id
    } catch {
        Write-Failure "API Responsiveness: FAIL - $($_.Exception.Message)"
        Add-TestResult -Category "API Responsiveness" -TestName "Time to first policy test" `
            -Passed $false -Details $_.Exception.Message
        return $null
    }
}

# ============================================================================
# TEST 2: Rollback Speed (< 30 seconds)
# ============================================================================
function Test-RollbackSpeed {
    param([string]$PolicyId)
    
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "TEST 2: Rollback Speed (< 30 seconds)" -ForegroundColor Yellow
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    
    if (-not $PolicyId) {
        Write-Warning "Skipping rollback test - no policy ID available"
            Add-TestResult -Category "Rollback Speed" -TestName "Policy rollback under 30s" `
            -Passed $false -Details "No policy ID available"
        return
    }
    
    try {
        $headers = Get-AuthHeaders
        
        # Measure rollback time
        $rollbackStart = Get-Date
        
        $rollbackBody = @{
            notes = "MVP Test Rollback"
        } | ConvertTo-Json
        
        $rollbackResponse = Invoke-RestMethod -Uri "$API_URL/policies/$PolicyId/rollback" `
            -Method POST `
            -Body $rollbackBody `
            -Headers $headers `
            -ErrorAction Stop
        
        $rollbackTime = ((Get-Date) - $rollbackStart).TotalSeconds
        $rollbackTimeRounded = [math]::Round($rollbackTime, 2)
        $timeStr = "$rollbackTimeRounded" + "s"
        Write-Info "Rollback completed in $timeStr"
        
        $threshold = 30 # 30 seconds
        
        if ($rollbackTime -lt $threshold) {
            $thresholdStr = "$threshold" + "s"
            $msg = 'Rollback Speed: PASS (' + $timeStr + ' less than ' + $thresholdStr + ')'
            Write-Success $msg
            $details = "Rollback time: $timeStr"
            $value = $timeStr
            Add-TestResult -Category "Rollback Speed" -TestName "Policy rollback under 30s" `
                -Passed $true -Details $details -Value $value
        } else {
            $thresholdStr = "$threshold" + "s"
            $msg = 'Rollback Speed: FAIL (' + $timeStr + ' greater than or equal to ' + $thresholdStr + ')'
            Write-Failure $msg
            $details = "Rollback time: $timeStr (threshold: $thresholdStr)"
            $value = $timeStr
            Add-TestResult -Category "Rollback Speed" -TestName "Policy rollback under 30s" `
                -Passed $false -Details $details -Value $value
        }
    } catch {
        Write-Failure "Rollback Speed: FAIL - $($_.Exception.Message)"
            Add-TestResult -Category "Rollback Speed" -TestName "Policy rollback under 30s" `
            -Passed $false -Details $_.Exception.Message
    }
}

# ============================================================================
# TEST 3: Shadow Mode (confidence_score in analytics)
# ============================================================================
function Test-ShadowModeConfidence {
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "TEST 3: Shadow Mode (confidence_score in analytics)" -ForegroundColor Yellow
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    
    try {
        $headers = Get-AuthHeaders
        
        $response = Invoke-RestMethod -Uri "$API_URL/analytics/shadow-mode?days=7" `
            -Method GET `
            -Headers $headers `
            -ErrorAction Stop
        
        if ($response.confidence_score -ne $null) {
            $confidence = $response.confidence_score
            Write-Info "Confidence score: $confidence%"
            Write-Info "Total logs: $($response.total_logs)"
            
            Write-Success "Shadow Mode: PASS (confidence_score present: $confidence%)"
            Add-TestResult -Category "Shadow Mode" -TestName "confidence_score in analytics" `
                -Passed $true -Details "Confidence score: $confidence%" `
                -Value "$confidence%"
        } else {
            Write-Failure "Shadow Mode: FAIL (confidence_score missing)"
            Add-TestResult -Category "Shadow Mode" -TestName "confidence_score in analytics" `
                -Passed $false -Details "confidence_score field missing from response"
        }
    } catch {
        Write-Failure "Shadow Mode: FAIL - $($_.Exception.Message)"
        Add-TestResult -Category "Shadow Mode" -TestName "confidence_score in analytics" `
            -Passed $false -Details $_.Exception.Message
    }
}

# ============================================================================
# TEST 4: Compliance Scores (GDPR/AI Act scores > 0)
# ============================================================================
function Test-ComplianceScores {
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "TEST 4: Compliance Scores (GDPR/AI Act scores > 0)" -ForegroundColor Yellow
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    
    try {
        $headers = Get-AuthHeaders
        
        # Try compliance-overview endpoint first
        try {
            $response = Invoke-RestMethod -Uri "$API_URL/reports/compliance-overview" `
                -Method GET `
                -Headers $headers `
                -ErrorAction Stop
            
            $gdprScore = $response.gdpr_score
            $aiActScore = $response.eu_ai_act_score
            
            Write-Info "GDPR Score: $gdprScore%"
            Write-Info "EU AI Act Score: $aiActScore%"
            
            if ($gdprScore -gt 0 -and $aiActScore -gt 0) {
                Write-Success "Compliance Scores: PASS (GDPR: $gdprScore%, AI Act: $aiActScore%)"
                Add-TestResult -Category "Compliance Scores" -TestName "GDPR/AI Act scores greater than 0" `
                    -Passed $true -Details "GDPR: $gdprScore%, AI Act: $aiActScore%" `
                    -Value "GDPR: $gdprScore%, AI Act: $aiActScore%"
            } else {
                Write-Failure "Compliance Scores: FAIL (GDPR: $gdprScore%, AI Act: $aiActScore%)"
                Add-TestResult -Category "Compliance Scores" -TestName "GDPR/AI Act scores greater than 0" `
                    -Passed $false -Details "GDPR: $gdprScore%, AI Act: $aiActScore%" `
                    -Value "GDPR: $gdprScore%, AI Act: $aiActScore%"
            }
        } catch {
            # Fallback to monthly-summary endpoint
            Write-Info "Trying monthly-summary endpoint as fallback..."
            $response = Invoke-RestMethod -Uri "$API_URL/reports/monthly-summary" `
                -Method GET `
                -Headers $headers `
                -ErrorAction Stop
            
            $gdprScore = $response.gdpr_score
            $aiActScore = $response.eu_ai_act_score
            
            Write-Info "GDPR Score: $gdprScore%"
            Write-Info "EU AI Act Score: $aiActScore%"
            
            if ($gdprScore -gt 0 -and $aiActScore -gt 0) {
                Write-Success "Compliance Scores: PASS (GDPR: $gdprScore%, AI Act: $aiActScore%)"
                Add-TestResult -Category "Compliance Scores" -TestName "GDPR/AI Act scores greater than 0" `
                    -Passed $true -Details "GDPR: $gdprScore%, AI Act: $aiActScore%" `
                    -Value "GDPR: $gdprScore%, AI Act: $aiActScore%"
            } else {
                Write-Failure "Compliance Scores: FAIL (GDPR: $gdprScore%, AI Act: $aiActScore%)"
                Add-TestResult -Category "Compliance Scores" -TestName "GDPR/AI Act scores greater than 0" `
                    -Passed $false -Details "GDPR: $gdprScore%, AI Act: $aiActScore%" `
                    -Value "GDPR: $gdprScore%, AI Act: $aiActScore%"
            }
        }
    } catch {
        Write-Failure "Compliance Scores: FAIL - $($_.Exception.Message)"
        Add-TestResult -Category "Compliance Scores" -TestName "GDPR/AI Act scores > 0" `
            -Passed $false -Details $_.Exception.Message
    }
}

# ============================================================================
# MAIN EXECUTION
# ============================================================================
Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║     Phase 1 MVP Success Criteria Verification                        ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Check if API is running
Write-Info "Checking API health..."
try {
    $healthResponse = Invoke-WebRequest -Uri "http://localhost:8080/health" -TimeoutSec 5 -UseBasicParsing
    if ($healthResponse.StatusCode -eq 200) {
        Write-Success "API is running"
    } else {
        Write-Failure "API health check failed"
        exit 1
    }
} catch {
    Write-Failure "API is not running. Please start the backend first."
    exit 1
}

# Run tests
$policyId = Test-APIResponsiveness
Test-RollbackSpeed -PolicyId $policyId
Test-ShadowModeConfidence
Test-ComplianceScores

# ============================================================================
# SUMMARY
# ============================================================================
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "📊 TEST SUMMARY" -ForegroundColor Yellow
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host ""

$passed = ($script:TEST_RESULTS | Where-Object { $_.Passed -eq $true }).Count
$failed = ($script:TEST_RESULTS | Where-Object { $_.Passed -eq $false }).Count
$total = $script:TEST_RESULTS.Count

Write-Host "Total Tests: $total" -ForegroundColor White
Write-Host "✅ Passed: $passed" -ForegroundColor Green
Write-Host "❌ Failed: $failed" -ForegroundColor $(if ($failed -gt 0) { "Red" } else { "Green" })
Write-Host ""

# Detailed results
Write-Host "Detailed Results:" -ForegroundColor Cyan
Write-Host ""
foreach ($result in $script:TEST_RESULTS) {
    $status = if ($result.Passed) { "✅" } else { "❌" }
    Write-Host "$status [$($result.Category)] $($result.TestName)" -ForegroundColor $(if ($result.Passed) { "Green" } else { "Red" })
    if ($result.Details) {
        Write-Host "   Details: $($result.Details)" -ForegroundColor Gray
    }
    if ($result.Value) {
        Write-Host "   Value: $($result.Value)" -ForegroundColor Gray
    }
}

Write-Host ""

if ($script:ALL_PASSED) {
    Write-Host "ALL TESTS PASSED! System is ready for Phase 1 launch!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "WARNING: Some tests failed. Please review the results above." -ForegroundColor Yellow
    exit 1
}

