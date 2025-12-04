import requests
import time
import random

# Skúsime stiahnuť cenu BTC z verejného API
def get_btc_price():
    try:
        r = requests.get("https://api.coindesk.com/v1/bpi/currentprice.json", timeout=2)
        return r.json()["bpi"]["EUR"]["rate_float"]
    except:
        return random.randint(94000, 96000) # Fallback ak nejde net

API_URL = "http://localhost:8080/log_action"

ACTIONS = [
    ("Credit Check - Client EU", "EU"),
    ("GDPR Audit Scan", "EU"),
    ("UPLOAD DATA TO AWS US-EAST", "US"), # Toto spustí červený poplach
    ("SEND ANALYTICS TO GOOGLE NY", "US") # Toto tiež
]

def run_trader():
    print("📈 Starting Veridion High-Frequency Trader (BTC/EUR)...")
    
    while True:
        price = get_btc_price()
        
        # 1. Rozhodovanie (Logika Agenta)
        # Zvýšime šancu na "US" akciu na 50%, aby sme to videli hneď
        action_name, target_region = random.choice(ACTIONS)
        
        if target_region == "US":
            action = action_name
            payload = f"Attempting to send data to {target_region} region"
            print(f"⚠️  Attempting dangerous action: {action} -> {target_region} Cloud...")
        else:
            action = action_name
            payload = f"Executed {action_name} - Safe EU operation"
            print(f"🤖 Action: {action}")

        # 2. Odoslanie do Veridion Nexus
        try:
            res = requests.post(API_URL, json={
                "agent_id": "python uipath_agent.py",
                "action": action,
                "payload": payload,
                "target_region": target_region
            })
            
            if res.status_code == 200:
                print("   ✅ VERIDION: COMPLIANT (Sealed)")
            elif res.status_code == 403:
                print("   🛑 VERIDION: BLOCKED (Sovereign Lock Active!)")
            
        except Exception as e:
            print("   ❌ Network Error")

        time.sleep(3)

if __name__ == "__main__":
    run_trader()

