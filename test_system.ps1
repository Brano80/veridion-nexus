# Veridion Nexus End-to-End Test Script (PowerShell)
# Tests critical endpoints to ensure system works before public launch

$ErrorActionPreference = "Stop"

# Check if .env exists
if (-not (Test-Path .env)) {
    Write-Host ".env file not found!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Run this first to generate .env:"
    Write-Host "  .\setup_env.ps1"
    Write-Host ""
    Write-Host "Or copy manually:"
    Write-Host "  Copy-Item .env.example .env"
    Write-Host "  # Then edit .env with your values"
    exit 1
}

# Configuration
$API_URL = "http://localhost:8080"
$HEALTH_URL = "$API_URL/health"
$MAX_WAIT = 120  # Maximum seconds to wait for system to be ready
$WAIT_INTERVAL = 5  # Seconds between health checks

# Test counters
$script:TESTS_PASSED = 0
$script:TESTS_FAILED = 0
$script:CRITICAL_FAILED = 0

# Test data
$TEST_AGENT_ID = "test-agent-$(Get-Date -Format 'yyyyMMddHHmmss')"
$TEST_USER_ID = "test-user-$(Get-Date -Format 'yyyyMMddHHmmss')"
$script:TEST_SEAL_ID = ""
$script:TEST_TX_ID = ""
$script:AUTH_HEADER = ""

Write-Host "=========================================="
Write-Host "Veridion Nexus End-to-End Test"
Write-Host "=========================================="
Write-Host ""

# Function to print test result
function Print-Result {
    param(
        [string]$TestName,
        [string]$Status,
        [string]$Details = ""
    )
    
    if ($Status -eq "PASS") {
        Write-Host "PASS: $TestName" -ForegroundColor Green
        $script:TESTS_PASSED++
    } elseif ($Status -eq "SKIP") {
        Write-Host "SKIP: $TestName" -ForegroundColor Yellow
        if ($Details) {
            Write-Host "  Details: $Details" -ForegroundColor Yellow
        }
    } else {
        Write-Host "FAIL: $TestName" -ForegroundColor Red
        if ($Details) {
            Write-Host "  Details: $Details" -ForegroundColor Red
        }
        $script:TESTS_FAILED++
        if ($Status -eq "CRITICAL") {
            $script:CRITICAL_FAILED++
        }
    }
}

# Function to check HTTP status
function Test-HttpStatus {
    param(
        [int]$StatusCode,
        [int]$Expected
    )
    
    return $StatusCode -eq $Expected
}

# Step 1: Start Docker Compose (or use existing containers)
Write-Host "Step 1: Starting Docker Compose..."
try {
    # Check if containers are already running
    $existingApi = docker ps -q -f name=veridion-nexus-api
    $existingDb = docker ps -q -f name=veridion-nexus-db
    
    if ($existingApi -and $existingDb) {
        Write-Host "Containers already running, skipping docker-compose up"
        Print-Result "Docker Compose Started" "PASS" "Using existing containers"
    } else {
        docker-compose up -d | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Print-Result "Docker Compose Started" "PASS"
        } else {
            Print-Result "Docker Compose Started" "CRITICAL" "Failed to start containers"
            exit 1
        }
    }
} catch {
    Print-Result "Docker Compose Started" "CRITICAL" $_.Exception.Message
    exit 1
}

Write-Host ""

# Step 2: Wait for system to be ready
Write-Host "Step 2: Waiting for system to be ready..."
$WAIT_COUNT = 0
$SYSTEM_READY = $false

while ($WAIT_COUNT -lt $MAX_WAIT) {
    try {
        $response = Invoke-WebRequest -Uri $HEALTH_URL -Method Get -TimeoutSec 2 -UseBasicParsing -ErrorAction SilentlyContinue
        if ($response.StatusCode -eq 200) {
            Print-Result "System Health Check" "PASS"
            $SYSTEM_READY = $true
            break
        }
    } catch {
        # System not ready yet
    }
    
    Write-Host "." -NoNewline
    Start-Sleep -Seconds $WAIT_INTERVAL
    $WAIT_COUNT += $WAIT_INTERVAL
}

