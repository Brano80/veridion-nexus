import requests
import time
import random
import os

# Skúsime stiahnuť cenu BTC z verejného API
def get_btc_price():
    try:
        r = requests.get("https://api.coindesk.com/v1/bpi/currentprice.json", timeout=2)
        return r.json()["bpi"]["EUR"]["rate_float"]
    except:
        return random.randint(94000, 96000) # Fallback ak nejde net

# Opravený API URL
API_URL = "http://localhost:8080/api/v1/log_action"

# JWT Token - môžete ho nastaviť ako environment variable alebo získate priamo
# Získajte token cez: POST http://localhost:8080/api/v1/auth/login
# Príklad: $env:VERIDION_JWT_TOKEN = "your-token-here" (PowerShell)
# Alebo: export VERIDION_JWT_TOKEN="your-token-here" (Bash)
JWT_TOKEN = os.getenv("VERIDION_JWT_TOKEN")

# Ak token nie je nastavený, pokúsi sa automaticky prihlásiť
if not JWT_TOKEN:
    try:
        print("🔐 Žiadny JWT token - pokúšam sa automaticky prihlásiť...")
        login_response = requests.post(
            "http://localhost:8080/api/v1/auth/login",
            json={"username": "testuser", "password": "test123"},
            timeout=5
        )
        if login_response.status_code == 200:
            JWT_TOKEN = login_response.json()["token"]
            print(f"✅ Úspešne prihlásený! Token: {JWT_TOKEN[:50]}...")
        else:
            print(f"❌ Chyba prihlásenia: {login_response.status_code}")
            print("💡 Nastavte VERIDION_JWT_TOKEN environment variable alebo upravte credentials v skripte")
    except Exception as e:
        print(f"❌ Chyba pri získavaní tokenu: {e}")
        print("💡 Nastavte VERIDION_JWT_TOKEN environment variable")

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

        # 2. Odoslanie do Veridion Nexus s autentifikáciou
        if not JWT_TOKEN:
            print("   ❌ Chýba JWT token - preskakujem...")
            time.sleep(3)
            continue
            
        try:
            res = requests.post(API_URL, json={
                "agent_id": "python uipath_agent.py",
                "action": action,
                "payload": payload,
                "target_region": target_region
            }, headers={
                "Authorization": f"Bearer {JWT_TOKEN}",
                "Content-Type": "application/json"
            }, timeout=10)
            
            if res.status_code == 200:
                response_data = res.json()
                print(f"   ✅ VERIDION: COMPLIANT (Sealed) - Seal ID: {response_data.get('seal_id', 'N/A')[:30]}...")
            elif res.status_code == 403:
                try:
                    response_data = res.json()
                    print(f"   🛑 VERIDION: BLOCKED ({response_data.get('status', 'SOVEREIGNTY')})")
                except:
                    print("   🛑 VERIDION: BLOCKED (Sovereign Lock Active!)")
            elif res.status_code == 401:
                print("   ❌ VERIDION: UNAUTHORIZED - Token vypršal alebo je neplatný!")
                print("   💡 Získajte nový token: POST http://localhost:8080/api/v1/auth/login")
            else:
                print(f"   ⚠️  VERIDION: Neočakávaný status {res.status_code}")
                try:
                    print(f"   Response: {res.text[:100]}")
                except:
                    pass
            
        except requests.exceptions.Timeout:
            print("   ⏱️  Timeout - API neodpovedá")
        except requests.exceptions.ConnectionError:
            print("   🔌 Connection Error - Skontrolujte, či API beží na http://localhost:8080")
        except Exception as e:
            print(f"   ❌ Network Error: {e}")

        time.sleep(3)

if __name__ == "__main__":
    run_trader()

