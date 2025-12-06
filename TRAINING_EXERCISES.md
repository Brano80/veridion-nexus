# Veridion Nexus - Practical Exercises
## Hands-On Training Scenarios

This document contains practical exercises to master Veridion Nexus operations.

---

## Exercise 1: Basic Setup & Verification

### Objective
Set up Veridion Nexus and verify it's working correctly.

### Steps

1. **Start the platform:**
```bash
docker-compose up --build
```

2. **Verify health:**
```bash
curl http://localhost:8080/health
```

Expected: `{"status":"healthy","service":"veridion-nexus","version":"1.0.0"}`

3. **Check Swagger UI:**
Open: `http://localhost:8080/swagger-ui/`

4. **Verify database:**
```bash
docker-compose exec db psql -U veridion -d veridion_nexus -c "\dt"
```

Expected: List of tables (compliance_records, users, etc.)

### Success Criteria
- ✅ Health check returns "healthy"
- ✅ Swagger UI loads
- ✅ Database has tables

---

## Exercise 2: Sovereign Lock Demonstration

### Objective
Demonstrate how Sovereign Lock blocks non-EU regions.

### Steps

1. **Try to log action with US region (should fail):**
```bash
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "agent_id": "test-agent",
    "action_type": "credit_scoring",
    "payload": {"user_id": "123", "score": 750},
    "target_region": "us-east-1"
  }'
```

Expected: Error `SOVEREIGN_LOCK_VIOLATION`

2. **Try with EU region (should succeed):**
```bash
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "agent_id": "test-agent",
    "action_type": "credit_scoring",
    "payload": {"user_id": "123", "score": 750},
    "target_region": "eu-west-1"
  }'
```

Expected: Success with `seal_id` and `tx_id`

### Success Criteria
- ✅ US region is blocked
- ✅ EU region is allowed
- ✅ Response includes seal_id

---

## Exercise 3: Complete Compliance Workflow

### Objective
Execute a complete compliance workflow from logging to reporting.

### Steps

1. **Log multiple actions:**
```bash
# Action 1
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "agent_id": "credit-scoring-v1",
    "action_type": "credit_scoring",
    "payload": {"user_id": "user_001", "score": 720, "decision": "approved"},
    "target_region": "eu-west-1",
    "inference_time_ms": 150,
    "cpu_power_rating_watts": 50
  }'

# Action 2
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "agent_id": "credit-scoring-v1",
    "action_type": "credit_scoring",
    "payload": {"user_id": "user_002", "score": 650, "decision": "rejected"},
    "target_region": "eu-west-1",
    "inference_time_ms": 120,
    "cpu_power_rating_watts": 45
  }'
```

Save the `seal_id` values from responses.

2. **View compliance logs:**
```bash
curl http://localhost:8080/api/v1/logs?agent_id=credit-scoring-v1 \
  -H "X-API-Key: test-key"
```

3. **Generate Annex IV report:**
```bash
curl http://localhost:8080/api/v1/download_report?agent_id=credit-scoring-v1 \
  -H "X-API-Key: test-key" \
  --output credit_scoring_report.pdf
```

4. **Execute Right to be Forgotten:**
```bash
curl -X POST http://localhost:8080/api/v1/shred_data \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "seal_id": "seal_id_from_step_1"
  }'
```

5. **Verify data is shredded:**
```bash
curl http://localhost:8080/api/v1/logs?seal_id=seal_id_from_step_1 \
  -H "X-API-Key: test-key"
```

Expected: Record shows `status: "ERASED (Art. 17)"`

### Success Criteria
- ✅ Actions logged successfully
- ✅ Logs are viewable
- ✅ PDF report generated
- ✅ Data can be shredded
- ✅ Shredded data shows correct status

---

## Exercise 4: Module Management

### Objective
Learn how to enable/disable operational modules.

### Steps

1. **List all modules:**
```bash
curl http://localhost:8080/api/v1/modules \
  -H "X-API-Key: test-key"
```

2. **Enable Human Oversight module:**
```bash
curl -X POST http://localhost:8080/api/v1/modules/human_oversight/enable \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{}'
```

3. **Check module status:**
```bash
curl http://localhost:8080/api/v1/modules/human_oversight/status \
  -H "X-API-Key: test-key"
```