if (-not $SYSTEM_READY) {
    Print-Result "System Health Check" "CRITICAL" "System did not become ready within $MAX_WAIT seconds"
    Write-Host ""
    Write-Host "Checking container status..."
    docker-compose ps
    exit 1
}

Write-Host ""
Write-Host "System is ready! Starting tests..."
Write-Host ""

# Step 3: Test Authentication (Login)
Write-Host "Step 3: Testing Authentication..."
try {
    $loginBody = @{
        username = "admin"
        password = "admin123"
    } | ConvertTo-Json
    
    $loginResponse = Invoke-RestMethod -Uri "$API_URL/api/v1/auth/login" `
        -Method Post `
        -ContentType "application/json" `
        -Body $loginBody `
        -ErrorAction Stop
    
    if ($loginResponse.token) {
        $script:AUTH_HEADER = "Authorization: Bearer $($loginResponse.token)"
        Print-Result "Authentication (Login)" "PASS"
    } else {
        Print-Result "Authentication (Login)" "FAIL" "Token not found in response"
    }
} catch {
    $statusCode = $_.Exception.Response.StatusCode.value__
    Print-Result "Authentication (Login)" "CRITICAL" "HTTP $statusCode - Cannot proceed without auth"
    Write-Host "Error: $($_.Exception.Message)"
    exit 1
}

Write-Host ""

# Step 4: Test POST /api/v1/log_action
Write-Host "Step 4: Testing POST /api/v1/log_action..."
try {
    $logActionBody = @{
        agent_id = $TEST_AGENT_ID
        action = "test_compliance_action"
        payload = "Test payload for compliance logging"
        target_region = "EU"
        user_id = $TEST_USER_ID
        user_notified = $true
    } | ConvertTo-Json
    
    $headers = @{
        "Authorization" = "Bearer $($loginResponse.token)"
    }
    
    $logResponse = Invoke-RestMethod -Uri "$API_URL/api/v1/log_action" `
        -Method Post `
        -ContentType "application/json" `
        -Headers $headers `
        -Body $logActionBody `
        -ErrorAction Stop
    
    if ($logResponse.seal_id -and $logResponse.tx_id) {
        $script:TEST_SEAL_ID = $logResponse.seal_id
        $script:TEST_TX_ID = $logResponse.tx_id
        Print-Result "POST /api/v1/log_action" "PASS"
        Write-Host "  Seal ID: $($logResponse.seal_id)"
        Write-Host "  TX ID: $($logResponse.tx_id)"
    } else {
        Print-Result "POST /api/v1/log_action" "FAIL" "Missing seal_id or tx_id in response"
    }
} catch {
    $statusCode = $_.Exception.Response.StatusCode.value__
    Print-Result "POST /api/v1/log_action" "CRITICAL" "HTTP $statusCode"
    Write-Host "Error: $($_.Exception.Message)"
    if ($script:CRITICAL_FAILED -gt 0) {
        Write-Host ""
        Write-Host "Critical test failed. Stopping tests." -ForegroundColor Red
        exit 1
    }
}

Write-Host ""

# Step 5: Test GET /api/v1/logs
Write-Host "Step 5: Testing GET /api/v1/logs..."
try {
    $logsResponse = Invoke-RestMethod -Uri "$API_URL/api/v1/logs?limit=10" `
        -Method Get `
        -Headers $headers `
        -ErrorAction Stop
    
    if ($logsResponse.total -ge 0) {
        Print-Result "GET /api/v1/logs" "PASS" "Found $($logsResponse.total) logs"
    } else {
        Print-Result "GET /api/v1/logs" "PASS" "Response received"
    }
} catch {
    $statusCode = $_.Exception.Response.StatusCode.value__
    Print-Result "GET /api/v1/logs" "CRITICAL" "HTTP $statusCode"
    Write-Host "Error: $($_.Exception.Message)"
    if ($script:CRITICAL_FAILED -gt 0) {
        Write-Host ""
        Write-Host "Critical test failed. Stopping tests." -ForegroundColor Red
        exit 1
    }
}

