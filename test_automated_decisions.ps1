# Test Script for GDPR Article 22 - Automated Decision-Making
# Tests automated decision detection, review requests, and appeals

$API_BASE = "http://localhost:8080/api/v1"
$JWT_TOKEN = $env:VERIDION_JWT_TOKEN

if (-not $JWT_TOKEN) {
    Write-Host "❌ VERIDION_JWT_TOKEN not set. Please login first:" -ForegroundColor Red
    Write-Host "   Invoke-RestMethod -Uri '$API_BASE/auth/login' -Method POST -ContentType 'application/json' -Body (@{username='testuser'; password='test123'} | ConvertTo-Json)" -ForegroundColor Yellow
    exit 1
}

Write-Host "`n═══════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "🧪 TESTING GDPR ARTICLE 22 - AUTOMATED DECISION-MAKING" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════════════`n" -ForegroundColor Green

$headers = @{
    "Authorization" = "Bearer $JWT_TOKEN"
    "Content-Type" = "application/json"
}

$testUserId = "test-user-decision-$(Get-Date -Format 'yyyyMMdd-HHmmss')"

# Setup: Grant consent for processing
Write-Host "0️⃣  Setup: Granting consent for processing" -ForegroundColor Cyan
try {
    $consentBody = @{
        user_id = $testUserId
        consent_type = "PROCESSING"
        purpose = "Testing automated decision-making"
        legal_basis = "CONSENT"
        consent_method = "EXPLICIT"
    } | ConvertTo-Json

    Invoke-RestMethod -Uri "$API_BASE/consent" `
        -Method POST -Headers $headers -Body $consentBody | Out-Null
    Write-Host "   ✅ Consent granted" -ForegroundColor Green
} catch {
    Write-Host "   ⚠️  Consent might already exist or failed: $($_.Exception.Message)" -ForegroundColor Yellow
}

# Test 1: Trigger automated decision detection with credit_scoring action
Write-Host "1️⃣  Testing: Trigger automated decision detection (credit_scoring)" -ForegroundColor Cyan
try {
    $logBody = @{
        agent_id = "test-agent-decision"
        action = "credit_scoring"
        payload = '{"credit_score": 650, "decision": "rejected", "reason": "Score below threshold"}'
        user_id = $testUserId
        target_region = "EU"
    } | ConvertTo-Json

    $logResponse = Invoke-RestMethod -Uri "$API_BASE/log_action" `
        -Method POST -Headers $headers -Body $logBody

    if ($logResponse.status -eq "COMPLIANT") {
        Write-Host "   ✅ Action logged successfully" -ForegroundColor Green
        Write-Host "   Seal ID: $($logResponse.seal_id)" -ForegroundColor White
        $sealId = $logResponse.seal_id
    } else {
        Write-Host "   ⚠️  Unexpected status: $($logResponse.status)" -ForegroundColor Yellow
        $sealId = "UNKNOWN"
    }
} catch {
    Write-Host "   ❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.ErrorDetails.Message) {
        Write-Host "   Response: $($_.ErrorDetails.Message)" -ForegroundColor Yellow
    }
    exit 1
}

# Wait a bit for async notification to complete
Start-Sleep -Seconds 2

# Test 2: Get automated decisions for user
Write-Host "`n2️⃣  Testing: Get automated decisions for user" -ForegroundColor Cyan
try {
    $decisions = Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/automated_decisions" `
        -Method GET -Headers $headers

    if ($decisions.decisions.Count -gt 0) {
        Write-Host "   ✅ Found $($decisions.decisions.Count) automated decision(s)" -ForegroundColor Green
        $decision = $decisions.decisions[0]
        Write-Host "   Decision ID: $($decision.decision_id)" -ForegroundColor White
        Write-Host "   Outcome: $($decision.decision_outcome)" -ForegroundColor White
        Write-Host "   Status: $($decision.status)" -ForegroundColor White
        $decisionId = $decision.decision_id
    } else {
        Write-Host "   ⚠️  No automated decisions found (may need to wait for async processing)" -ForegroundColor Yellow
        $decisionId = "NONE"
    }
} catch {
    Write-Host "   ❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
    $decisionId = "NONE"
}