Expected: `{"enabled": true}`

4. **Disable module:**
```bash
curl -X POST http://localhost:8080/api/v1/modules/human_oversight/disable \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{}'
```

### Success Criteria
- ✅ Can list modules
- ✅ Can enable module
- ✅ Can check status
- ✅ Can disable module

---

## Exercise 5: Human Oversight Workflow

### Objective
Practice the human oversight approval workflow.

### Prerequisites
- Human Oversight module enabled (Exercise 4)

### Steps

1. **Log an action:**
```bash
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "agent_id": "fraud-detection-v1",
    "action_type": "fraud_detection",
    "payload": {"transaction_id": "tx_001", "amount": 10000, "risk_score": 9.5},
    "target_region": "eu-west-1"
  }'
```

Save the `seal_id`.

2. **Require approval:**
```bash
curl -X POST http://localhost:8080/api/v1/action/{seal_id}/require_approval \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "reason": "High-risk transaction detected",
    "reviewer_role": "compliance_officer"
  }'
```

3. **Approve the action:**
```bash
curl -X POST http://localhost:8080/api/v1/action/{seal_id}/approve \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "reviewer_id": "reviewer_001",
    "notes": "Approved after manual review"
  }'
```

4. **Alternative: Reject the action:**
```bash
# (Instead of approve, you can reject)
curl -X POST http://localhost:8080/api/v1/action/{seal_id}/reject \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "reviewer_id": "reviewer_001",
    "reason": "Risk score too high"
  }'
```

### Success Criteria
- ✅ Can require approval
- ✅ Can approve action
- ✅ Can reject action
- ✅ Approval/rejection is recorded

---

## Exercise 6: Risk Assessment

### Objective
Practice risk assessment functionality.

### Steps

1. **Log a high-risk action:**
```bash
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "agent_id": "loan-approval-v1",
    "action_type": "loan_approval",
    "payload": {"user_id": "user_003", "loan_amount": 100000, "risk_score": 8.5},
    "target_region": "eu-west-1"
  }'
```

Save the `seal_id`.

2. **Get risk assessment:**
```bash
curl http://localhost:8080/api/v1/risk_assessment/{seal_id} \
  -H "X-API-Key: test-key"
```

3. **List all high-risk assessments:**
```bash
curl http://localhost:8080/api/v1/risks?risk_level=high \
  -H "X-API-Key: test-key"
```

### Success Criteria
- ✅ Risk assessment is generated
- ✅ Risk level is identified
- ✅ Can filter by risk level

---

## Exercise 7: Data Breach Reporting

### Objective
Practice data breach reporting workflow.

### Steps

1. **Report a breach:**
```bash
curl -X POST http://localhost:8080/api/v1/breach_report \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "breach_type": "unauthorized_access",
    "description": "Unauthorized access to user database detected",
    "affected_users": 150,
    "discovery_time": "2025-01-15T14:30:00Z",
    "severity": "high"
  }'
```

Save the `breach_id`.

2. **List all breaches:**
```bash
curl http://localhost:8080/api/v1/breaches \
  -H "X-API-Key: test-key"
```

3. **View specific breach:**
```bash
curl http://localhost:8080/api/v1/breaches/{breach_id} \
  -H "X-API-Key: test-key"
```

### Success Criteria
- ✅ Breach can be reported
- ✅ Breach appears in list
- ✅ Notification deadline is calculated (72 hours)

---

## Exercise 8: Webhook Configuration

### Objective
Set up and test webhook notifications.

### Steps

1. **Create a test webhook endpoint** (use webhook.site or similar):
- Go to https://webhook.site
- Copy your unique webhook URL

2. **Register webhook:**
```bash
curl -X POST http://localhost:8080/api/v1/webhooks \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "url": "https://webhook.site/your-unique-id",
    "events": ["action_logged", "breach_reported"],
    "secret": "my-webhook-secret"
  }'
```

Save the `webhook_id`.

3. **Trigger an event (log action):**
```bash
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "agent_id": "test-agent",
    "action_type": "test_action",
    "payload": {"test": "data"},
    "target_region": "eu-west-1"
  }'
```

4. **Check webhook delivery:**
- Go to webhook.site and see if request was received
- Or check delivery history:

```bash
curl http://localhost:8080/api/v1/webhooks/{webhook_id}/deliveries \
  -H "X-API-Key: test-key"
```

5. **List all webhooks:**
```bash
curl http://localhost:8080/api/v1/webhooks \
  -H "X-API-Key: test-key"
```

### Success Criteria
- ✅ Webhook is registered
- ✅ Event triggers webhook delivery
- ✅ Delivery history is tracked
- ✅ Can verify webhook signature

---

## Exercise 9: API Key Management

### Objective
Practice API key creation and management.

### Prerequisites
- JWT authentication (need to login first)

### Steps

1. **Login to get JWT token:**
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "your-password"
  }'
```

Save the `token`.

2. **Create API key:**
```bash
curl -X POST http://localhost:8080/api/v1/api_keys \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {token}" \
  -d '{
    "name": "production-api-key",
    "expires_at": "2026-01-15T00:00:00Z"
  }'
```

**⚠️ IMPORTANT: Save the API key immediately - it's only shown once!**

3. **List all API keys:**
```bash
curl http://localhost:8080/api/v1/api_keys \
  -H "Authorization: Bearer {token}"
```

4. **Get API key details:**
```bash
curl http://localhost:8080/api/v1/api_keys/{api_key_id} \
  -H "Authorization: Bearer {token}"
```

5. **Test API key:**
```bash
curl http://localhost:8080/api/v1/logs \
  -H "X-API-Key: {api_key_from_step_2}"
```

6. **Revoke API key:**
```bash
curl -X POST http://localhost:8080/api/v1/api_keys/{api_key_id}/revoke \
  -H "Authorization: Bearer {token}" \
  -d '{}'
```

### Success Criteria
- ✅ Can create API key
- ✅ API key works for authentication
- ✅ Can list API keys
- ✅ Can revoke API key

---

## Exercise 10: Dashboard Operations

### Objective
Master dashboard navigation and operations.

### Steps

1. **Start dashboard:**
```bash
cd dashboard
npm run dev
```

2. **Open dashboard:**
Navigate to: `http://localhost:3000`

3. **Complete dashboard tasks:**
   - View Compliance Overview
   - Navigate to Runtime Logs
   - Filter logs by agent_id
   - Generate Annex IV report
   - View module status
   - Create API key via UI
   - Configure webhook via UI

4. **Test module management:**
   - Enable a module
   - Disable a module
   - Verify module appears/disappears in navigation

### Success Criteria
- ✅ Dashboard loads correctly
- ✅ Can navigate all pages
- ✅ Can perform operations via UI
- ✅ Module management works

---

## Exercise 11: Python SDK Integration

### Objective
Integrate Veridion Nexus with Python application.

### Prerequisites
- Python 3.8+
- Veridion Nexus API running

### Steps

1. **Install SDK:**
```bash
pip install veridion-nexus-sdks[langchain]
```

2. **Create test script:**
```python
# test_veridion.py
from sdks.langchain import wrap_langchain_llm
from langchain.llms import OpenAI

# Create LangChain LLM
llm = OpenAI(temperature=0.7)

# Wrap with Veridion compliance
veridion_llm = wrap_langchain_llm(
    llm=llm,
    veridion_api_url="http://localhost:8080",
    veridion_api_key="your-api-key",
    agent_id="my-python-agent"
)

# Use normally - compliance is automatic
response = veridion_llm("What is GDPR?")
print(response)
```

3. **Run script:**
```bash
python test_veridion.py
```

4. **Verify compliance logging:**
```bash
curl http://localhost:8080/api/v1/logs?agent_id=my-python-agent \
  -H "X-API-Key: your-api-key"
```

### Success Criteria
- ✅ SDK installs correctly
- ✅ Script runs without errors
- ✅ Compliance records are created
- ✅ Can view logs in API

---

## Exercise 12: Complete Customer Demo Scenario

### Objective
Practice a complete customer presentation scenario.

### Scenario: Fintech Startup Demo

**Setup:**
1. Start Veridion Nexus
2. Start Dashboard
3. Have Swagger UI ready

**Demo Flow (15 minutes):**

1. **Introduction (2 min)**
   - Explain EU AI Act requirements
   - Show problem: Manual compliance is expensive

