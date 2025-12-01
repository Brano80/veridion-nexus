#!/usr/bin/env python3
"""
Veridion Nexus - AI Agent Simulator

This script simulates an AI agent sending actions to the Veridion Nexus API.
It continuously sends random actions every 3-5 seconds to test the compliance pipeline.
"""

import requests
import time
import random
import json
from datetime import datetime

# API endpoint
API_URL = "http://localhost:8080/log_action"

# Sample actions that an AI agent might perform
ACTIONS = [
    "Loan Approval",
    "Credit Check",
    "Fraud Alert",
    "Transaction Blocked",
    "Limit Increase",
    "Account Verification",
    "Risk Assessment",
    "Compliance Review"
]

# Agent identifier
AGENT_ID = "Python-Test-Agent"

def generate_payload(action: str) -> dict:
    """Generate a fake payload based on the action type."""
    base_payload = {
        "user_id": random.randint(1000, 9999),
        "timestamp": datetime.now().isoformat(),
    }
    
    # Add action-specific fields
    if "Loan" in action or "Credit" in action:
        base_payload["amount"] = random.randint(1000, 50000)
        base_payload["currency"] = "EUR"
        base_payload["duration_months"] = random.randint(12, 60)
    elif "Fraud" in action or "Transaction" in action:
        base_payload["transaction_id"] = f"TXN-{random.randint(100000, 999999)}"
        base_payload["risk_score"] = round(random.uniform(0.0, 1.0), 2)
    elif "Limit" in action:
        base_payload["current_limit"] = random.randint(1000, 10000)
        base_payload["new_limit"] = random.randint(10000, 50000)
    elif "Verification" in action:
        base_payload["verification_type"] = random.choice(["KYC", "AML", "Identity"])
        base_payload["status"] = random.choice(["Pending", "Verified", "Rejected"])
    elif "Risk" in action:
        base_payload["risk_level"] = random.choice(["Low", "Medium", "High"])
        base_payload["factors"] = random.randint(1, 5)
    elif "Compliance" in action:
        base_payload["regulation"] = random.choice(["GDPR", "EU AI Act", "MiFID II"])
        base_payload["check_type"] = random.choice(["Automated", "Manual Review"])
    
    return base_payload

def send_action(action: str) -> dict:
    """Send an action to the Veridion Nexus API."""
    payload = generate_payload(action)
    
    request_data = {
        "agent_id": AGENT_ID,
        "action": action,
        "payload": json.dumps(payload)
    }
    
    try:
        response = requests.post(
            API_URL,
            json=request_data,
            timeout=10
        )
        
        if response.status_code == 200:
            result = response.json()
            return {
                "success": True,
                "status_code": response.status_code,
                "status": result.get("status", "UNKNOWN"),
                "seal_id": result.get("seal_id", "N/A"),
                "tx_id": result.get("tx_id", "N/A")
            }
        elif response.status_code == 403:
            # Lockdown or blocked
            try:
                error_data = response.json()
                reason = error_data.get("reason", "Unknown reason")
                return {
                    "success": False,
                    "status_code": response.status_code,
                    "status": error_data.get("status", "BLOCKED"),
                    "reason": reason,
                    "seal_id": "N/A",
                    "tx_id": "N/A"
                }
            except:
                return {
                    "success": False,
                    "status_code": response.status_code,
                    "status": "BLOCKED",
                    "reason": "Forbidden",
                    "seal_id": "N/A",
                    "tx_id": "N/A"
                }
        else:
            return {
                "success": False,
                "status_code": response.status_code,
                "status": "ERROR",
                "reason": response.text[:100],
                "seal_id": "N/A",
                "tx_id": "N/A"
            }
            
    except requests.exceptions.ConnectionError:
        return {
            "success": False,
            "status_code": 0,
            "status": "CONNECTION_ERROR",
            "reason": "Server is not running or unreachable",
            "seal_id": "N/A",
            "tx_id": "N/A"
        }
    except requests.exceptions.Timeout:
        return {
            "success": False,
            "status_code": 0,
            "status": "TIMEOUT",
            "reason": "Request timed out",
            "seal_id": "N/A",
            "tx_id": "N/A"
        }
    except Exception as e:
        return {
            "success": False,
            "status_code": 0,
            "status": "ERROR",
            "reason": str(e),
            "seal_id": "N/A",
            "tx_id": "N/A"
        }

def print_result(action: str, result: dict):
    """Print the result in a formatted way."""
    timestamp = datetime.now().strftime("%H:%M:%S")
    
    if result["success"]:
        status_emoji = "✅"
        status_color = "COMPLIANT"
    elif result["status_code"] == 403:
        status_emoji = "⛔"
        status_color = "BLOCKED"
    else:
        status_emoji = "❌"
        status_color = result["status"]
    
    print(f"[{timestamp}] {status_emoji} Action: {action:25} | Status: {status_color:12} | Seal: {result['seal_id'][:30]}...")
    
    if not result["success"] and "reason" in result:
        print(f"         ⚠️  Reason: {result['reason']}")

def main():
    """Main loop that continuously sends actions."""
    print("=" * 80)
    print("🤖 Veridion Nexus - AI Agent Simulator")
    print("=" * 80)
    print(f"Agent ID: {AGENT_ID}")
    print(f"API URL: {API_URL}")
    print(f"Actions: {', '.join(ACTIONS)}")
    print("=" * 80)
    print("\nStarting simulation... (Press Ctrl+C to stop)\n")
    
    iteration = 0
    
    try:
        while True:
            iteration += 1
            
            # Pick a random action
            action = random.choice(ACTIONS)
            
            # Send the action
            result = send_action(action)
            
            # Print the result
            print_result(action, result)
            
            # Wait 3-5 seconds before next action
            wait_time = random.uniform(3.0, 5.0)
            time.sleep(wait_time)
            
    except KeyboardInterrupt:
        print("\n\n" + "=" * 80)
        print(f"🛑 Simulation stopped after {iteration} actions")
        print("=" * 80)

if __name__ == "__main__":
    main()

