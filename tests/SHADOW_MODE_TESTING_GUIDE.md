# Shadow Mode Testing Guide

## Overview

This guide provides comprehensive instructions for testing Shadow Mode functionality. Shadow Mode is a critical feature that allows testing policies without affecting production traffic.

## Quick Start

### 1. Prerequisites

- Server running on `http://127.0.0.1:8080`
- Database accessible
- (Optional) Authentication token if required

### 2. Quick Test (Recommended First)

Run the simple test script to verify basic functionality:

**Windows PowerShell:**
```powershell
.\tests\test_shadow_mode_simple.ps1
```

This will:
- ✅ Check current enforcement mode
- ✅ Switch to SHADOW mode
- ✅ Send a test request
- ✅ Get analytics
- ✅ Test export
- ✅ Reset to ENFORCING mode

---

## Complete Test Suite

### Automated Tests

**PowerShell (Windows):**
```powershell
.\tests\shadow_mode_test.ps1
```

**Bash (Linux/Mac):**
```bash
chmod +x tests/shadow_mode_test.sh
./tests/shadow_mode_test.sh
```

**Note:** Set `AUTH_TOKEN` environment variable if authentication is required:
```powershell
$env:AUTH_TOKEN = "your-token-here"
```

### Manual Testing

For comprehensive manual testing, follow the [Manual Test Checklist](SHADOW_MODE_MANUAL_TEST_CHECKLIST.md).

---

## Test Scenarios

### Scenario 1: Basic Shadow Mode Flow

1. **Set Mode to SHADOW**
   ```powershell
   $body = @{enforcement_mode="SHADOW";description="Testing"} | ConvertTo-Json
   Invoke-RestMethod -Uri "http://127.0.0.1:8080/api/v1/system/enforcement-mode" -Method POST -Body $body -ContentType "application/json"
   ```

2. **Send Test Request**
   ```powershell
   $logBody = @{agent_id="test-001";action="test";payload="data"} | ConvertTo-Json
   Invoke-RestMethod -Uri "http://127.0.0.1:8080/api/v1/log_action" -Method POST -Body $logBody -ContentType "application/json"
   ```
   - ✅ Should return 200 OK (not blocked)
   - ✅ Status should contain "SHADOW_MODE"

3. **Check Analytics**
   ```powershell
   Invoke-RestMethod -Uri "http://127.0.0.1:8080/api/v1/analytics/shadow-mode?days=7" -Method GET
   ```
   - ✅ Should show your test request in analytics

4. **Export Logs**
   ```powershell
   Invoke-WebRequest -Uri "http://127.0.0.1:8080/api/v1/analytics/shadow-mode/export?format=csv&days=7" -OutFile "shadow_export.csv"
   ```
   - ✅ CSV file should contain your test request

### Scenario 2: Mode Switching

1. **Start in ENFORCING Mode**
   - Send request that violates policy
   - ✅ Should be blocked (403 Forbidden)

2. **Switch to SHADOW Mode**
   - Send same request
   - ✅ Should be allowed (200 OK)
   - ✅ Should be logged to shadow_mode_logs

3. **Switch Back to ENFORCING**
   - Send same request again
   - ✅ Should be blocked (403 Forbidden)

### Scenario 3: GDPR Integration

1. **Register DPA/SCCs** (via Wizard/Settings)
   - Register transfer agreement for a country

2. **Test in Shadow Mode**
   - Send request to that country
   - ✅ Log should show `would_allow = true`
   - ✅ No alert should be sent

3. **Test Without DPA/SCCs**
   - Send request to country without agreement
   - ✅ Log should show `would_block = true`
   - ✅ Alert should be sent (rate limited)

---

## API Endpoints to Test

### 1. Enforcement Mode

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/system/enforcement-mode` | Get current mode |
| POST | `/api/v1/system/enforcement-mode` | Set mode (SHADOW/DRY_RUN/ENFORCING) |

### 2. Analytics

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/analytics/shadow-mode?days=7` | Get analytics |
| GET | `/api/v1/analytics/shadow-mode?days=7&agent_id=xxx` | Filtered analytics |

### 3. Export

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/analytics/shadow-mode/export?format=csv&days=7` | CSV export |
| GET | `/api/v1/analytics/shadow-mode/export?format=json&days=7` | JSON export |
| GET | `/api/v1/analytics/shadow-mode/export?format=csv&days=7&would_block=true` | Filtered export |

---

## Frontend Testing

### Dashboard Access

1. Navigate to `http://localhost:3000/shadow-mode` (or your frontend URL)
2. Verify page loads without errors

