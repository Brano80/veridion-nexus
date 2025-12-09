#!/usr/bin/env python3
"""
Rate Limiting Bypass Test
Tests for rate limiting vulnerabilities (CVE-2025-VN-005)
"""
import requests
import threading
import time
import sys
from collections import defaultdict

def test_rate_limit_bypass(base_url="http://localhost:8080", endpoint="/api/v1/auth/login"):
    """Test rate limiting bypass using multiple IPs (simulated)"""
    print("[*] Testing rate limiting bypass...")
    
    # Test data
    test_data = {
        'username': 'test',
        'password': 'wrong_password'
    }
    
    success_count = 0
    rate_limited_count = 0
    error_count = 0
    
    def make_request(thread_id, request_count=100):
        nonlocal success_count, rate_limited_count, error_count
        
        for i in range(request_count):
            try:
                # Simulate different IPs by using different User-Agent headers
                headers = {
                    'User-Agent': f'TestBot-{thread_id}-{i}',
                    'X-Forwarded-For': f'192.168.1.{thread_id % 255}',
                }
                
                response = requests.post(
                    f'{base_url}{endpoint}',
                    json=test_data,
                    headers=headers,
                    timeout=5
                )
                
                if response.status_code == 200:
                    success_count += 1
                    print(f"  [!] Thread {thread_id} - Request {i} succeeded (bypassed rate limit?)")
                elif response.status_code == 429:
                    rate_limited_count += 1
                else:
                    error_count += 1
                
                time.sleep(0.05)  # Small delay between requests
                
            except Exception as e:
                error_count += 1
                print(f"  [-] Thread {thread_id} - Request {i} failed: {e}")
    
    # Launch multiple threads to simulate distributed attack
    print(f"[*] Launching 10 threads, 100 requests each...")
    threads = []
    for i in range(10):
        t = threading.Thread(target=make_request, args=(i, 100))
        threads.append(t)
        t.start()
    
    # Wait for all threads
    for t in threads:
        t.join()
    
    print(f"\n[*] Results:")
    print(f"  Successful requests: {success_count}")
    print(f"  Rate limited (429): {rate_limited_count}")
    print(f"  Errors: {error_count}")
    print(f"  Total requests: {success_count + rate_limited_count + error_count}")
    
    if success_count > 100:
        print("\n[!] VULNERABLE: Rate limiting can be bypassed with multiple IPs!")
        return True
    else:
        print("\n[-] Rate limiting appears to be working")
        return False

def test_rapid_requests(base_url="http://localhost:8080", endpoint="/api/v1/auth/login"):
    """Test rapid requests from single IP"""
    print("\n[*] Testing rapid requests from single IP...")
    
    test_data = {
        'username': 'test',
        'password': 'wrong_password'
    }
    
    success_count = 0
    rate_limited_count = 0
    
    # Make 200 rapid requests
    for i in range(200):
        try:
            response = requests.post(
                f'{base_url}{endpoint}',
                json=test_data,
                timeout=5
            )
            
            if response.status_code == 200:
                success_count += 1
            elif response.status_code == 429:
                rate_limited_count += 1
                break  # Rate limited, stop
            
        except Exception as e:
            print(f"  [-] Request {i} failed: {e}")
        
        time.sleep(0.01)  # Very small delay
    
    print(f"\n[*] Results:")
    print(f"  Successful requests before rate limit: {success_count}")
    print(f"  Rate limited at request: {success_count + rate_limited_count}")
    
    if success_count > 100:
        print("\n[!] WARNING: High number of requests allowed before rate limiting")
        return True
    else:
        print("\n[-] Rate limiting activated appropriately")
        return False

def test_endpoint_specific_limits(base_url="http://localhost:8080"):
    """Test if different endpoints have different rate limits"""
    print("\n[*] Testing endpoint-specific rate limits...")
    
    endpoints = [
        "/api/v1/auth/login",
        "/api/v1/logs",
        "/api/v1/data_subject/test/access",
    ]
    
    results = {}
    
    for endpoint in endpoints:
        print(f"\n[*] Testing {endpoint}...")
        success_count = 0
        
        for i in range(100):
            try:
                if endpoint == "/api/v1/auth/login":
                    response = requests.post(
                        f'{base_url}{endpoint}',
                        json={'username': 'test', 'password': 'test'},
                        timeout=5
                    )
                else:
                    # Would need auth token for other endpoints
                    response = requests.get(f'{base_url}{endpoint}', timeout=5)
                
                if response.status_code != 429:
                    success_count += 1
                else:
                    break
                    
            except Exception as e:
                pass
            
            time.sleep(0.05)
        
        results[endpoint] = success_count
        print(f"  Requests before rate limit: {success_count}")
    
    return results

if __name__ == '__main__':
    base_url = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:8080"
    
    print("=" * 60)
    print("Rate Limiting Penetration Test - Veridion Nexus")
    print("=" * 60)
    
    # Test 1: Distributed attack simulation
    bypass_vuln = test_rate_limit_bypass(base_url)
    
    # Test 2: Rapid requests
    rapid_vuln = test_rapid_requests(base_url)
    
    # Test 3: Endpoint-specific limits
    endpoint_results = test_endpoint_specific_limits(base_url)
    
    print("\n" + "=" * 60)
    print("Test Summary:")
    print(f"  Rate Limit Bypass Possible: {'YES' if bypass_vuln else 'NO'}")
    print(f"  Rapid Request Vulnerability: {'YES' if rapid_vuln else 'NO'}")
    print("\n  Endpoint-Specific Results:")
    for endpoint, count in endpoint_results.items():
        print(f"    {endpoint}: {count} requests before limit")
    print("=" * 60)

