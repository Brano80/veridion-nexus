# Test Script for GDPR Article 21 - Processing Objections
# Tests all objection endpoints and integration with log_action

$API_BASE = "http://localhost:8080/api/v1"
$JWT_TOKEN = $env:VERIDION_JWT_TOKEN

if (-not $JWT_TOKEN) {
    Write-Host "❌ VERIDION_JWT_TOKEN not set. Please login first:" -ForegroundColor Red
    Write-Host "   Invoke-RestMethod -Uri '$API_BASE/auth/login' -Method POST -ContentType 'application/json' -Body (@{username='testuser'; password='test123'} | ConvertTo-Json)" -ForegroundColor Yellow
    exit 1
}

Write-Host "`n═══════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "🧪 TESTING GDPR ARTICLE 21 - PROCESSING OBJECTIONS" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════════════`n" -ForegroundColor Green

$headers = @{
    "Authorization" = "Bearer $JWT_TOKEN"
    "Content-Type" = "application/json"
}

$testUserId = "test-user-objection-$(Get-Date -Format 'yyyyMMdd-HHmmss')"

# Test 1: Create FULL objection
Write-Host "1️⃣  Testing: Create FULL processing objection" -ForegroundColor Cyan
try {
    $objectBody = @{
        user_id = $testUserId
        objection_type = "FULL"
        reason = "User objects to all processing (GDPR Article 21)"
    } | ConvertTo-Json

    $response = Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/object" `
        -Method POST -Headers $headers -Body $objectBody

    Write-Host "   ✅ FULL objection created: $($response.objection_id)" -ForegroundColor Green
    $objectionId = $response.objection_id
} catch {
    Write-Host "   ❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.ErrorDetails.Message) {
        Write-Host "   Response: $($_.ErrorDetails.Message)" -ForegroundColor Yellow
    }
    exit 1
}

# Test 2: Try to log action with objected user (should be blocked)
Write-Host "`n2️⃣  Testing: log_action with FULL objection (should be BLOCKED)" -ForegroundColor Cyan
try {
    $logBody = @{
        agent_id = "test-agent-objection"
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

# Test 3: Create PARTIAL objection
Write-Host "`n3️⃣  Testing: Create PARTIAL objection (specific actions)" -ForegroundColor Cyan
try {
    # First withdraw the FULL objection
    $withdrawBody = @{
        user_id = $testUserId
        reason = "Changing to partial objection"
    } | ConvertTo-Json

    Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/withdraw_objection" `
        -Method POST -Headers $headers -Body $withdrawBody | Out-Null

    # Create PARTIAL objection
    $partialBody = @{
        user_id = $testUserId
        objection_type = "PARTIAL"
        objected_actions = @("credit_scoring", "automated_decision")
        legal_basis = "LEGITIMATE_INTERESTS"
        reason = "User objects to specific AI actions"
    } | ConvertTo-Json

    $response = Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/object" `
        -Method POST -Headers $headers -Body $partialBody

    Write-Host "   ✅ PARTIAL objection created: $($response.objection_id)" -ForegroundColor Green
} catch {
    Write-Host "   ❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.ErrorDetails.Message) {
        Write-Host "   Response: $($_.ErrorDetails.Message)" -ForegroundColor Yellow
    }
}

# Test 4: Try blocked action (should be blocked)
Write-Host "`n4️⃣  Testing: log_action with PARTIAL objection - blocked action" -ForegroundColor Cyan
try {
    $logBody = @{
        agent_id = "test-agent-objection"
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
Write-Host "`n5️⃣  Testing: log_action with PARTIAL objection - allowed action" -ForegroundColor Cyan
try {
    $logBody = @{
        agent_id = "test-agent-objection"
        action = "data_analysis"  # Not in objected list
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

# Test 6: Test DIRECT_MARKETING objection
Write-Host "`n6️⃣  Testing: DIRECT_MARKETING objection" -ForegroundColor Cyan
try {
    # Withdraw current objection
    $withdrawBody = @{
        user_id = $testUserId
        reason = "Testing direct marketing objection"
    } | ConvertTo-Json

    Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/withdraw_objection" `
        -Method POST -Headers $headers -Body $withdrawBody | Out-Null

    # Create DIRECT_MARKETING objection
    $marketingBody = @{
        user_id = $testUserId
        objection_type = "DIRECT_MARKETING"
        reason = "User objects to direct marketing"
    } | ConvertTo-Json

    $response = Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/object" `
        -Method POST -Headers $headers -Body $marketingBody

    Write-Host "   ✅ DIRECT_MARKETING objection created: $($response.objection_id)" -ForegroundColor Green
} catch {
    Write-Host "   ❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 7: Get all objections
Write-Host "`n7️⃣  Testing: Get all objections for user" -ForegroundColor Cyan
try {
    $objections = Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/objections" `
        -Method GET -Headers $headers

    Write-Host "   ✅ Found $($objections.objections.Count) objection(s)" -ForegroundColor Green
    foreach ($objection in $objections.objections) {
        Write-Host "   - $($objection.objection_id): $($objection.objection_type) - $($objection.status)" -ForegroundColor White
    }
} catch {
    Write-Host "   ❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 8: Reject objection (with required reason per GDPR Article 21(1))
Write-Host "`n8️⃣  Testing: Reject objection (with required reason)" -ForegroundColor Cyan
try {
    $rejectBody = @{
        user_id = $testUserId
        rejection_reason = "Processing is necessary for the performance of a task carried out in the public interest (GDPR Article 21(1))"
        rejected_by = "admin-001"
    } | ConvertTo-Json

    $response = Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/reject_objection" `
        -Method POST -Headers $headers -Body $rejectBody

    Write-Host "   ✅ Objection rejected successfully" -ForegroundColor Green
    Write-Host "   Status: $($response.status)" -ForegroundColor White
    Write-Host "   Rejection Reason: $($response.rejection_reason)" -ForegroundColor White
} catch {
    Write-Host "   ❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.ErrorDetails.Message) {
        Write-Host "   Response: $($_.ErrorDetails.Message)" -ForegroundColor Yellow
    }
}

# Test 9: Try to reject without reason (should fail)
Write-Host "`n9️⃣  Testing: Reject objection without reason (should fail per GDPR Article 21(1))" -ForegroundColor Cyan
try {
    # Create new objection first
    $newObjectBody = @{
        user_id = $testUserId
        objection_type = "FULL"
        reason = "Test objection for rejection test"
    } | ConvertTo-Json

    $newObjection = Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/object" `
        -Method POST -Headers $headers -Body $newObjectBody

    # Try to reject without reason
    $rejectBodyNoReason = @{
        user_id = $testUserId
        rejection_reason = ""  # Empty reason
    } | ConvertTo-Json

    try {
        $response = Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/reject_objection" `
            -Method POST -Headers $headers -Body $rejectBodyNoReason
        Write-Host "   ❌ ERROR: Rejection without reason was allowed!" -ForegroundColor Red
    } catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        if ($statusCode -eq 400) {
            $errorBody = $_.ErrorDetails.Message | ConvertFrom-Json
            Write-Host "   ✅ Rejection correctly REJECTED (400 Bad Request)" -ForegroundColor Green
            Write-Host "   Error: $($errorBody.error)" -ForegroundColor White
        } else {
            Write-Host "   ⚠️  Unexpected status code: $statusCode" -ForegroundColor Yellow
        }
    }
} catch {
    Write-Host "   ❌ Test failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n═══════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "✅ ALL TESTS COMPLETED" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════════════`n" -ForegroundColor Green

