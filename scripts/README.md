# MVP Verification Scripts

## verify_mvp_metrics.ps1

Verifies Phase 1 MVP Success Criteria for launch readiness.

### Prerequisites

1. Backend API running on `http://localhost:8080`
2. Database initialized and migrations applied
3. Default admin user exists (username: `admin`, password: `admin`)

### Usage

```powershell
powershell -ExecutionPolicy Bypass -File scripts/verify_mvp_metrics.ps1
```

### Tests Performed

1. **API Responsiveness**
   - Creates a policy and simulates it
   - Measures total time (create + simulate)
   - **Threshold:** < 5 seconds
   - **Metric:** Time to first policy test

2. **Rollback Speed**
   - Creates a policy, enforces it, then rolls it back
   - Measures rollback time
   - **Threshold:** < 30 seconds
   - **Metric:** Policy rollback completion time

3. **Shadow Mode**
   - Calls `/analytics/shadow-mode` endpoint
   - Verifies `confidence_score` field is present and calculated
   - **Metric:** Shadow mode confidence score

4. **Compliance Scores**
   - Calls `/reports/compliance-overview` or `/reports/monthly-summary`
   - Verifies GDPR score > 0
   - Verifies EU AI Act score > 0
   - **Metric:** Compliance scores for GDPR and EU AI Act

### Expected Output

```
╔════════════════════════════════════════════════════════════════════════╗
║     Phase 1 MVP Success Criteria Verification                        ║
╚════════════════════════════════════════════════════════════════════════╝

✅ API Responsiveness: PASS (1234.56ms < 5000ms)
✅ Rollback Speed: PASS (12.34s < 30s)
✅ Shadow Mode: PASS (confidence_score present: 85%)
✅ Compliance Scores: PASS (GDPR: 95%, AI Act: 90%)

🎉 ALL TESTS PASSED! System is ready for Phase 1 launch!
```

### Exit Codes

- `0` - All tests passed
- `1` - One or more tests failed

### Notes

- The script requires authentication. It will attempt to login with default admin credentials.
- If authentication fails, some tests may still work (depending on endpoint requirements).
- Policy created during testing may remain in the database (can be cleaned up manually).
