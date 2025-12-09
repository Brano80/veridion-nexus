# Penetration Testing Guide

This directory contains automated penetration testing scripts for Veridion Nexus.

## Prerequisites

1. **Python 3.7+** installed
2. **Required Python packages:**
   ```bash
   pip install pyjwt requests
   ```

3. **Target system running:**
   - Backend API should be accessible
   - Default: `http://localhost:8080`

## Test Scripts

### 1. JWT Secret Exploitation (`penetration_test_jwt.py`)
Tests for:
- Default JWT secret vulnerability (CVE-2025-VN-001)
- Token manipulation attacks
- Role escalation attempts
- Token expiration bypass

**Usage:**
```bash
python3 tests/penetration_test_jwt.py [BASE_URL]
```

**Example:**
```bash
python3 tests/penetration_test_jwt.py http://localhost:8080
```

---

### 2. SQL Injection Test (`penetration_test_sql.py`)
Tests for:
- SQL injection vulnerabilities (CVE-2025-VN-002)
- Parameter pollution
- Time-based blind SQL injection
- Error-based SQL injection

**Usage:**
```bash
python3 tests/penetration_test_sql.py [BASE_URL] [TOKEN]
```

**Example:**
```bash
python3 tests/penetration_test_sql.py http://localhost:8080
# Or with authentication token:
python3 tests/penetration_test_sql.py http://localhost:8080 "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

---

### 3. Rate Limiting Bypass (`penetration_test_rate_limit.py`)
Tests for:
- Rate limiting bypass (CVE-2025-VN-005)
- Distributed attack simulation
- Rapid request testing
- Endpoint-specific limits

**Usage:**
```bash
python3 tests/penetration_test_rate_limit.py [BASE_URL]
```

**Example:**
```bash
python3 tests/penetration_test_rate_limit.py http://localhost:8080
```

---

### 4. Comprehensive Test Suite

#### Linux/macOS:
```bash
chmod +x tests/penetration_test_all.sh
./tests/penetration_test_all.sh [BASE_URL]
```

#### Windows (PowerShell):
```powershell
.\tests\penetration_test_all.ps1 -BaseUrl "http://localhost:8080"
```

This runs all tests and generates a timestamped report.

---

## Test Results

### Expected Output

Each test script will output:
- `[!]` - Vulnerability found
- `[-]` - Test passed (no vulnerability)
- `[*]` - Informational message

### Example Output:
```
==========================================
JWT Penetration Test - Veridion Nexus
==========================================
[*] Testing for default JWT secret...
[!] CRITICAL: Default JWT secret is in use!
[+] Token accepted: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

---

## Interpreting Results

### Critical Vulnerabilities (🔴)
- **JWT Default Secret**: Immediate fix required
- **SQL Injection**: Data breach risk
- **Information Disclosure**: System structure exposed

### High-Risk Vulnerabilities (🟠)
- **Rate Limiting Bypass**: DoS risk
- **CORS Misconfiguration**: CSRF risk
- **Weak CSP**: XSS risk

### Medium-Risk Vulnerabilities (🟡)
- **Input Validation**: Potential for future exploits
- **Missing Headers**: Minor security gaps

---

## Manual Testing

### 1. Test JWT Token Manipulation
```bash
# Generate token with default secret
python3 -c "
import jwt
token = jwt.encode({
    'sub': '00000000-0000-0000-0000-000000000001',
    'username': 'admin',
    'roles': ['admin'],
    'exp': 9999999999,
    'iat': 1000000000
}, 'your-secret-key-change-in-production', algorithm='HS256')
print(token)
"

# Test token
curl -H "Authorization: Bearer <TOKEN>" http://localhost:8080/api/v1/auth/me
```

### 2. Test SQL Injection
```bash
# Test seal_id parameter
curl "http://localhost:8080/api/v1/logs?seal_id=' OR '1'='1"
```

### 3. Test Rate Limiting
```bash
# Rapid requests
for i in {1..200}; do
  curl -X POST http://localhost:8080/api/v1/auth/login \
    -H "Content-Type: application/json" \
    -d '{"username":"test","password":"test"}' &
done
```

---

## Security Best Practices

1. **Run tests in isolated environment** - Never test production systems
2. **Review results carefully** - False positives may occur
3. **Fix critical issues immediately** - Don't delay on P0 vulnerabilities
4. **Re-test after fixes** - Verify vulnerabilities are resolved
5. **Document findings** - Keep records for compliance

---

## Integration with CI/CD

Add to your CI/CD pipeline:

```yaml
# .github/workflows/penetration-test.yml
name: Penetration Tests
on: [push, pull_request]
jobs:
  pentest:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Set up Python
        uses: actions/setup-python@v2
        with:
          python-version: '3.9'
      - name: Install dependencies
        run: pip install pyjwt requests
      - name: Run penetration tests
        run: |
          python3 tests/penetration_test_jwt.py http://localhost:8080
          python3 tests/penetration_test_sql.py http://localhost:8080
          python3 tests/penetration_test_rate_limit.py http://localhost:8080
```

---

## Reporting Issues

If you find vulnerabilities:

1. **Document the issue** in `PENETRATION_TEST_REPORT.md`
2. **Create a CVE entry** (if applicable)
3. **Fix the vulnerability** following the remediation guide
4. **Re-test** to verify the fix
5. **Update the report** with fix status

---

## References

- **OWASP Top 10**: https://owasp.org/www-project-top-ten/
- **CWE Database**: https://cwe.mitre.org/
- **CVSS Calculator**: https://www.first.org/cvss/calculator/3.1

---

## Support

For questions or issues with the penetration tests:
1. Check `PENETRATION_TEST_REPORT.md` for detailed findings
2. Review the main `README.md` for system documentation
3. Check individual test script comments for implementation details

---

**Last Updated:** January 2025  
**Test Suite Version:** 1.0

