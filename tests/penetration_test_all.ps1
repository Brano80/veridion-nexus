# Comprehensive Penetration Test Suite (PowerShell)
# Runs all penetration tests and generates summary report

param(
    [string]$BaseUrl = "http://localhost:8080"
)

$ReportFile = "PENETRATION_TEST_RESULTS_$(Get-Date -Format 'yyyyMMdd_HHmmss').txt"

Write-Host "=========================================="
Write-Host "Veridion Nexus - Penetration Test Suite"
Write-Host "=========================================="
Write-Host "Target: $BaseUrl"
Write-Host "Report: $ReportFile"
Write-Host ""

# Check if Python 3 is available
try {
    $pythonVersion = python --version 2>&1
    Write-Host "[*] Found: $pythonVersion"
} catch {
    Write-Host "ERROR: Python not found. Please install Python 3."
    exit 1
}

# Check if required Python packages are installed
Write-Host "[*] Checking dependencies..."
python -c "import jwt, requests" 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Required Python packages not installed."
    Write-Host "Install with: pip install pyjwt requests"
    exit 1
}

Write-Host "[*] Starting penetration tests..."
Write-Host ""

# Test 1: JWT Secret Exploitation
Write-Host "=========================================="
Write-Host "TEST 1: JWT Secret Exploitation"
Write-Host "=========================================="
python tests/penetration_test_jwt.py $BaseUrl | Tee-Object -FilePath $ReportFile -Append
Write-Host ""

# Test 2: SQL Injection
Write-Host "=========================================="
Write-Host "TEST 2: SQL Injection"
Write-Host "=========================================="
python tests/penetration_test_sql.py $BaseUrl | Tee-Object -FilePath $ReportFile -Append
Write-Host ""

# Test 3: Rate Limiting
Write-Host "=========================================="
Write-Host "TEST 3: Rate Limiting Bypass"
Write-Host "=========================================="
python tests/penetration_test_rate_limit.py $BaseUrl | Tee-Object -FilePath $ReportFile -Append
Write-Host ""

Write-Host "=========================================="
Write-Host "All tests completed!"
Write-Host "Full report saved to: $ReportFile"
Write-Host "=========================================="

