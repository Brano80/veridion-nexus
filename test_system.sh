#!/bin/bash

# Veridion Nexus End-to-End Test Script
# Tests critical endpoints to ensure system works before public launch

set -e  # Exit on error

# Check if .env exists
if [ ! -f .env ]; then
    echo "❌ .env file not found!"
    echo ""
    echo "Run this first to generate .env:"
    echo "  ./setup_env.sh"
    echo ""
    echo "Or copy manually:"
    echo "  cp .env.example .env"
    echo "  # Then edit .env with your values"
    exit 1
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
API_URL="http://localhost:8080"
HEALTH_URL="${API_URL}/health"
MAX_WAIT=120  # Maximum seconds to wait for system to be ready
WAIT_INTERVAL=5  # Seconds between health checks

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
CRITICAL_FAILED=0

# Test data
TEST_AGENT_ID="test-agent-$(date +%s)"
TEST_USER_ID="test-user-$(date +%s)"
TEST_SEAL_ID=""
TEST_TX_ID=""

echo "=========================================="
echo "Veridion Nexus End-to-End Test"
echo "=========================================="
echo ""

# Function to print test result
print_result() {
    local test_name=$1
    local status=$2
    local details=$3
    
    if [ "$status" = "PASS" ]; then
        echo -e "${GREEN}✓ PASS${NC}: $test_name"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗ FAIL${NC}: $test_name"
        if [ -n "$details" ]; then
            echo "  Details: $details"
        fi
        ((TESTS_FAILED++))
        if [ "$status" = "CRITICAL" ]; then
            ((CRITICAL_FAILED++))
        fi
    fi
}

# Function to check HTTP status
check_status() {
    local response=$1
    local expected=$2
    local status_code=$(echo "$response" | grep -oP 'HTTP/\d\.\d \K\d{3}' | head -1)
    
    if [ -z "$status_code" ]; then
        # Try to extract from curl output
        status_code=$(echo "$response" | tail -1 | grep -oP '\d{3}' | head -1)
    fi
    
    if [ "$status_code" = "$expected" ]; then
        return 0
    else
        echo "Expected: $expected, Got: $status_code"
        return 1
    fi
}

# Step 1: Start Docker Compose
echo "Step 1: Starting Docker Compose..."
if docker-compose up -d; then
    print_result "Docker Compose Started" "PASS"
else
    print_result "Docker Compose Started" "CRITICAL" "Failed to start containers"
    exit 1
fi

echo ""

# Step 2: Wait for system to be ready
echo "Step 2: Waiting for system to be ready..."
WAIT_COUNT=0
while [ $WAIT_COUNT -lt $MAX_WAIT ]; do
    if curl -s -f "${HEALTH_URL}" > /dev/null 2>&1; then
        print_result "System Health Check" "PASS"
        break
    fi
    
    echo -n "."
    sleep $WAIT_INTERVAL
    WAIT_COUNT=$((WAIT_COUNT + WAIT_INTERVAL))
done

if [ $WAIT_COUNT -ge $MAX_WAIT ]; then
    print_result "System Health Check" "CRITICAL" "System did not become ready within ${MAX_WAIT} seconds"
    echo ""
    echo "Checking container status..."
    docker-compose ps
    exit 1
fi

echo ""
echo "System is ready! Starting tests..."
echo ""

# Step 3: Test Authentication (Login)
echo "Step 3: Testing Authentication..."
LOGIN_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "${API_URL}/api/v1/auth/login" \
    -H "Content-Type: application/json" \
    -d '{
        "username": "admin",
        "password": "admin123"
    }' 2>&1)

HTTP_BODY=$(echo "$LOGIN_RESPONSE" | head -n -1)
HTTP_CODE=$(echo "$LOGIN_RESPONSE" | tail -n 1)

if [ "$HTTP_CODE" = "200" ]; then
    TOKEN=$(echo "$HTTP_BODY" | grep -oP '"token"\s*:\s*"\K[^"]+' | head -1)
    if [ -n "$TOKEN" ]; then
        print_result "Authentication (Login)" "PASS"
        AUTH_HEADER="Authorization: Bearer $TOKEN"
    else
        print_result "Authentication (Login)" "FAIL" "Token not found in response"
        AUTH_HEADER=""
    fi
else
    print_result "Authentication (Login)" "CRITICAL" "HTTP $HTTP_CODE - Cannot proceed without auth"
    echo "Response: $HTTP_BODY"
    AUTH_HEADER=""
fi

echo ""

# Step 4: Test POST /api/v1/log_action
echo "Step 4: Testing POST /api/v1/log_action..."
LOG_ACTION_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "${API_URL}/api/v1/log_action" \
    -H "Content-Type: application/json" \
    -H "$AUTH_HEADER" \
    -d "{
        \"agent_id\": \"${TEST_AGENT_ID}\",
        \"action\": \"test_compliance_action\",
        \"payload\": \"Test payload for compliance logging\",
        \"target_region\": \"EU\",
        \"user_id\": \"${TEST_USER_ID}\",
        \"user_notified\": true
    }" 2>&1)

HTTP_BODY=$(echo "$LOG_ACTION_RESPONSE" | head -n -1)
HTTP_CODE=$(echo "$LOG_ACTION_RESPONSE" | tail -n 1)