2. **Live Demo: Sovereign Lock (3 min)**
   - Show blocking non-EU region
   - Show allowing EU region
   - Explain technical guarantee

3. **Live Demo: Compliance Logging (3 min)**
   - Log credit scoring action
   - Show dashboard with logs
   - Show Annex IV PDF generation

4. **Live Demo: Right to be Forgotten (2 min)**
   - Show data shredding
   - Verify data is unreadable
   - Explain GDPR compliance

5. **SDK Integration (3 min)**
   - Show Python SDK example
   - Explain automatic compliance
   - Show how easy integration is

6. **Pricing Discussion (2 min)**
   - Starter tier: €35K/year
   - What's included
   - Value proposition

### Success Criteria
- ✅ Demo flows smoothly
- ✅ All features work
- ✅ Can answer questions
- ✅ Pricing is clear

---

## Exercise 13: Troubleshooting Practice

### Objective
Practice diagnosing and fixing common issues.

### Scenarios

**Scenario 1: API Returns 500 Error**
- Check logs: `docker-compose logs api`
- Verify database connection
- Check environment variables
- Fix the issue

**Scenario 2: Database Connection Failed**
- Check if PostgreSQL is running
- Verify DATABASE_URL
- Check database logs
- Fix connection

**Scenario 3: Webhook Not Delivering**
- Check webhook URL is accessible
- Verify webhook secret
- Check delivery history
- Fix configuration

**Scenario 4: Dashboard Not Loading**
- Check API is running
- Verify CORS configuration
- Check browser console
- Fix the issue

### Success Criteria
- ✅ Can diagnose issues
- ✅ Can fix problems
- ✅ Understand log analysis
- ✅ Know where to find help

---

## Exercise 14: Performance Testing

### Objective
Test platform performance under load.

### Steps

1. **Test API response time:**
```bash
time curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{"agent_id":"test","action_type":"test","payload":{},"target_region":"eu-west-1"}'
```

2. **Load test (install Apache Bench first):**
```bash
# Install: apt-get install apache2-utils (Linux) or brew install httpd (macOS)

ab -n 100 -c 10 -H "X-API-Key: test-key" \
  -p test_payload.json -T application/json \
  http://localhost:8080/api/v1/log_action
```

3. **Monitor database performance:**
```sql
-- Check slow queries
SELECT * FROM pg_stat_statements 
ORDER BY total_time DESC 
LIMIT 10;
```

4. **Check connection pool:**
```sql
-- Check active connections
SELECT count(*) FROM pg_stat_activity;
```

### Success Criteria
- ✅ API responds quickly (<100ms)
- ✅ Can handle concurrent requests
- ✅ Database performance is good
- ✅ No memory leaks

---

## Exercise 15: Security Audit

### Objective
Practice security best practices.

### Steps

1. **Check API key security:**
   - Verify keys are not in code
   - Check keys are rotated
   - Verify unused keys are revoked

2. **Check JWT configuration:**
   - Verify JWT_SECRET is strong (32+ chars)
   - Check token expiration
   - Verify tokens are not exposed

3. **Check CORS configuration:**
   - Verify ALLOWED_ORIGINS is set (not *)
   - Check production vs development

4. **Check database security:**
   - Verify strong passwords
   - Check SSL is enabled
   - Verify firewall rules

5. **Check audit logging:**
```sql
SELECT * FROM security_audit_logs 
ORDER BY created_at DESC 
LIMIT 100;
```

### Success Criteria
- ✅ All security checks pass
- ✅ No credentials in code
- ✅ CORS is properly configured
- ✅ Audit logs are working

---

## Final Assessment

### Complete All Exercises

Rate yourself on each exercise:
- ✅ **Mastered**: Can do without reference
- ⚠️ **Familiar**: Can do with reference
- ❌ **Needs Practice**: Need more time

### Customer Presentation Readiness

**Can you:**
- ✅ Set up Veridion Nexus from scratch?
- ✅ Demonstrate all core features?
- ✅ Explain the value proposition?
- ✅ Handle customer questions?
- ✅ Troubleshoot common issues?
- ✅ Present pricing confidently?

**If yes to all, you're ready for customer presentations! 🎉**

---

**Good luck with your training!**

