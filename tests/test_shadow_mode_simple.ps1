# Simple Shadow Mode Test Script
# Quick test to verify Shadow Mode is working

$API_BASE = "http://127.0.0.1:8080/api/v1"
$ErrorActionPreference = "Stop"

Write-Host "`n=== Shadow Mode Quick Test ===" -ForegroundColor Cyan
Write-Host ""

# Test 1: Get current mode
Write-Host "1. Getting current enforcement mode..." -ForegroundColor Yellow
try {
    $mode = Invoke-RestMethod -Uri "$API_BASE/system/enforcement-mode" -Method GET
    Write-Host "   Current mode: $($mode.enforcement_mode)" -ForegroundColor Green
} catch {
    Write-Host "   ERROR: Could not get enforcement mode" -ForegroundColor Red
    Write-Host "   Make sure server is running on $API_BASE" -ForegroundColor Red
    exit 1
}

# Test 2: Set to SHADOW
Write-Host "`n2. Setting mode to SHADOW..." -ForegroundColor Yellow
try {
    $body = @{
        enforcement_mode = "SHADOW"
        description = "Automated test"
    } | ConvertTo-Json
    
    $result = Invoke-RestMethod -Uri "$API_BASE/system/enforcement-mode" -Method POST -Body $body -ContentType "application/json"
    Write-Host "   Mode set to: $($result.enforcement_mode)" -ForegroundColor Green
} catch {
    Write-Host "   ERROR: Could not set mode" -ForegroundColor Red
    exit 1
}

# Test 3: Send test log_action
Write-Host "`n3. Sending test log_action request..." -ForegroundColor Yellow
try {
    $timestamp = [DateTimeOffset]::Now.ToUnixTimeSeconds()
    $logBody = @{
        agent_id = "test-shadow-$timestamp"
        action = "test_action"
        payload = "test payload"
    } | ConvertTo-Json
    
    $logResponse = Invoke-RestMethod -Uri "$API_BASE/log_action" -Method POST -Body $logBody -ContentType "application/json"
    Write-Host "   Response status: $($logResponse.status)" -ForegroundColor Green
    if ($logResponse.status -like "*SHADOW*") {
        Write-Host "   ✓ Shadow mode correctly identified" -ForegroundColor Green
    }
} catch {
    Write-Host "   ERROR: Could not send log_action" -ForegroundColor Red
    Write-Host "   Error: $_" -ForegroundColor Red
}

# Test 4: Get analytics
Write-Host "`n4. Getting shadow mode analytics..." -ForegroundColor Yellow
try {
    $analytics = Invoke-RestMethod -Uri "$API_BASE/analytics/shadow-mode?days=7" -Method GET
    Write-Host "   Total logs: $($analytics.total_logs)" -ForegroundColor Green
    Write-Host "   Would block: $($analytics.would_block_count)" -ForegroundColor Green
    Write-Host "   Would allow: $($analytics.would_allow_count)" -ForegroundColor Green
    Write-Host "   Block %: $([math]::Round($analytics.block_percentage, 2))%" -ForegroundColor Green
    Write-Host "   Confidence: $([math]::Round($analytics.confidence_score, 1))%" -ForegroundColor Green
} catch {
    Write-Host "   ERROR: Could not get analytics" -ForegroundColor Red
}

# Test 5: Test export
Write-Host "`n5. Testing export (CSV)..." -ForegroundColor Yellow
try {
    $exportUrl = "$API_BASE/analytics/shadow-mode/export?format=csv`&days=7"
    $export = Invoke-WebRequest -Uri $exportUrl -Method GET
    if ($export.Content -like "*id,agent_id*") {
        Write-Host "   ✓ CSV export working" -ForegroundColor Green
        Write-Host "   File size: $($export.Content.Length) bytes" -ForegroundColor Gray
    } else {
        Write-Host "   ⚠ CSV format may be incorrect" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   ERROR: Could not export" -ForegroundColor Red
    Write-Host "   Error: $_" -ForegroundColor Red
}

# Test 6: Reset to ENFORCING
Write-Host "`n6. Resetting to ENFORCING mode..." -ForegroundColor Yellow
try {
    $resetBody = @{
        enforcement_mode = "ENFORCING"
        description = "Test complete"
    } | ConvertTo-Json
    
    $reset = Invoke-RestMethod -Uri "$API_BASE/system/enforcement-mode" -Method POST -Body $resetBody -ContentType "application/json"
    Write-Host "   Mode reset to: $($reset.enforcement_mode)" -ForegroundColor Green
} catch {
    Write-Host "   WARNING: Could not reset mode" -ForegroundColor Yellow
}

Write-Host "`n=== Test Complete ===" -ForegroundColor Cyan
Write-Host "Check results above. All tests should show green checkmarks." -ForegroundColor Gray
Write-Host ""

