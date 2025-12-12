# Shadow Mode Manual Test Checklist

## Pre-test Setup

- [ ] Server is running on `http://127.0.0.1:8080`
- [ ] Database is accessible
- [ ] Authentication token available (if required)
- [ ] Browser ready for frontend testing

---

## 1. Enforcement Mode Toggle API Tests

### Test 1.1: Get Current Mode
- [ ] Open API client (Postman/curl/Thunder Client)
- [ ] Send `GET /api/v1/system/enforcement-mode`
- [ ] Verify response contains `enforcement_mode` field
- [ ] Note current mode value

### Test 1.2: Switch to SHADOW Mode
- [ ] Send `POST /api/v1/system/enforcement-mode` with:
  ```json
  {
    "enforcement_mode": "SHADOW",
    "description": "Testing shadow mode"
  }
  ```
- [ ] Verify response shows `enforcement_mode: "SHADOW"`
- [ ] Verify `enabled_at` timestamp is recent
- [ ] Verify `enabled_by` contains user ID

### Test 1.3: Verify SHADOW Mode Active
- [ ] Send `GET /api/v1/system/enforcement-mode` again
- [ ] Confirm mode is now "SHADOW"

### Test 1.4: Test Invalid Mode
- [ ] Send `POST /api/v1/system/enforcement-mode` with:
  ```json
  {
    "enforcement_mode": "INVALID_MODE"
  }
  ```
- [ ] Verify 400 Bad Request response
- [ ] Verify error message mentions valid modes

---

## 2. Shadow Mode Logging Tests

### Test 2.1: Non-EU Request in Shadow Mode
- [ ] Ensure mode is SHADOW
- [ ] Send `POST /api/v1/log_action` with:
  ```json
  {
    "agent_id": "test-agent-001",
    "action": "data_transfer",
    "payload": "test data"
  }
  ```
- [ ] Verify response status is 200 OK (not blocked!)
- [ ] Verify response contains "SHADOW_MODE" in status
- [ ] Check database: `SELECT * FROM shadow_mode_logs WHERE agent_id = 'test-agent-001'`
- [ ] Verify log entry exists with `would_block = true` (if non-EU)

### Test 2.2: EU Request in Shadow Mode
- [ ] Send request with EU country target
- [ ] Verify response is 200 OK
- [ ] Check database log entry has `would_allow = true`

### Test 2.3: Multiple Requests
- [ ] Send 5 different requests with same agent_id
- [ ] Check database: all should be logged
- [ ] Verify analytics reflect all 5 requests

---

## 3. Shadow Mode Analytics API Tests

### Test 3.1: Basic Analytics
- [ ] Send `GET /api/v1/analytics/shadow-mode?days=7`
- [ ] Verify response structure:
  - [ ] `total_logs` (number)
  - [ ] `would_block_count` (number)
  - [ ] `would_allow_count` (number)
  - [ ] `block_percentage` (0-100)
  - [ ] `top_blocked_agents` (array)
  - [ ] `top_blocked_regions` (array)
  - [ ] `top_policies_applied` (array)
  - [ ] `confidence_score` (0-100)
  - [ ] `time_range` (object with start, end, days)

### Test 3.2: Analytics with Agent Filter
- [ ] Send `GET /api/v1/analytics/shadow-mode?days=7&agent_id=test-agent-001`
- [ ] Verify results are filtered to that agent only
- [ ] Verify `total_logs` matches agent's log count

### Test 3.3: Different Time Ranges
- [ ] Test with `days=7`
- [ ] Test with `days=30`
- [ ] Test with `days=90`
- [ ] Verify `time_range.days` matches query parameter
- [ ] Verify `time_range.start` and `time_range.end` are correct

### Test 3.4: Confidence Score
- [ ] With < 10 logs: verify confidence = 50%
- [ ] With 10-99 logs: verify confidence = 70%
- [ ] With 100-999 logs: verify confidence = 85%
- [ ] With >= 1000 logs: verify confidence = 95%

---

## 4. Shadow Mode Export Tests

### Test 4.1: CSV Export
- [ ] Send `GET /api/v1/analytics/shadow-mode/export?format=csv&days=7`
- [ ] Verify Content-Type: `text/csv`
- [ ] Verify Content-Disposition header contains filename
- [ ] Download and open CSV file
- [ ] Verify headers: id, agent_id, action_summary, action_type, etc.
- [ ] Verify data rows exist
- [ ] Verify CSV is properly formatted (commas, quotes)

