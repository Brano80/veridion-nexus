#!/usr/bin/env python3
"""
SQL Injection Test
Tests for SQL injection vulnerabilities (CVE-2025-VN-002)
"""
import requests
import sys
import json
import time

def test_sql_injection(base_url="http://localhost:8080", token=None):
    """Test for SQL injection vulnerabilities"""
    print("[*] Testing for SQL injection vulnerabilities...")
    
    headers = {}
    if token:
        headers['Authorization'] = f'Bearer {token}'
    
    # SQL injection payloads
    payloads = [
        # Basic SQL injection
        ("' OR '1'='1", "Basic OR injection"),
        ("' OR '1'='1' --", "OR injection with comment"),
        ("' UNION SELECT NULL, NULL, NULL --", "UNION injection"),
        ("'; DROP TABLE users; --", "DROP table injection"),
        ("1' OR '1'='1", "Numeric OR injection"),
        ("admin'--", "Comment injection"),
        ("' OR 1=1#", "Hash comment injection"),
        ("' OR 'x'='x", "String comparison injection"),
        ("') OR ('1'='1", "Parentheses injection"),
        ("1' AND '1'='1", "AND injection"),
        
        # Time-based blind SQL injection
        ("'; WAITFOR DELAY '00:00:05' --", "Time-based injection (SQL Server)"),
        ("'; SELECT pg_sleep(5) --", "Time-based injection (PostgreSQL)"),
        ("'; SELECT SLEEP(5) --", "Time-based injection (MySQL)"),
        
        # Boolean-based blind SQL injection
        ("' AND 1=1 --", "Boolean true injection"),
        ("' AND 1=2 --", "Boolean false injection"),
        
        # Error-based SQL injection
        ("' AND EXTRACTVALUE(1, CONCAT(0x7e, (SELECT version()), 0x7e)) --", "Error-based injection"),
    ]
    
    vulnerable_endpoints = []
    
    # Test endpoints
    endpoints = [
        ("/api/v1/logs", "GET", {"seal_id": None}),
        ("/api/v1/logs", "GET", {"agent_id": None}),
        ("/api/v1/data_subject/{user_id}/access", "GET", {"user_id": None}),
        ("/api/v1/download_report", "GET", {"seal_id": None}),
    ]
    
    for endpoint_path, method, params in endpoints:
        print(f"\n[*] Testing endpoint: {endpoint_path}")
        
        for payload, description in payloads:
            # Replace {user_id} placeholder
            test_endpoint = endpoint_path
            if "{user_id}" in test_endpoint:
                test_endpoint = test_endpoint.replace("{user_id}", payload)
            
            # Build request
            if method == "GET":
                # Add payload as query parameter
                test_url = f"{base_url}{test_endpoint}"
                if "seal_id" in params:
                    test_url += f"?seal_id={payload}"
                elif "agent_id" in params:
                    test_url += f"?agent_id={payload}"
                
                try:
                    start_time = time.time()
                    response = requests.get(test_url, headers=headers, timeout=10)
                    elapsed = time.time() - start_time
                    
                    # Check for SQL errors
                    response_text = response.text.lower()
                    sql_errors = [
                        'sql syntax',
                        'mysql error',
                        'postgresql error',
                        'sqlite error',
                        'ora-',
                        'sqlstate',
                        'syntax error',
                        'unclosed quotation',
                        'quoted string',
                        'sql command',
                    ]
                    
                    if any(error in response_text for error in sql_errors):
                        print(f"  [!] POTENTIAL SQL INJECTION: {description}")
                        print(f"      Payload: {payload}")
                        print(f"      Response: {response.text[:200]}")
                        vulnerable_endpoints.append((endpoint_path, payload, description))
                    
                    # Check for time-based injection (delayed response)
                    if elapsed > 4 and "sleep" in payload.lower() or "waitfor" in payload.lower():
                        print(f"  [!] POTENTIAL TIME-BASED INJECTION: {description}")
                        print(f"      Payload: {payload}")
                        print(f"      Response time: {elapsed:.2f}s")
                        vulnerable_endpoints.append((endpoint_path, payload, description))
                    
                except requests.exceptions.Timeout:
                    if "sleep" in payload.lower() or "waitfor" in payload.lower():
                        print(f"  [!] POTENTIAL TIME-BASED INJECTION (timeout): {description}")
                        vulnerable_endpoints.append((endpoint_path, payload, description))
                except Exception as e:
                    print(f"  [-] Error testing {payload}: {e}")
    
    return vulnerable_endpoints

def test_parameter_pollution(base_url="http://localhost:8080", token=None):
    """Test HTTP parameter pollution"""
    print("\n[*] Testing HTTP parameter pollution...")
    
    headers = {}
    if token:
        headers['Authorization'] = f'Bearer {token}'
    
    # Test duplicate parameters
    test_url = f"{base_url}/api/v1/logs?seal_id=test&seal_id=' OR '1'='1"
    
    try:
        response = requests.get(test_url, headers=headers, timeout=5)
        if response.status_code == 200:
            print("  [!] Parameter pollution may be possible")
            return True
    except Exception as e:
        print(f"  [-] Parameter pollution test failed: {e}")
    
    return False

if __name__ == '__main__':
    base_url = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:8080"
    token = sys.argv[2] if len(sys.argv) > 2 else None
    
    print("=" * 60)
    print("SQL Injection Penetration Test - Veridion Nexus")
    print("=" * 60)
    
    vulnerable = test_sql_injection(base_url, token)
    param_pollution = test_parameter_pollution(base_url, token)
    
    print("\n" + "=" * 60)
    print("Test Summary:")
    print(f"  Vulnerable Endpoints Found: {len(vulnerable)}")
    if vulnerable:
        print("\n  Vulnerabilities:")
        for endpoint, payload, desc in vulnerable:
            print(f"    - {endpoint}: {desc}")
            print(f"      Payload: {payload}")
    print(f"  Parameter Pollution: {'Possible' if param_pollution else 'Not detected'}")
    print("=" * 60)

