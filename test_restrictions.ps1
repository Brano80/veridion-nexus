# Test Script for GDPR Article 18 - Processing Restrictions
# Tests all restriction endpoints and integration with log_action

$API_BASE = "http://localhost:8080/api/v1"
$JWT_TOKEN = $env:VERIDION_JWT_TOKEN

if (-not $JWT_TOKEN) {
    Write-Host "❌ VERIDION_JWT_TOKEN not set. Please login first:" -ForegroundColor Red
    Write-Host "   Invoke-RestMethod -Uri '$API_BASE/auth/login' -Method POST -ContentType 'application/json' -Body (@{username='testuser'; password='test123'} | ConvertTo-Json)" -ForegroundColor Yellow
    exit 1
}

Write-Host "`n═══════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "🧪 TESTING GDPR ARTICLE 18 - PROCESSING RESTRICTIONS" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════════════`n" -ForegroundColor Green

$headers = @{
    "Authorization" = "Bearer $JWT_TOKEN"
    "Content-Type" = "application/json"
}

$testUserId = "test-user-restriction-$(Get-Date -Format 'yyyyMMdd-HHmmss')"

# Test 1: Create FULL restriction
Write-Host "1️⃣  Testing: Create FULL processing restriction" -ForegroundColor Cyan
try {
    $restrictBody = @{
        user_id = $testUserId
        restriction_type = "FULL"
        reason = "User requested full processing restriction (GDPR Article 18)"
    } | ConvertTo-Json

    $response = Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/restrict" `
        -Method POST -Headers $headers -Body $restrictBody

    Write-Host "   ✅ FULL restriction created: $($response.restriction_id)" -ForegroundColor Green
    $restrictionId = $response.restriction_id
} catch {
    Write-Host "   ❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.ErrorDetails.Message) {
        Write-Host "   Response: $($_.ErrorDetails.Message)" -ForegroundColor Yellow
    }
    exit 1
}