### Test 4.2: JSON Export
- [ ] Send `GET /api/v1/analytics/shadow-mode/export?format=json&days=7`
- [ ] Verify Content-Type: `application/json`
- [ ] Verify response is valid JSON array
- [ ] Verify each object has all required fields

### Test 4.3: Export with Filters
- [ ] Export with `agent_id` filter
- [ ] Export with `would_block=true` filter
- [ ] Export with both filters
- [ ] Verify filtered results match expectations

### Test 4.4: Invalid Format
- [ ] Send `GET /api/v1/analytics/shadow-mode/export?format=invalid`
- [ ] Verify 400 Bad Request
- [ ] Verify error message mentions valid formats

---

## 5. Frontend Dashboard Tests

### Test 5.1: Dashboard Access
- [ ] Navigate to `http://localhost:3000/shadow-mode` (or your frontend URL)
- [ ] Verify page loads without errors
- [ ] Verify "Shadow Mode Analytics" heading is visible

### Test 5.2: Metrics Display
- [ ] Verify "Total Logs" card shows number
- [ ] Verify "Would Block" card shows count and percentage
- [ ] Verify "Would Allow" card shows count
- [ ] Verify "Block Rate" card shows percentage
- [ ] Verify all numbers are formatted correctly

### Test 5.3: Confidence Score Banner
- [ ] Verify confidence score banner is visible
- [ ] Verify color changes based on score:
  - [ ] Green for >= 90%
  - [ ] Yellow for 70-89%
  - [ ] Orange for < 70%
- [ ] Verify message matches score level

### Test 5.4: Time Range Selector
- [ ] Click dropdown, select "Last 30 days"
- [ ] Verify data refreshes
- [ ] Verify time range updates
- [ ] Test "Last 90 days" option

### Test 5.5: Agent Filter
- [ ] Type agent ID in filter input
- [ ] Verify results filter to that agent
- [ ] Clear filter, verify all data returns

### Test 5.6: Export Buttons
- [ ] Click "Export CSV" button
- [ ] Verify file downloads
- [ ] Open file, verify content
- [ ] Click "Export JSON" button
- [ ] Verify JSON file downloads
- [ ] Verify filters are applied to export

### Test 5.7: Top Blocked Agents Table
- [ ] Verify table displays top 10 agents
- [ ] Verify each row shows:
  - [ ] Agent ID
  - [ ] Block percentage badge
  - [ ] Would block count
  - [ ] Would allow count
  - [ ] Total count
  - [ ] Progress bar
- [ ] Verify sorting by block count (descending)

### Test 5.8: Top Blocked Regions
- [ ] Verify regions grid displays
- [ ] Verify each region shows:
  - [ ] Region name
  - [ ] Block percentage
  - [ ] Block/allow counts

### Test 5.9: Policies Applied
- [ ] Verify policies table displays
- [ ] Verify policy statistics are shown

### Test 5.10: Warning Banner
- [ ] If block_percentage > 20%, verify warning banner appears
- [ ] Verify banner has yellow/orange styling
- [ ] Verify message is actionable

### Test 5.11: Real-time Refresh
- [ ] Send new log_action request
- [ ] Wait 30 seconds (refresh interval)
- [ ] Verify dashboard updates with new data

---

## 6. Alert System Tests

### Test 6.1: Alert Triggering
- [ ] Ensure shadow mode is active
- [ ] Send request that would be blocked
- [ ] Check `user_notifications` table:
  ```sql
  SELECT * FROM user_notifications 
  WHERE notification_type = 'SHADOW_MODE_VIOLATION' 
  ORDER BY created_at DESC LIMIT 1;
  ```
- [ ] Verify alert was created
- [ ] Verify alert contains correct information

### Test 6.2: Rate Limiting
- [ ] Send 3 requests with same agent_id within 1 minute
- [ ] Check notifications table
- [ ] Verify only 1 alert was created (first one)
- [ ] Wait 6 minutes
- [ ] Send another request
- [ ] Verify new alert is created

---

## 7. Mode Switching Tests

### Test 7.1: SHADOW → ENFORCING
- [ ] Start in SHADOW mode
- [ ] Send request that would be blocked
- [ ] Verify request is allowed (200 OK)
- [ ] Switch to ENFORCING mode
- [ ] Send same request
- [ ] Verify request is blocked (403 Forbidden)