if ($decisionId -ne "NONE") {
    # Test 3: Request human review
    Write-Host "`n3️⃣  Testing: Request human review" -ForegroundColor Cyan
    try {
        $reviewBody = @{
            user_id = $testUserId
            decision_id = $decisionId
            reason = "User requests human review of automated decision"
        } | ConvertTo-Json

        $reviewResponse = Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/request_review" `
            -Method POST -Headers $headers -Body $reviewBody

        Write-Host "   ✅ Review requested successfully" -ForegroundColor Green
        Write-Host "   Status: $($reviewResponse.status)" -ForegroundColor White
        Write-Host "   Message: $($reviewResponse.message)" -ForegroundColor White
    } catch {
        Write-Host "   ❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
        if ($_.ErrorDetails.Message) {
            Write-Host "   Response: $($_.ErrorDetails.Message)" -ForegroundColor Yellow
        }
    }

    # Test 4: Appeal decision
    Write-Host "`n4️⃣  Testing: Appeal automated decision" -ForegroundColor Cyan
    try {
        $appealBody = @{
            user_id = $testUserId
            decision_id = $decisionId
            appeal_reason = "I believe the automated decision was incorrect. My credit history should qualify me for approval."
        } | ConvertTo-Json

        $appealResponse = Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/appeal_decision" `
            -Method POST -Headers $headers -Body $appealBody

        Write-Host "   ✅ Appeal submitted successfully" -ForegroundColor Green
        Write-Host "   Status: $($appealResponse.status)" -ForegroundColor White
        Write-Host "   Appeal requested at: $($appealResponse.appeal_requested_at)" -ForegroundColor White
    } catch {
        Write-Host "   ❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
        if ($_.ErrorDetails.Message) {
            Write-Host "   Response: $($_.ErrorDetails.Message)" -ForegroundColor Yellow
        }
    }

    # Test 5: Get updated decisions
    Write-Host "`n5️⃣  Testing: Get updated decisions (after appeal)" -ForegroundColor Cyan
    try {
        $updatedDecisions = Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId/automated_decisions" `
            -Method GET -Headers $headers

        Write-Host "   ✅ Found $($updatedDecisions.decisions.Count) decision(s)" -ForegroundColor Green
        foreach ($decision in $updatedDecisions.decisions) {
            Write-Host "   - $($decision.decision_id): $($decision.status) - $($decision.decision_outcome)" -ForegroundColor White
        }
    } catch {
        Write-Host "   ❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
    }
} else {
    Write-Host "`n⚠️  Skipping review/appeal tests - no decision found" -ForegroundColor Yellow
}

# Test 6: Test with loan_approval action (another automated decision type)
Write-Host "`n6️⃣  Testing: Trigger automated decision with loan_approval action" -ForegroundColor Cyan
try {
    $loanBody = @{
        agent_id = "test-agent-decision"
        action = "loan_approval"
        payload = '{"loan_amount": 50000, "decision": "approved", "interest_rate": 3.5}'
        user_id = "$testUserId-loan"
        target_region = "EU"
    } | ConvertTo-Json

    $loanResponse = Invoke-RestMethod -Uri "$API_BASE/log_action" `
        -Method POST -Headers $headers -Body $loanBody

    if ($loanResponse.status -eq "COMPLIANT") {
        Write-Host "   ✅ Loan approval action logged" -ForegroundColor Green
        Write-Host "   Seal ID: $($loanResponse.seal_id)" -ForegroundColor White
    } else {
        Write-Host "   ⚠️  Unexpected status: $($loanResponse.status)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   ❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 7: Test with non-automated decision action (should not create decision record)
Write-Host "`n7️⃣  Testing: Non-automated action (should NOT create decision)" -ForegroundColor Cyan
try {
    $normalBody = @{
        agent_id = "test-agent-decision"
        action = "data_analysis"
        payload = '{"analysis_type": "statistical", "result": "completed"}'
        user_id = "$testUserId-normal"
        target_region = "EU"
    } | ConvertTo-Json

    $normalResponse = Invoke-RestMethod -Uri "$API_BASE/log_action" `
        -Method POST -Headers $headers -Body $normalBody

    if ($normalResponse.status -eq "COMPLIANT") {
        Write-Host "   ✅ Normal action logged (no automated decision created)" -ForegroundColor Green
        
        # Verify no decision was created
        Start-Sleep -Seconds 2
        try {
            $decisionsCheck = Invoke-RestMethod -Uri "$API_BASE/data_subject/$testUserId-normal/automated_decisions" `
                -Method GET -Headers $headers
            
            if ($decisionsCheck.decisions.Count -eq 0) {
                Write-Host "   ✅ Confirmed: No automated decision created for non-decision action" -ForegroundColor Green
            } else {
                Write-Host "   ⚠️  Unexpected: Decision was created for non-decision action" -ForegroundColor Yellow
            }
        } catch {
            Write-Host "   ⚠️  Could not verify decision creation" -ForegroundColor Yellow
        }
    } else {
        Write-Host "   ⚠️  Unexpected status: $($normalResponse.status)" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   ❌ Failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n═══════════════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "✅ ALL TESTS COMPLETED" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════════════`n" -ForegroundColor Green

