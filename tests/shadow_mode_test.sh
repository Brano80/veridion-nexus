#!/bin/bash

# Shadow Mode Complete Test Suite
# Tests all Shadow Mode functionality

API_BASE="http://127.0.0.1:8080/api/v1"
AUTH_TOKEN="${AUTH_TOKEN:-}"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Helper function to print test results
print_test() {
    local test_name=$1
    local status=$2
    if [ "$status" == "PASS" ]; then
        echo -e "${GREEN}✓${NC} $test_name"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}✗${NC} $test_name"
        ((TESTS_FAILED++))
    fi
}

# Helper function to make API calls
api_call() {
    local method=$1
    local endpoint=$2
    local data=$3
    
    if [ -z "$AUTH_TOKEN" ]; then
        curl -s -X "$method" "$API_BASE$endpoint" \
            -H "Content-Type: application/json" \
            ${data:+-d "$data"}
    else
        curl -s -X "$method" "$API_BASE$endpoint" \
            -H "Content-Type: application/json" \
            -H "Authorization: Bearer $AUTH_TOKEN" \
            ${data:+-d "$data"}
    fi
}

echo "========================================="
echo "Shadow Mode Complete Test Suite"
echo "========================================="
echo ""

# Test 1: Get current enforcement mode
echo "Test 1: Get current enforcement mode"
RESPONSE=$(api_call "GET" "/system/enforcement-mode")
if echo "$RESPONSE" | grep -q "enforcement_mode"; then
    print_test "Get enforcement mode" "PASS"
    CURRENT_MODE=$(echo "$RESPONSE" | grep -o '"enforcement_mode":"[^"]*"' | cut -d'"' -f4)
    echo "  Current mode: $CURRENT_MODE"
else
    print_test "Get enforcement mode" "FAIL"
    echo "  Response: $RESPONSE"
fi
echo ""

# Test 2: Set enforcement mode to SHADOW
echo "Test 2: Set enforcement mode to SHADOW"
RESPONSE=$(api_call "POST" "/system/enforcement-mode" '{"enforcement_mode":"SHADOW","description":"Testing shadow mode"}')
if echo "$RESPONSE" | grep -q "SHADOW"; then
    print_test "Set enforcement mode to SHADOW" "PASS"
else
    print_test "Set enforcement mode to SHADOW" "FAIL"
    echo "  Response: $RESPONSE"
fi
echo ""

# Test 3: Verify shadow mode is active
echo "Test 3: Verify shadow mode is active"
RESPONSE=$(api_call "GET" "/system/enforcement-mode")
if echo "$RESPONSE" | grep -q '"enforcement_mode":"SHADOW"'; then
    print_test "Verify shadow mode is active" "PASS"
else
    print_test "Verify shadow mode is active" "FAIL"
    echo "  Response: $RESPONSE"
fi
echo ""

# Test 4: Send log_action request that would be blocked (non-EU country)
echo "Test 4: Send log_action request that would be blocked (non-EU country)"
TIMESTAMP=$(date +%s)
RESPONSE=$(api_call "POST" "/log_action" "{
    \"agent_id\": \"test-agent-shadow-${TIMESTAMP}\",
    \"action\": \"test_action\",
    \"payload\": \"test payload for shadow mode\"
}")
if echo "$RESPONSE" | grep -q "status"; then
    print_test "Send log_action in shadow mode" "PASS"
    # In shadow mode, should return OK even if violation
    if echo "$RESPONSE" | grep -q "SHADOW_MODE"; then
        print_test "Shadow mode correctly logs violations without blocking" "PASS"
    else
        print_test "Shadow mode correctly logs violations without blocking" "FAIL"
    fi
else
    print_test "Send log_action in shadow mode" "FAIL"
    echo "  Response: $RESPONSE"
fi
echo ""

# Test 5: Get shadow mode analytics
echo "Test 5: Get shadow mode analytics"
RESPONSE=$(api_call "GET" "/analytics/shadow-mode?days=7")
if echo "$RESPONSE" | grep -q "total_logs"; then
    print_test "Get shadow mode analytics" "PASS"
    TOTAL_LOGS=$(echo "$RESPONSE" | grep -o '"total_logs":[0-9]*' | cut -d':' -f2)
    echo "  Total logs: $TOTAL_LOGS"