### Test 7.2: ENFORCING → SHADOW
- [ ] Start in ENFORCING mode
- [ ] Switch to SHADOW mode
- [ ] Verify requests are no longer blocked
- [ ] Verify shadow logs are created

---

## 8. Integration with GDPR Module

### Test 8.1: DPA/SCCs Registered
- [ ] Register DPA/SCCs for a country via Wizard/Settings
- [ ] Send request to that country in SHADOW mode
- [ ] Verify log shows `would_allow = true`
- [ ] Verify no alert is sent

### Test 8.2: No DPA/SCCs
- [ ] Send request to country without DPA/SCCs
- [ ] Verify log shows `would_block = true`
- [ ] Verify alert is sent

---

## 9. Database Verification

### Test 9.1: Log Entry Structure
- [ ] Query: `SELECT * FROM shadow_mode_logs LIMIT 1`
- [ ] Verify all columns present:
  - [ ] id (UUID)
  - [ ] agent_id (VARCHAR)
  - [ ] action_summary (TEXT)
  - [ ] action_type (VARCHAR)
  - [ ] payload_hash (VARCHAR)
  - [ ] target_region (VARCHAR)
  - [ ] would_block (BOOLEAN)
  - [ ] would_allow (BOOLEAN)
  - [ ] policy_applied (VARCHAR)
  - [ ] risk_level (VARCHAR)
  - [ ] detected_country (VARCHAR)
  - [ ] timestamp (TIMESTAMP)
  - [ ] created_at (TIMESTAMP)

### Test 9.2: Indexes
- [ ] Test query performance on `agent_id`
- [ ] Test query performance on `timestamp`
- [ ] Test query performance on `would_block`
- [ ] Verify queries are fast (< 100ms)

---

## 10. Error Handling

### Test 10.1: Invalid API Parameters
- [ ] Test invalid `days` parameter (negative, too large)
- [ ] Test invalid `format` parameter
- [ ] Verify appropriate error responses

### Test 10.2: Database Errors
- [ ] Simulate database connection issue
- [ ] Verify graceful error handling
- [ ] Verify user-friendly error messages

---

## Test Results

**Date:** _______________  
**Tester:** _______________  
**Environment:** _______________

**Total Tests:** 50+  
**Passed:** _______  
**Failed:** _______  
**Notes:** _______________

---

## Quick Test Commands

### Using curl (Linux/Mac):
```bash
# Get enforcement mode
curl -X GET "http://127.0.0.1:8080/api/v1/system/enforcement-mode"

# Set to SHADOW
curl -X POST "http://127.0.0.1:8080/api/v1/system/enforcement-mode" \
  -H "Content-Type: application/json" \
  -d '{"enforcement_mode":"SHADOW","description":"Testing"}'

# Get analytics
curl -X GET "http://127.0.0.1:8080/api/v1/analytics/shadow-mode?days=7"

# Export CSV
curl -X GET "http://127.0.0.1:8080/api/v1/analytics/shadow-mode/export?format=csv&days=7" \
  -o shadow_mode_export.csv
```

### Using PowerShell:
```powershell
# Get enforcement mode
Invoke-RestMethod -Uri "http://127.0.0.1:8080/api/v1/system/enforcement-mode" -Method GET

# Set to SHADOW
$body = @{enforcement_mode="SHADOW";description="Testing"} | ConvertTo-Json
Invoke-RestMethod -Uri "http://127.0.0.1:8080/api/v1/system/enforcement-mode" -Method POST -Body $body -ContentType "application/json"

# Get analytics
Invoke-RestMethod -Uri "http://127.0.0.1:8080/api/v1/analytics/shadow-mode?days=7" -Method GET

# Export CSV
Invoke-WebRequest -Uri "http://127.0.0.1:8080/api/v1/analytics/shadow-mode/export?format=csv&days=7" -OutFile "shadow_mode_export.csv"
```

---

## Success Criteria

✅ All API endpoints return expected responses  
✅ Shadow mode logs are created correctly  
✅ Analytics reflect logged data accurately  
✅ Export generates valid CSV/JSON files  
✅ Frontend dashboard displays data correctly  
✅ Alerts are sent with rate limiting  
✅ Mode switching works correctly  
✅ No errors in server logs  
✅ Performance is acceptable (< 500ms for analytics)

---

**Status:** Ready for Testing  
**Last Updated:** 2024-12-19

