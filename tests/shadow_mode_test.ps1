# Shadow Mode Complete Test Suite (PowerShell)
# Tests all Shadow Mode functionality

$API_BASE = "http://127.0.0.1:8080/api/v1"
$AUTH_TOKEN = $env:AUTH_TOKEN

$TestsPassed = 0
$TestsFailed = 0

function Print-Test {
    param(
        [string]$TestName,
        [string]$Status
    )
    if ($Status -eq "PASS") {
        Write-Host "✓ $TestName" -ForegroundColor Green
        $script:TestsPassed++
    } else {
        Write-Host "✗ $TestName" -ForegroundColor Red
        $script:TestsFailed++
    }
}

function Invoke-ApiCall {
    param(
        [string]$Method,
        [string]$Endpoint,
        [string]$Data = $null
    )
    
    $headers = @{
        "Content-Type" = "application/json"
    }
    
    if ($AUTH_TOKEN) {
        $headers["Authorization"] = "Bearer $AUTH_TOKEN"
    }
    
    if ($Data) {
        return Invoke-RestMethod -Uri "$API_BASE$Endpoint" -Method $Method -Headers $headers -Body $Data
    } else {
        return Invoke-RestMethod -Uri "$API_BASE$Endpoint" -Method $Method -Headers $headers
    }
}

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "Shadow Mode Complete Test Suite" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""

# Test 1: Get current enforcement mode
Write-Host "Test 1: Get current enforcement mode"
try {
    $response = Invoke-ApiCall -Method "GET" -Endpoint "/system/enforcement-mode"
    if ($response.enforcement_mode) {
        Print-Test "Get enforcement mode" "PASS"
        Write-Host "  Current mode: $($response.enforcement_mode)" -ForegroundColor Gray
    } else {
        Print-Test "Get enforcement mode" "FAIL"
    }
} catch {
    Print-Test "Get enforcement mode" "FAIL"
    Write-Host "  Error: $_" -ForegroundColor Red
}
Write-Host ""

# Test 2: Set enforcement mode to SHADOW
Write-Host "Test 2: Set enforcement mode to SHADOW"
try {
    $data = @{
        enforcement_mode = "SHADOW"
        description = "Testing shadow mode"
    } | ConvertTo-Json
    
    $response = Invoke-ApiCall -Method "POST" -Endpoint "/system/enforcement-mode" -Data $data
    if ($response.enforcement_mode -eq "SHADOW") {
        Print-Test "Set enforcement mode to SHADOW" "PASS"
    } else {
        Print-Test "Set enforcement mode to SHADOW" "FAIL"
    }
} catch {
    Print-Test "Set enforcement mode to SHADOW" "FAIL"
    Write-Host "  Error: $_" -ForegroundColor Red
}
Write-Host ""

# Test 3: Verify shadow mode is active
Write-Host "Test 3: Verify shadow mode is active"
try {
    $response = Invoke-ApiCall -Method "GET" -Endpoint "/system/enforcement-mode"
    if ($response.enforcement_mode -eq "SHADOW") {
        Print-Test "Verify shadow mode is active" "PASS"
    } else {
        Print-Test "Verify shadow mode is active" "FAIL"
    }
} catch {
    Print-Test "Verify shadow mode is active" "FAIL"
    Write-Host "  Error: $_" -ForegroundColor Red
}
Write-Host ""

# Test 4: Send log_action request in shadow mode
Write-Host "Test 4: Send log_action request in shadow mode"
try {
    $timestamp = [DateTimeOffset]::Now.ToUnixTimeSeconds()
    $data = @{
        agent_id = "test-agent-shadow-$timestamp"
        action = "test_action"
        payload = "test payload for shadow mode"
    } | ConvertTo-Json
    
    $response = Invoke-ApiCall -Method "POST" -Endpoint "/log_action" -Data $data
    if ($response.status) {
        Print-Test "Send log_action in shadow mode" "PASS"
        if ($response.status -like "*SHADOW*") {
            Print-Test "Shadow mode correctly logs violations without blocking" "PASS"
        } else {
            Print-Test "Shadow mode correctly logs violations without blocking" "FAIL"
        }
    } else {
        Print-Test "Send log_action in shadow mode" "FAIL"
    }
} catch {
    Print-Test "Send log_action in shadow mode" "FAIL"
    Write-Host "  Error: $_" -ForegroundColor Red
}
Write-Host ""

# Test 5: Get shadow mode analytics
Write-Host "Test 5: Get shadow mode analytics"
try {
    $response = Invoke-ApiCall -Method "GET" -Endpoint "/analytics/shadow-mode?days=7"
    if ($response.total_logs -ge 0) {
        Print-Test "Get shadow mode analytics" "PASS"
        Write-Host "  Total logs: $($response.total_logs)" -ForegroundColor Gray
        Write-Host "  Would block: $($response.would_block_count)" -ForegroundColor Gray
        Write-Host "  Would allow: $($response.would_allow_count)" -ForegroundColor Gray
        Write-Host "  Confidence score: $($response.confidence_score)" -ForegroundColor Gray
    } else {
        Print-Test "Get shadow mode analytics" "FAIL"
    }
} catch {
    Print-Test "Get shadow mode analytics" "FAIL"
    Write-Host "  Error: $_" -ForegroundColor Red
}
Write-Host ""

