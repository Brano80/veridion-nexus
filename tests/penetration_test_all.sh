#!/bin/bash
# Comprehensive Penetration Test Suite
# Runs all penetration tests and generates summary report

BASE_URL="${1:-http://localhost:8080}"
REPORT_FILE="PENETRATION_TEST_RESULTS_$(date +%Y%m%d_%H%M%S).txt"

echo "=========================================="
echo "Veridion Nexus - Penetration Test Suite"
echo "=========================================="
echo "Target: $BASE_URL"
echo "Report: $REPORT_FILE"
echo ""

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo "ERROR: python3 not found. Please install Python 3."
    exit 1
fi

# Check if required Python packages are installed
echo "[*] Checking dependencies..."
python3 -c "import jwt, requests" 2>/dev/null
if [ $? -ne 0 ]; then
    echo "ERROR: Required Python packages not installed."
    echo "Install with: pip3 install pyjwt requests"
    exit 1
fi

echo "[*] Starting penetration tests..."
echo ""

# Test 1: JWT Secret Exploitation
echo "=========================================="
echo "TEST 1: JWT Secret Exploitation"
echo "=========================================="
python3 tests/penetration_test_jwt.py "$BASE_URL" | tee -a "$REPORT_FILE"
echo ""

# Test 2: SQL Injection
echo "=========================================="
echo "TEST 2: SQL Injection"
echo "=========================================="
python3 tests/penetration_test_sql.py "$BASE_URL" | tee -a "$REPORT_FILE"
echo ""

# Test 3: Rate Limiting
echo "=========================================="
echo "TEST 3: Rate Limiting Bypass"
echo "=========================================="
python3 tests/penetration_test_rate_limit.py "$BASE_URL" | tee -a "$REPORT_FILE"
echo ""

echo "=========================================="
echo "All tests completed!"
echo "Full report saved to: $REPORT_FILE"
echo "=========================================="