else
    print_test "Get shadow mode analytics" "FAIL"
    echo "  Response: $RESPONSE"
fi
echo ""

# Test 6: Get shadow mode analytics with agent filter
echo "Test 6: Get shadow mode analytics with agent filter"
AGENT_ID="test-agent-shadow-${TIMESTAMP}"
RESPONSE=$(api_call "GET" "/analytics/shadow-mode?days=7&agent_id=$AGENT_ID")
if echo "$RESPONSE" | grep -q "total_logs"; then
    print_test "Get shadow mode analytics with agent filter" "PASS"
else
    print_test "Get shadow mode analytics with agent filter" "FAIL"
    echo "  Response: $RESPONSE"
fi
echo ""

# Test 7: Export shadow mode logs (CSV)
echo "Test 7: Export shadow mode logs (CSV)"
RESPONSE=$(api_call "GET" "/analytics/shadow-mode/export?format=csv&days=7")
if echo "$RESPONSE" | grep -q "id,agent_id"; then
    print_test "Export shadow mode logs (CSV)" "PASS"
    CSV_LINES=$(echo "$RESPONSE" | wc -l)
    echo "  CSV lines: $CSV_LINES"
else
    print_test "Export shadow mode logs (CSV)" "FAIL"
    echo "  Response: ${RESPONSE:0:200}..."
fi
echo ""

# Test 8: Export shadow mode logs (JSON)
echo "Test 8: Export shadow mode logs (JSON)"
RESPONSE=$(api_call "GET" "/analytics/shadow-mode/export?format=json&days=7")
if echo "$RESPONSE" | grep -q "\["; then
    print_test "Export shadow mode logs (JSON)" "PASS"
else
    print_test "Export shadow mode logs (JSON)" "FAIL"
    echo "  Response: ${RESPONSE:0:200}..."
fi
echo ""

# Test 9: Export with filters
echo "Test 9: Export with filters (would_block=true)"
RESPONSE=$(api_call "GET" "/analytics/shadow-mode/export?format=csv&days=7&would_block=true")
if echo "$RESPONSE" | grep -q "id,agent_id"; then
    print_test "Export with filters" "PASS"
else
    print_test "Export with filters" "FAIL"
    echo "  Response: ${RESPONSE:0:200}..."
fi
echo ""

# Test 10: Set enforcement mode back to ENFORCING
echo "Test 10: Set enforcement mode back to ENFORCING"
RESPONSE=$(api_call "POST" "/system/enforcement-mode" '{"enforcement_mode":"ENFORCING","description":"Testing complete"}')
if echo "$RESPONSE" | grep -q "ENFORCING"; then
    print_test "Set enforcement mode to ENFORCING" "PASS"
else
    print_test "Set enforcement mode to ENFORCING" "FAIL"
    echo "  Response: $RESPONSE"
fi
echo ""

# Test 11: Verify enforcing mode blocks requests
echo "Test 11: Verify enforcing mode blocks non-EU requests"
RESPONSE=$(api_call "POST" "/log_action" "{
    \"agent_id\": \"test-agent-enforcing-${TIMESTAMP}\",
    \"action\": \"test_action\",
    \"payload\": \"test payload\"
}")
# In enforcing mode, non-EU requests should be blocked (403)
if echo "$RESPONSE" | grep -q "BLOCKED\|FORBIDDEN\|403"; then
    print_test "Enforcing mode blocks violations" "PASS"
else
    print_test "Enforcing mode blocks violations" "FAIL"
    echo "  Response: $RESPONSE"
fi
echo ""

# Summary
echo "========================================="
echo "Test Summary"
echo "========================================="
echo -e "${GREEN}Tests Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Tests Failed: $TESTS_FAILED${NC}"
echo "Total Tests: $((TESTS_PASSED + TESTS_FAILED))"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}All tests passed! ✓${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed! ✗${NC}"
    exit 1
fi