# Test 6: Get shadow mode analytics with agent filter
Write-Host "Test 6: Get shadow mode analytics with agent filter"
try {
    $agentId = "test-agent-shadow-$timestamp"
    $response = Invoke-ApiCall -Method "GET" -Endpoint "/analytics/shadow-mode?days=7&agent_id=$agentId"
    if ($response.total_logs -ge 0) {
        Print-Test "Get shadow mode analytics with agent filter" "PASS"
    } else {
        Print-Test "Get shadow mode analytics with agent filter" "FAIL"
    }
} catch {
    Print-Test "Get shadow mode analytics with agent filter" "FAIL"
    Write-Host "  Error: $_" -ForegroundColor Red
}
Write-Host ""

# Test 7: Export shadow mode logs (CSV)
Write-Host "Test 7: Export shadow mode logs (CSV)"
try {
    $response = Invoke-WebRequest -Uri "$API_BASE/analytics/shadow-mode/export?format=csv&days=7" -Method "GET" -Headers @{
        "Authorization" = "Bearer $AUTH_TOKEN"
    } -ErrorAction SilentlyContinue
    
    if ($response.Content -like "*id,agent_id*") {
        Print-Test "Export shadow mode logs (CSV)" "PASS"
        $lines = ($response.Content -split "`n").Count
        Write-Host "  CSV lines: $lines" -ForegroundColor Gray
    } else {
        Print-Test "Export shadow mode logs (CSV)" "FAIL"
    }
} catch {
    Print-Test "Export shadow mode logs (CSV)" "FAIL"
    Write-Host "  Error: $_" -ForegroundColor Red
}
Write-Host ""

# Test 8: Export shadow mode logs (JSON)
Write-Host "Test 8: Export shadow mode logs (JSON)"
try {
    $response = Invoke-ApiCall -Method "GET" -Endpoint "/analytics/shadow-mode/export?format=json&days=7"
    if ($response -is [Array] -or ($response -is [PSCustomObject])) {
        Print-Test "Export shadow mode logs (JSON)" "PASS"
        $count = if ($response -is [Array]) { $response.Count } else { 1 }
        Write-Host "  JSON records: $count" -ForegroundColor Gray
    } else {
        Print-Test "Export shadow mode logs (JSON)" "FAIL"
    }
} catch {
    Print-Test "Export shadow mode logs (JSON)" "FAIL"
    Write-Host "  Error: $_" -ForegroundColor Red
}
Write-Host ""

# Test 9: Export with filters
Write-Host "Test 9: Export with filters (would_block=true)"
try {
    $response = Invoke-WebRequest -Uri "$API_BASE/analytics/shadow-mode/export?format=csv&days=7&would_block=true" -Method "GET" -Headers @{
        "Authorization" = "Bearer $AUTH_TOKEN"
    } -ErrorAction SilentlyContinue
    
    if ($response.Content -like "*id,agent_id*") {
        Print-Test "Export with filters" "PASS"
    } else {
        Print-Test "Export with filters" "FAIL"
    }
} catch {
    Print-Test "Export with filters" "FAIL"
    Write-Host "  Error: $_" -ForegroundColor Red
}
Write-Host ""

# Test 10: Set enforcement mode back to ENFORCING
Write-Host "Test 10: Set enforcement mode back to ENFORCING"
try {
    $data = @{
        enforcement_mode = "ENFORCING"
        description = "Testing complete"
    } | ConvertTo-Json
    
    $response = Invoke-ApiCall -Method "POST" -Endpoint "/system/enforcement-mode" -Data $data
    if ($response.enforcement_mode -eq "ENFORCING") {
        Print-Test "Set enforcement mode to ENFORCING" "PASS"
    } else {
        Print-Test "Set enforcement mode to ENFORCING" "FAIL"
    }
} catch {
    Print-Test "Set enforcement mode to ENFORCING" "FAIL"
    Write-Host "  Error: $_" -ForegroundColor Red
}
Write-Host ""

# Summary
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "Test Summary" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "Tests Passed: $TestsPassed" -ForegroundColor Green
Write-Host "Tests Failed: $TestsFailed" -ForegroundColor Red
Write-Host "Total Tests: $($TestsPassed + $TestsFailed)"
Write-Host ""

if ($TestsFailed -eq 0) {
    Write-Host "All tests passed! ✓" -ForegroundColor Green
    exit 0
} else {
    Write-Host "Some tests failed! ✗" -ForegroundColor Red
    exit 1
}