if check_status "$HTTP_CODE" "200"; then
    TEST_SEAL_ID=$(echo "$HTTP_BODY" | grep -oP '"seal_id"\s*:\s*"\K[^"]+' | head -1)
    TEST_TX_ID=$(echo "$HTTP_BODY" | grep -oP '"tx_id"\s*:\s*"\K[^"]+' | head -1)
    if [ -n "$TEST_SEAL_ID" ] && [ -n "$TEST_TX_ID" ]; then
        print_result "POST /api/v1/log_action" "PASS"
        echo "  Seal ID: $TEST_SEAL_ID"
        echo "  TX ID: $TEST_TX_ID"
    else
        print_result "POST /api/v1/log_action" "FAIL" "Missing seal_id or tx_id in response"
    fi
else
    print_result "POST /api/v1/log_action" "CRITICAL" "HTTP $HTTP_CODE"
    echo "Response: $HTTP_BODY"
    if [ $CRITICAL_FAILED -gt 0 ]; then
        echo ""
        echo "${RED}Critical test failed. Stopping tests.${NC}"
        exit 1
    fi
fi

echo ""

# Step 5: Test GET /api/v1/logs
echo "Step 5: Testing GET /api/v1/logs..."
LOGS_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "${API_URL}/api/v1/logs?limit=10" \
    -H "$AUTH_HEADER" 2>&1)

HTTP_BODY=$(echo "$LOGS_RESPONSE" | head -n -1)
HTTP_CODE=$(echo "$LOGS_RESPONSE" | tail -n 1)

if check_status "$HTTP_CODE" "200"; then
    LOG_COUNT=$(echo "$HTTP_BODY" | grep -oP '"total"\s*:\s*\K\d+' | head -1)
    if [ -n "$LOG_COUNT" ]; then
        print_result "GET /api/v1/logs" "PASS" "Found $LOG_COUNT logs"
    else
        print_result "GET /api/v1/logs" "PASS" "Response received (count not parsed)"
    fi
else
    print_result "GET /api/v1/logs" "CRITICAL" "HTTP $HTTP_CODE"
    echo "Response: $HTTP_BODY"
    if [ $CRITICAL_FAILED -gt 0 ]; then
        echo ""
        echo "${RED}Critical test failed. Stopping tests.${NC}"
        exit 1
    fi
fi

echo ""

# Step 6: Test GET /api/v1/download_report (if we have a seal_id)
if [ -n "$TEST_SEAL_ID" ]; then
    echo "Step 6: Testing GET /api/v1/download_report..."
    REPORT_RESPONSE=$(curl -s -w "\n%{http_code}" -X GET "${API_URL}/api/v1/download_report?seal_id=${TEST_SEAL_ID}&format=pdf" \
        -H "$AUTH_HEADER" \
        -o /tmp/veridion_report.pdf 2>&1)
    
    HTTP_CODE=$(echo "$REPORT_RESPONSE" | tail -n 1)
    
    if check_status "$HTTP_CODE" "200"; then
        if [ -f "/tmp/veridion_report.pdf" ] && [ -s "/tmp/veridion_report.pdf" ]; then
            FILE_SIZE=$(stat -f%z /tmp/veridion_report.pdf 2>/dev/null || stat -c%s /tmp/veridion_report.pdf 2>/dev/null || echo "0")
            if [ "$FILE_SIZE" -gt 0 ]; then
                print_result "GET /api/v1/download_report" "PASS" "PDF generated ($FILE_SIZE bytes)"
            else
                print_result "GET /api/v1/download_report" "FAIL" "PDF file is empty"
            fi
        else
            print_result "GET /api/v1/download_report" "FAIL" "PDF file not created"
        fi
    else
        print_result "GET /api/v1/download_report" "FAIL" "HTTP $HTTP_CODE"
    fi
else
    print_result "GET /api/v1/download_report" "SKIP" "No seal_id available from previous test"
fi

echo ""

# Step 7: Test POST /api/v1/shred_data (if we have a seal_id)
if [ -n "$TEST_SEAL_ID" ]; then
    echo "Step 7: Testing POST /api/v1/shred_data..."
    SHRED_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "${API_URL}/api/v1/shred_data" \
        -H "Content-Type: application/json" \
        -H "$AUTH_HEADER" \
        -d "{
            \"seal_id\": \"${TEST_SEAL_ID}\"
        }" 2>&1)
    
    HTTP_BODY=$(echo "$SHRED_RESPONSE" | head -n -1)
    HTTP_CODE=$(echo "$SHRED_RESPONSE" | tail -n 1)
    
    if check_status "$HTTP_CODE" "200"; then
        SHRED_STATUS=$(echo "$HTTP_BODY" | grep -oP '"status"\s*:\s*"\K[^"]+' | head -1)
        if [ "$SHRED_STATUS" = "SHREDDED" ] || [ -n "$SHRED_STATUS" ]; then
            print_result "POST /api/v1/shred_data" "PASS" "Status: $SHRED_STATUS"
        else
            print_result "POST /api/v1/shred_data" "PASS" "Response received"
        fi
    else
        print_result "POST /api/v1/shred_data" "CRITICAL" "HTTP $HTTP_CODE"
        echo "Response: $HTTP_BODY"
    fi
else
    print_result "POST /api/v1/shred_data" "SKIP" "No seal_id available from previous test"
fi

echo ""

# Final Summary
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo -e "${GREEN}Tests Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Tests Failed: $TESTS_FAILED${NC}"
echo ""

if [ $CRITICAL_FAILED -gt 0 ]; then
    echo -e "${RED}❌ CRITICAL TESTS FAILED${NC}"
    echo "System is not ready for production."
    exit 1
elif [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${YELLOW}⚠️  SOME TESTS FAILED${NC}"
    echo "Review failures above."
    exit 0
else
    echo -e "${GREEN}✅ ALL TESTS PASSED${NC}"
    echo "System is ready!"
    exit 0
fi