### Key Features to Test

- [ ] **Metrics Cards** - Total logs, would block, would allow, block rate
- [ ] **Confidence Score** - Displays correctly with color coding
- [ ] **Time Range Selector** - 7/30/90 days options work
- [ ] **Agent Filter** - Filters results correctly
- [ ] **Export Buttons** - CSV and JSON downloads work
- [ ] **Top Blocked Agents** - Table displays correctly
- [ ] **Top Blocked Regions** - Grid displays correctly
- [ ] **Policies Applied** - Statistics shown
- [ ] **Warning Banner** - Appears when block rate > 20%
- [ ] **Real-time Refresh** - Updates every 30 seconds

---

## Database Verification

### Check Shadow Mode Logs

```sql
-- View recent shadow mode logs
SELECT * FROM shadow_mode_logs 
ORDER BY timestamp DESC 
LIMIT 10;

-- Count logs by would_block status
SELECT 
    would_block,
    COUNT(*) as count
FROM shadow_mode_logs
WHERE timestamp > NOW() - INTERVAL '7 days'
GROUP BY would_block;

-- Check specific agent
SELECT * FROM shadow_mode_logs 
WHERE agent_id = 'test-agent-001'
ORDER BY timestamp DESC;
```

### Check Enforcement Mode

```sql
-- View current enforcement mode
SELECT * FROM system_enforcement_mode 
ORDER BY enabled_at DESC 
LIMIT 1;

-- View mode history
SELECT * FROM system_enforcement_mode 
ORDER BY enabled_at DESC;
```

### Check Alerts

```sql
-- View shadow mode alerts
SELECT * FROM user_notifications 
WHERE notification_type = 'SHADOW_MODE_VIOLATION'
ORDER BY created_at DESC 
LIMIT 10;
```

---

## Expected Behavior

### In SHADOW Mode

✅ **Requests are NOT blocked** - Even if they violate policies  
✅ **All requests are logged** - To `shadow_mode_logs` table  
✅ **Analytics are generated** - Real-time metrics available  
✅ **Alerts are sent** - For violations (rate limited)  
✅ **Export works** - CSV/JSON export available  

### In ENFORCING Mode

✅ **Violations are blocked** - Returns 403 Forbidden  
✅ **Compliant requests allowed** - Returns 200 OK  
✅ **Shadow logs not created** - Only compliance_records  

---

## Troubleshooting

### Issue: Analytics returns 0 logs

**Solution:**
- Ensure shadow mode is active
- Send some test requests via `/log_action`
- Wait a few seconds for database to update
- Check database directly: `SELECT COUNT(*) FROM shadow_mode_logs`

### Issue: Export returns empty file

**Solution:**
- Check time range (days parameter)
- Verify logs exist in database
- Try without filters first
- Check server logs for errors

### Issue: Dashboard shows no data

**Solution:**
- Check browser console for errors
- Verify API is accessible
- Check network tab for failed requests
- Ensure authentication token is valid

### Issue: Alerts not being sent

**Solution:**
- Check notification service configuration
- Verify rate limiting (max 1 per 5 min per agent)
- Check `user_notifications` table
- Review server logs for errors

---

## Performance Benchmarks

Expected performance metrics:

- **Analytics API:** < 500ms for 7 days of data
- **Export API:** < 1s for 1000 records
- **Dashboard Load:** < 2s initial load
- **Real-time Refresh:** < 200ms per update

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
✅ Performance is acceptable  

---

## Test Results Template

```
Date: _______________
Tester: _______________
Environment: _______________

API Tests:
- Enforcement Mode Toggle: [ ] PASS [ ] FAIL
- Shadow Mode Logging: [ ] PASS [ ] FAIL
- Analytics API: [ ] PASS [ ] FAIL
- Export API: [ ] PASS [ ] FAIL

Frontend Tests:
- Dashboard Load: [ ] PASS [ ] FAIL
- Metrics Display: [ ] PASS [ ] FAIL
- Export Buttons: [ ] PASS [ ] FAIL

Integration Tests:
- Mode Switching: [ ] PASS [ ] FAIL
- GDPR Integration: [ ] PASS [ ] FAIL
- Alert System: [ ] PASS [ ] FAIL

Notes:
_______________________________________
_______________________________________
```

---

## Next Steps

After completing tests:

1. ✅ Review test results
2. ✅ Document any issues found
3. ✅ Verify all critical paths work
4. ✅ Check performance metrics
5. ✅ Update test documentation if needed

---

**Last Updated:** 2024-12-19  
**Status:** Ready for Testing