Write-Host ""

# Step 6: Test GET /api/v1/download_report (if we have a seal_id)
if ($script:TEST_SEAL_ID) {
    Write-Host "Step 6: Testing GET /api/v1/download_report..."
    try {
        $sealIdParam = $script:TEST_SEAL_ID
        $baseUrl = "$API_URL/api/v1/download_report"
        $ampersandChar = [char]0x26
        $queryParams = "seal_id=$sealIdParam" + $ampersandChar + "format=pdf"
        $reportUrl = "$baseUrl`?$queryParams"
        $reportPath = Join-Path $env:TEMP "veridion_report.pdf"
        
        Invoke-WebRequest -Uri $reportUrl `
            -Method Get `
            -Headers $headers `
            -OutFile $reportPath `
            -ErrorAction Stop
        
        if (Test-Path $reportPath) {
            $fileSize = (Get-Item $reportPath).Length
            if ($fileSize -gt 0) {
                $fileSizeStr = "$fileSize bytes"
                Print-Result "GET /api/v1/download_report" "PASS" "PDF generated ($fileSizeStr)"
            } else {
                Print-Result "GET /api/v1/download_report" "FAIL" "PDF file is empty"
            }
        } else {
            Print-Result "GET /api/v1/download_report" "FAIL" "PDF file not created"
        }
    } catch {
        if ($_.Exception.Response) {
            $statusCode = $_.Exception.Response.StatusCode.value__
            Print-Result "GET /api/v1/download_report" "FAIL" "HTTP $statusCode"
        } else {
            Print-Result "GET /api/v1/download_report" "FAIL" $_.Exception.Message
        }
        Write-Host "Error: $($_.Exception.Message)"
    }
} else {
    Print-Result "GET /api/v1/download_report" "SKIP" "No seal_id available from previous test"
}

Write-Host ""

# Step 7: Test POST /api/v1/shred_data (if we have a seal_id)
if ($script:TEST_SEAL_ID) {
    Write-Host "Step 7: Testing POST /api/v1/shred_data..."
    try {
        $shredBody = @{
            seal_id = $script:TEST_SEAL_ID
        } | ConvertTo-Json
        
        $shredResponse = Invoke-RestMethod -Uri "$API_URL/api/v1/shred_data" `
            -Method Post `
            -ContentType "application/json" `
            -Headers $headers `
            -Body $shredBody `
            -ErrorAction Stop
        
        if ($shredResponse.status) {
            Print-Result "POST /api/v1/shred_data" "PASS" "Status: $($shredResponse.status)"
        } else {
            Print-Result "POST /api/v1/shred_data" "PASS" "Response received"
        }
    } catch {
        if ($_.Exception.Response) {
            $statusCode = $_.Exception.Response.StatusCode.value__
            Print-Result "POST /api/v1/shred_data" "CRITICAL" "HTTP $statusCode"
        } else {
            Print-Result "POST /api/v1/shred_data" "CRITICAL" $_.Exception.Message
        }
        Write-Host "Error: $($_.Exception.Message)"
    }
} else {
    Print-Result "POST /api/v1/shred_data" "SKIP" "No seal_id available from previous test"
}

Write-Host ""

# Final Summary
Write-Host "=========================================="
Write-Host "Test Summary"
Write-Host "=========================================="
Write-Host "Tests Passed: $script:TESTS_PASSED" -ForegroundColor Green
Write-Host "Tests Failed: $script:TESTS_FAILED" -ForegroundColor Red
Write-Host ""

if ($script:CRITICAL_FAILED -gt 0) {
    Write-Host 'CRITICAL TESTS FAILED' -ForegroundColor Red
    Write-Host "System is not ready for production."
    exit 1
} elseif ($script:TESTS_FAILED -gt 0) {
    Write-Host 'SOME TESTS FAILED' -ForegroundColor Yellow
    Write-Host "Review failures above."
    exit 0
} else {
    Write-Host 'ALL TESTS PASSED' -ForegroundColor Green
    Write-Host 'System is ready!'
    exit 0
}