# Test 2: Try to log action with restricted user (should be blocked)
Write-Host "`n2️⃣  Testing: log_action with FULL restriction (should be BLOCKED)" -ForegroundColor Cyan
try {
    $logBody = @{
        agent_id = "test-agent-restriction"
        action = "credit_scoring"
        payload = '{"test": "data"}'
        user_id = $testUserId
        target_region = "EU"
    } | ConvertTo-Json

    try {
        $logResponse = Invoke-RestMethod -Uri "$API_BASE/log_action" `
            -Method POST -Headers $headers -Body $logBody
        Write-Host "   ❌ ERROR: Action was NOT blocked! Status: $($logResponse.status)" -ForegroundColor Red
    } catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        if ($statusCode -eq 403) {
            $errorBody = $_.ErrorDetails.Message | ConvertFrom-Json
            Write-Host "   ✅ Action correctly BLOCKED (403 Forbidden)" -ForegroundColor Green
            Write-Host "   Status: $($errorBody.status)" -ForegroundColor White
            Write-Host "   Reason: $($errorBody.reason)" -ForegroundColor White
        } else {
            Write-Host "   ❌ Unexpected status code: $statusCode" -ForegroundColor Red
        }
    }
} catch {
    Write-Host "   ❌ Test failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 3: Create PARTIAL restriction
Write-Host "`n3️⃣  Testing: Create PARTIAL restriction (specific actions)" -ForegroundColor Cyan
try {
    # First lift the FULL restriction
    $liftBody = @{
        user_id = $testUserId
        reason = "Changing to partial restriction"
    } | ConvertTo-Json

    Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/lift_restriction" `
        -Method POST -Headers $headers -Body $liftBody | Out-Null

    # Create PARTIAL restriction
    $partialBody = @{
        user_id = $testUserId
        restriction_type = "PARTIAL"
        restricted_actions = @("credit_scoring", "automated_decision")
        reason = "User objects to specific AI actions"
    } | ConvertTo-Json

    $response = Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/restrict" `
        -Method POST -Headers $headers -Body $partialBody

    Write-Host "   ✅ PARTIAL restriction created: $($response.restriction_id)" -ForegroundColor Green
} catch {
    Write-Host "   ❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.ErrorDetails.Message) {
        Write-Host "   Response: $($_.ErrorDetails.Message)" -ForegroundColor Yellow
    }
}

# Test 4: Try blocked action (should be blocked)
Write-Host "`n4️⃣  Testing: log_action with PARTIAL restriction - blocked action" -ForegroundColor Cyan
try {
    $logBody = @{
        agent_id = "test-agent-restriction"
        action = "credit_scoring"
        payload = '{"test": "data"}'
        user_id = $testUserId
        target_region = "EU"
    } | ConvertTo-Json

    try {
        $logResponse = Invoke-RestMethod -Uri "$API_BASE/log_action" `
            -Method POST -Headers $headers -Body $logBody
        Write-Host "   ❌ ERROR: Blocked action was NOT blocked!" -ForegroundColor Red
    } catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        if ($statusCode -eq 403) {
            Write-Host "   ✅ Blocked action correctly BLOCKED (403 Forbidden)" -ForegroundColor Green
        } else {
            Write-Host "   ❌ Unexpected status code: $statusCode" -ForegroundColor Red
        }
    }
} catch {
    Write-Host "   ❌ Test failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 5: Try allowed action (should pass)
Write-Host "`n5️⃣  Testing: log_action with PARTIAL restriction - allowed action" -ForegroundColor Cyan
try {
    $logBody = @{
        agent_id = "test-agent-restriction"
        action = "data_analysis"  # Not in restricted list
        payload = '{"test": "data"}'
        user_id = $testUserId
        target_region = "EU"
    } | ConvertTo-Json

    $logResponse = Invoke-RestMethod -Uri "$API_BASE/log_action" `
        -Method POST -Headers $headers -Body $logBody

    if ($logResponse.status -eq "COMPLIANT") {
        Write-Host "   ✅ Allowed action correctly PASSED (COMPLIANT)" -ForegroundColor Green
        Write-Host "   Seal ID: $($logResponse.seal_id)" -ForegroundColor White
    } else {
        Write-Host "   ⚠️  Unexpected status: $($logResponse.status)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   ❌ Test failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 6: Get all restrictions
Write-Host "`n6️⃣  Testing: Get all restrictions for user" -ForegroundColor Cyan
try {
    $restrictions = Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/restrictions" `
        -Method GET -Headers $headers

    Write-Host "   ✅ Found $($restrictions.restrictions.Count) restriction(s)" -ForegroundColor Green
    foreach ($restriction in $restrictions.restrictions) {
        Write-Host "   - $($restriction.restriction_id): $($restriction.restriction_type) - $($restriction.status)" -ForegroundColor White
    }
} catch {
    Write-Host "   ❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 7: Lift restriction
Write-Host "`n7️⃣  Testing: Lift processing restriction" -ForegroundColor Cyan
try {
    $liftBody = @{
        user_id = $testUserId
        reason = "User requested removal of restriction"
        lifted_by = "test-admin"
    } | ConvertTo-Json

    $response = Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/lift_restriction" `
        -Method POST -Headers $headers -Body $liftBody

    Write-Host "   ✅ Restriction lifted successfully" -ForegroundColor Green
    Write-Host "   Status: $($response.status)" -ForegroundColor White
} catch {
    Write-Host "   ❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.ErrorDetails.Message) {
        Write-Host "   Response: $($_.ErrorDetails.Message)" -ForegroundColor Yellow
    }
}

# Test 8: Verify action works after lifting
Write-Host "`n8️⃣  Testing: log_action after lifting restriction (should work)" -ForegroundColor Cyan
try {
    $logBody = @{
        agent_id = "test-agent-restriction"
        action = "credit_scoring"
        payload = '{"test": "data"}'
        user_id = $testUserId
        target_region = "EU"
    } | ConvertTo-Json

    $logResponse = Invoke-RestMethod -Uri "$API_BASE/log_action" `
        -Method POST -Headers $headers -Body $logBody

    if ($logResponse.status -eq "COMPLIANT") {
        Write-Host "   ✅ Action correctly PASSED after lifting restriction" -ForegroundColor Green
        Write-Host "   Seal ID: $($logResponse.seal_id)" -ForegroundColor White
    } else {
        Write-Host "   ⚠️  Unexpected status: $($logResponse.status)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   ❌ Test failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n═══════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "✅ ALL TESTS COMPLETED" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════════════`n" -ForegroundColor Green

