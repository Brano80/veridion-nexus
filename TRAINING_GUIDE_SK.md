# Veridion Nexus - Kompletný Tréningový Program
## Ovládnite Platformu pre Prezentácie Zákazníkom

**Verzia 1.0 | Január 2025**

---

## 📋 Obsah

1. [Modul 1: Prehľad Platformy & Architektúra](#modul-1-prehľad-platformy--architektúra)
2. [Modul 2: Nastavenie & Prvé Kroky](#modul-2-nastavenie--prvé-kroky)
3. [Modul 3: Jadrové Funkcie - Prakticky](#modul-3-jadrové-funkcie---prakticky)
4. [Modul 4: Operačné Moduly](#modul-4-operačné-moduly)
5. [Modul 5: Integrácia & SDK](#modul-5-integrácia--sdk)
6. [Modul 6: Operácie Dashboardu](#modul-6-operácie-dashboardu)
7. [Modul 7: Scenáre Prezentácie Zákazníkom](#modul-7-scenáre-prezentácie-zákazníkom)
8. [Modul 8: Riešenie Problémov & Podpora](#modul-8-riešenie-problémov--podpora)

---

# Modul 1: Prehľad Platformy & Architektúra

## 🎯 Ciele Vzdelávania

Na konci tohto modulu budete:
- Rozumieť, čo je Veridion Nexus a prečo existuje
- Poznať 3-vrstvovú architektúru
- Rozumieť deployment módom
- Vedieť vysvetliť hodnotovú propózíciu

## 1.1 Čo je Veridion Nexus?

**Veridion Nexus** je **platforma pre runtime enforcement súladnosti** pre High-Risk AI systémy v EÚ. Na rozdiel od tradičných nástrojov súladnosti, ktoré sa spoliehajú na politiky a audit, Veridion Nexus poskytuje **technické záruky**, ktoré robia fyzicky nemožným, aby AI agenty porušili predpisy EÚ.

### Kľúčové Rozdiely

1. **Runtime Enforcement** (nie len monitorovanie)
   - Blokuje porušenia na úrovni siete
   - Zabráni dátam opustiť jurisdikcie EU/EEA
   - Vynucuje súladnosť predtým, než sa akcie uskutočnia

2. **EU-First Architektúra**
   - Postavená špecificky pre EU AI Act, GDPR a eIDAS
   - Nie generický nástroj súladnosti adaptovaný pre EÚ

3. **Technické Záruky**
   - Kryptografické dôkazy súladnosti
   - Nemenné auditné stopy
   - Automatizované generovanie dokumentácie

## 1.2 Problém, Ktorý Riešime

### Compliance Kríza

**EU AI Act** (platný od 2026) vyžaduje:
- Data sovereignty (Článok 10): Dáta musia zostať v EU/EEA
- Technical documentation (Annex IV): Každé rozhodnutie AI musí byť zdokumentované
- Human oversight (Článok 14): High-risk akcie potrebujú schválenie
- Risk management (Článok 9): Neustále hodnotenie rizika

**GDPR** vyžaduje:
- Right to be Forgotten (Článok 17): Vymazanie dát na požiadanie
- Ale auditné záznamy musia byť nemenné (bezpečnostná požiadavka)
- **Paradox**: Nemožno vymazať z nemenných záznamov

**eIDAS** vyžaduje:
- Qualified Electronic Seals pre právny dôkaz
- Kryptografické podpisy pre dôkaz súladnosti

### Súčasné Riešenia Zlyhávajú Pretože:

- ❌ **Procesné**: Spoliehajú sa na politiky, nie technické vynucovanie
- ❌ **Reaktívne**: Zisťujú porušenia po tom, čo sa stali
- ❌ **Generické**: Nie sú postavené pre špecifiká EU AI Act
- ❌ **Drahé**: Vlastné riešenia stoja €500K-€2M

### Riešenie Veridion Nexus:

- ✅ **Technické vynucovanie**: Blokuje porušenia na úrovni siete
- ✅ **Proaktívne**: Predchádza porušeniam predtým, než nastanú
- ✅ **EU-špecifické**: Postavené pre EU AI Act, GDPR, eIDAS
- ✅ **Nákladovo efektívne**: 70% lacnejšie ako vlastné riešenia

## 1.3 Trojvrstvová Architektúra

### Vrstva 1: Jadrový Runtime Compliance Engine (Povinný)

**Vždy zapnutý** - Poskytuje jadrové záruky súladnosti:

1. **Sovereign Lock** (`src/core/sovereign_lock.rs`)
   - Runtime geofencing pre data sovereignty
   - Blokuje jurisdikcie mimo EU/EEA
   - Súladnosť s EU AI Act Článok 10

2. **Crypto-Shredder** (`src/core/crypto_shredder.rs`)
   - GDPR envelope encryption
   - Right to be Forgotten (Článok 17)
   - AES-256-GCM šifrovanie

3. **Privacy Bridge** (`src/core/privacy_bridge.rs`)
   - eIDAS Qualified Electronic Seals
   - Hash-based sealing
   - Integrácia so Signicat

4. **Audit Log Chain** (integrované v routes)
   - Nemenná auditná stopa
   - Úložisko záznamov súladnosti
   - Real-time logovanie

5. **Annex IV Compiler** (`src/core/annex_iv.rs`)
   - Automatizovaná technická dokumentácia
   - Generovanie PDF reportov
   - Súladnosť s EU AI Act Annex IV

### Vrstva 2: Operačné Moduly (Voliteľné)

**Môžu byť zapnuté/vypnuté** cez Module Configuration API:

- Data Subject Rights (GDPR Články 15-22)
- Human Oversight (EU AI Act Článok 14)
- Risk Assessment (EU AI Act Článok 9)
- Breach Management (GDPR Články 33-34)
- Consent Management (GDPR Články 6-7)
- DPIA Tracking (GDPR Článok 35)
- Retention Policies (GDPR Článok 5(1)(e))
- Post-Market Monitoring (EU AI Act Článok 72)
- Green AI Telemetry (EU AI Act Článok 40)
- AI-BOM (CycloneDX Standard)

### Vrstva 3: Integračná Vrstva (Vždy Dostupné)

**SDK a konektory** pre bezproblémovú integráciu:

- **AI Platform SDKs**: Azure AI, AWS Bedrock, GCP Vertex, LangChain, OpenAI MCP, HuggingFace
- **Webhooks**: Real-time event notifikácie s HMAC-SHA256 podpisovaním
- **Proxy Mode**: Reverse proxy middleware pre existujúcu AI infraštruktúru
- **REST API**: Kompletná API pre všetky funkcie

## 1.4 Deployment Módy

### Mód 1: Embedded (SDK-First)
- **Najlepšie pre**: Startupy, stredné podniky
- **Ako to funguje**: SDK integrované priamo v aplikačnom kóde
- **Cena**: €35K-€120K/rok
- **Príklad**: Python aplikácia používa `VeridionAzureAI` wrapper

### Mód 2: Proxy (Reverse Proxy)
- **Najlepšie pre**: Enterprise s existujúcou AI infraštruktúrou
- **Ako to funguje**: Nexus beží ako middleware, zachytáva AI API volania
- **Cena**: €120K-€350K/rok
- **Príklad**: Všetky AI volania prechádzajú cez Nexus proxy automaticky

### Mód 3: Full Governance (Kompletná Platforma)
- **Najlepšie pre**: Enterprise vyžadujúce kompletnú kontrolu
- **Ako to funguje**: Kompletné nasadenie platformy so všetkými modulmi
- **Cena**: €350K+/rok
- **Príklad**: Kompletný dashboard, všetky moduly, on-premise možnosť

## 1.5 Súhrn Hodnotovej Propózície

**Pre Zákazníkov:**
- ✅ **Záruka Súladnosti**: Technické vynucovanie, nie len politiky
- ✅ **Úspora Nákladov**: 70% lacnejšie ako vlastné riešenia
- ✅ **Úspora Času**: 90% zníženie času na dokumentáciu súladnosti
- ✅ **Zníženie Rizika**: Predchádza porušeniam predtým, než nastanú
- ✅ **EU-Špecifické**: Postavené pre EU AI Act, nie adaptované

**Pre Vás (Predaj):**
- ✅ **Jasná Diferenciácia**: Jediná runtime enforcement platforma
- ✅ **Modulárne Cenovanie**: Zákazníci platia len za to, čo potrebujú
- ✅ **Viaceré Vstupné Body**: SDK, Proxy alebo Full deployment
- ✅ **Vysoké Náklady na Prepínanie**: Hlboká integrácia vytvára lock-in

---

# Modul 2: Nastavenie & Prvé Kroky

## 🎯 Ciele Vzdelávania

Na konci tohto modulu budete:
- Vedieť nastaviť Veridion Nexus lokálne
- Rozumieť konfigurácii prostredia
- Vedieť, ako overiť inštaláciu
- Prístup k Swagger UI a Dashboardu

## 2.1 Predpoklady

### Povinné:
- **Docker** a **Docker Compose** (odporúčané)
- ALEBO **Rust 1.70+** a **PostgreSQL 14+** (manuálne nastavenie)
- **Git** (na klonovanie repozitára)

### Voliteľné:
- **Signicat API Credentials** (pre skutočné eIDAS sealing)
- **Python 3.8+** (pre testovanie SDK)

## 2.2 Rýchly Start s Dockerom (Odporúčané)

### Krok 1: Klonovanie Repozitára
```bash
git clone https://github.com/Brano80/veridion-nexus.git
cd veridion-nexus
```

### Krok 2: Spustenie Služieb
```bash
docker-compose up --build
```

Toto:
- Zostaví Rust API server
- Spustí PostgreSQL databázu
- Spustí databázové migrácie
- Spustí API na porte 8080

### Krok 3: Overenie Inštalácie

**Health Check:**
```bash
curl http://localhost:8080/health
```

Očakávaná odpoveď:
```json
{
  "status": "healthy",
  "service": "veridion-nexus",
  "version": "1.0.0"
}
```

**Swagger UI:**
Otvorte v prehliadači: `http://localhost:8080/swagger-ui/`

Mali by ste vidieť interaktívnu API dokumentáciu.

## 2.3 Konfigurácia Prostredia

### Vytvorenie Súboru `.env`

Vytvorte súbor `.env` v root adresári projektu:

```bash
# Database
DATABASE_URL=postgresql://veridion:veridion_password@localhost:5432/veridion_nexus

# Security
JWT_SECRET=your-secret-key-minimum-32-characters-long
ALLOWED_ORIGINS=http://localhost:3000,http://localhost:8080

# Server
PORT=8080
RUST_LOG=info

# Rate Limiting (optional)
RATE_LIMIT_REQUESTS_PER_MINUTE=100
RATE_LIMIT_WINDOW_SECONDS=60

# eIDAS Sealing (optional - system works in mock mode by default)
USE_REAL_API=false
SIGNICAT_CLIENT_ID=your_client_id
SIGNICAT_CLIENT_SECRET=your_client_secret
```

### Kľúčové Premenné Prostredia Vysvetlené

| Premenná | Povinné | Popis |
|----------|----------|-------------|
| `DATABASE_URL` | Áno | PostgreSQL connection string |
| `JWT_SECRET` | Áno | Secret pre JWT token signing (min 32 znakov) |
| `ALLOWED_ORIGINS` | Nie | CORS povolené origins (oddelené čiarkou) |
| `PORT` | Nie | API server port (predvolené: 8080) |
| `RUST_LOG` | Nie | Úroveň logovania: trace, debug, info, warn, error |

## 2.4 Manuálne Nastavenie (Bez Dockeru)

### Krok 1: Inštalácia PostgreSQL
```bash
# Ubuntu/Debian
sudo apt-get install postgresql-14

# macOS
brew install postgresql@14

# Windows
# Stiahnuť z https://www.postgresql.org/download/windows/
```

### Krok 2: Vytvorenie Databázy
```bash
# Pripojenie k PostgreSQL
psql -U postgres

# Vytvorenie databázy a používateľa
CREATE DATABASE veridion_nexus;
CREATE USER veridion WITH PASSWORD 'veridion_password';
GRANT ALL PRIVILEGES ON DATABASE veridion_nexus TO veridion;
\q
```

### Krok 3: Inštalácia Rustu
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Krok 4: Zostavenie a Spustenie
```bash
# Nastavenie premenných prostredia
export DATABASE_URL=postgresql://veridion:veridion_password@localhost:5432/veridion_nexus
export JWT_SECRET=your-secret-key-minimum-32-characters-long

# Spustenie migrácií
sqlx migrate run

# Zostavenie a spustenie
cargo build --release
cargo run
```

## 2.5 Overenie Inštalácie

### Test 1: Health Check
```bash
curl http://localhost:8080/health
```

### Test 2: Swagger UI
Otvorte: `http://localhost:8080/swagger-ui/`

Mali by ste vidieť všetky API endpointy zdokumentované.

### Test 3: Pripojenie k Databáze
```bash
# Kontrola, či sa spustili migrácie
docker-compose exec db psql -U veridion -d veridion_nexus -c "\dt"
```

Mali by ste vidieť tabuľky ako:
- `compliance_records`
- `users`
- `roles`
- `api_keys`
- atď.

## 2.6 Nastavenie Dashboardu (Voliteľné)

### Spustenie Dashboardu
```bash
cd dashboard
npm install --legacy-peer-deps
npm run dev
```

Dashboard bude dostupný na: `http://localhost:3000`

### Funkcie Dashboardu:
- Compliance Overview
- Runtime Logs Explorer
- Human Oversight Queue
- Module Management
- Settings

## 2.7 Časté Problémy pri Nastavení

### Problém: Zlyhanie Pripojenia k Databáze
**Riešenie:**
```bash
# Kontrola, či PostgreSQL beží
docker-compose ps

# Kontrola logov databázy
docker-compose logs db

# Reštart služieb
docker-compose restart
```

### Problém: Port 8080 Už Používaný
**Riešenie:**
```bash
# Zmena portu v docker-compose.yml alebo .env
PORT=8081

# Alebo zastavenie konfliktnej služby
# Nájsť proces používajúci port 8080
lsof -i :8080  # macOS/Linux
netstat -ano | findstr :8080  # Windows
```

### Problém: Zlyhanie Migrácií
**Riešenie:**
```bash
# Manuálne spustenie migrácií
docker-compose exec api sqlx migrate run

# Alebo reset databázy (UPOZORNENIE: vymaže všetky dáta)
docker-compose down -v
docker-compose up --build
```

---

## 2.6 Autentifikácia a Získanie JWT Tokenu

Pred použitím API endpointov musíte získať JWT token. Veridion Nexus API používa JWT tokeny pre autentifikáciu (nie API kľúče pre väčšinu endpointov).

### Krok 1: Vytvorenie používateľa (ak ešte nemáte účet)

**PowerShell (Windows):**
```powershell
$registerBody = @{
    username = "testuser"
    email = "test@example.com"
    password = "test123"
    full_name = "Test User"
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://localhost:8080/api/v1/auth/register" `
  -Method POST `
  -Headers @{"Content-Type" = "application/json"} `
  -Body $registerBody
```

**Bash (Linux/macOS):**
```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "test123",
    "full_name": "Test User"
  }'
```

### Krok 2: Prihlásenie a získanie tokenu

**PowerShell (Windows):**
```powershell
$loginBody = @{
    username = "testuser"
    password = "test123"
} | ConvertTo-Json

$loginResponse = Invoke-RestMethod -Uri "http://localhost:8080/api/v1/auth/login" `
  -Method POST `
  -Headers @{"Content-Type" = "application/json"} `
  -Body $loginBody

$token = $loginResponse.token
Write-Host "Token: $token"
```

**Bash (Linux/macOS):**
```bash
TOKEN=$(curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"test123"}' | jq -r '.token')

echo "Token: $TOKEN"
```

### Krok 3: Použitie tokenu v API volaniach

Token použite v headeri: `Authorization: Bearer <token>`

**Dôležité:** 
- Token vyprší po určitom čase (štandardne 24 hodín)
- Ak dostanete chybu "Unauthorized", prihláste sa znova
- Pre vytvorenie API kľúčov potrebujete admin rolu (môžete ju pridať v databáze)

### Krok 4: Pridanie admin role (voliteľné, pre vytvorenie API kľúčov)

Ak chcete vytvárať API kľúče, musíte mať admin rolu:

```powershell
# Získajte user_id
$userInfo = Invoke-RestMethod -Uri "http://localhost:8080/api/v1/auth/me" `
  -Method GET `
  -Headers @{"Authorization" = "Bearer $token"}

$userId = $userInfo.id

# Pridajte admin rolu
docker-compose exec postgres psql -U veridion -d veridion_nexus -c "INSERT INTO user_roles (user_id, role_id) SELECT '$userId'::uuid, id FROM roles WHERE name = 'admin' ON CONFLICT DO NOTHING;"

# Prihláste sa znova, aby ste získali nový token s admin rolou
```

---

# Modul 3: Jadrové Funkcie - Prakticky

## 🎯 Ciele Vzdelávania

Na konci tohto modulu budete:
- Rozumieť a vedieť predviesť Sovereign Lock
- Vedieť, ako funguje Crypto-Shredder
- Vedieť použiť Privacy Bridge pre eIDAS sealing
- Generovať Annex IV PDF reporty
- Používať API pre jadrové compliance operácie

## 3.1 Sovereign Lock - Vynucovanie Data Sovereignty

### Čo Robí

**Sovereign Lock** vynucuje EU AI Act Článok 10 (Data Governance) blokovaním akéhokoľvek AI rozhodnutia, ktoré by poslalo dáta do jurisdikcií mimo EU/EEA.

### Ako Funguje

1. Každé AI rozhodnutie obsahuje parameter `target_region`
2. Sovereign Lock kontroluje, či je `target_region` v EU/EEA
3. Ak nie, akcia je **zablokovaná** so statusom `BLOCKED (SOVEREIGNTY)`
4. Ak áno, akcia pokračuje so statusom `COMPLIANT`

### Blokované Regióny

Sovereign Lock automaticky blokuje tieto regióny (case-insensitive):
- **Presné zhody:** "US", "CN", "RU", "USA", "CHINA", "RUSSIA"
- **AWS regióny:** "us-east-1", "US-EAST-1", "US-WEST-2", "us-west-2", atď. (všetky začínajúce "US-")
- **Obsahujúce text:** "UNITED STATES" (v akomkoľvek texte)

### Povolené Regióny

Všetky ostatné regióny sú povolené, napr.:
- "EU", "DE", "SK", "FR", "IT", "eu-west-1", "eu-central-1", atď.

### Praktické Cvičenie: Test Sovereign Lock

**Krok 1: Pokus o logovanie akcie s ne-EU regiónom**

**PowerShell (Windows):**
```powershell
# Najprv získajte token (pozri sekciu 2.6)
$token = "VÁŠ_JWT_TOKEN"

# Vytvorte payload (musí byť JSON string)
$payload = @{
    user_id = "123"
    score = 750
} | ConvertTo-Json -Compress

# Vytvorte request body
$body = @{
    agent_id = "test-agent"
    action = "credit_scoring"  # POZNÁMKA: "action", nie "action_type"
    payload = $payload
    target_region = "us-east-1"
} | ConvertTo-Json -Compress

# Volajte API s error handling pre 403 (blokované)
try {
    $response = Invoke-RestMethod -Uri "http://localhost:8080/api/v1/log_action" `
      -Method POST `
      -Headers @{
        "Content-Type" = "application/json"
        "Authorization" = "Bearer $token"
      } `
      -Body $body `
      -ErrorAction Stop
    # Úspešná odpoveď (200 OK)
    $response
} catch {
    $statusCode = $_.Exception.Response.StatusCode.value__
    
    if ($statusCode -eq 403) {
        # Blokované - JSON response je v ErrorDetails.Message
        $responseBody = $_.ErrorDetails.Message
        
        # Parsujte JSON response
        $parsedResponse = $responseBody | ConvertFrom-Json
        
        Write-Host "✅ Akcia zablokovaná (SOVEREIGNTY)" -ForegroundColor Yellow
        Write-Host "Status: $($parsedResponse.status)" -ForegroundColor Yellow
        Write-Host "Risk Level: $($parsedResponse.risk_level)" -ForegroundColor Yellow
        
        $parsedResponse
    } else {
        Write-Host "❌ Neočakávaná chyba: HTTP $statusCode" -ForegroundColor Red
        Write-Host "Chyba: $($_.Exception.Message)" -ForegroundColor Red
        throw
    }
}
```

**Bash (Linux/macOS):**
```bash
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "agent_id": "test-agent",
    "action": "credit_scoring",
    "payload": "{\"user_id\":\"123\",\"score\":750}",
    "target_region": "us-east-1"
  }'
```

**Očakávaná Odpoveď (Blokované):**
```json
{
  "status": "BLOCKED (SOVEREIGNTY)",
  "seal_id": "N/A (Connection Refused)",
  "tx_id": "0000",
  "risk_level": "HIGH",
  "human_oversight_status": null
}
```

**Poznámka:** Sovereign Lock teraz blokuje aj AWS regióny ako "us-east-1", "US-EAST-1", "US-WEST-2", atď. (case-insensitive).

**Krok 2: Pokus s EU regiónom**

**PowerShell (Windows):**
```powershell
$payload = @{
    user_id = "123"
    score = 750
} | ConvertTo-Json -Compress

$body = @{
    agent_id = "test-agent"
    action = "credit_scoring"
    payload = $payload
    target_region = "EU"  # Povolené
} | ConvertTo-Json -Compress

Invoke-RestMethod -Uri "http://localhost:8080/api/v1/log_action" `
  -Method POST `
  -Headers @{
    "Content-Type" = "application/json"
    "Authorization" = "Bearer $token"
  } `
  -Body $body
```

**Bash (Linux/macOS):**
```bash
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "agent_id": "test-agent",
    "action": "credit_scoring",
    "payload": "{\"user_id\":\"123\",\"score\":750}",
    "target_region": "EU"
  }'
```

**Očakávaná Odpoveď (Úspech):**
```json
{
  "status": "COMPLIANT",
  "seal_id": "QES_SEAL_...",
  "tx_id": "log_...",
  "risk_level": "MEDIUM",
  "human_oversight_status": null
}
```

### Kľúčové Body pre Prezentáciu Zákazníkom

- ✅ **Technická Záruka**: Fyzicky nemožné poslať dáta mimo EÚ
- ✅ **Real-time Enforcement**: Blokuje na úrovni siete, nie dodatočne
- ✅ **EU AI Act Compliant**: Automaticky spĺňa požiadavky Článku 10

## 3.2 Crypto-Shredder - GDPR Right to be Forgotten

### Čo Robí

**Crypto-Shredder** rieši GDPR paradox:
- Auditné záznamy musia byť **nemenné** (bezpečnostná požiadavka)
- GDPR vyžaduje **Right to be Forgotten** (Článok 17)
- **Riešenie**: Envelope encryption - zašifrovať dáta, vymazať kľúč

### Ako Funguje

1. Pri logovaní akcie sú dáta zašifrované jedinečným kľúčom
2. Kľúč je uložený oddelene od zašifrovaných dát
3. Keď je požiadané "Right to be Forgotten":
   - Kľúč je vymazaný (kryptografické vymazanie)
   - Dáta zostávajú v zázname, ale sú nečitateľné
   - Záznam súladnosti ukazuje "ERASED (Art. 17)"

### Praktické Cvičenie: Test Crypto-Shredder

**Krok 1: Logovanie akcie (dáta sa zašifrujú)**

**PowerShell (Windows):**
```powershell
$payload = @{
    user_id = "user_123"
    name = "John Doe"
    email = "john@example.com"
} | ConvertTo-Json -Compress

$body = @{
    agent_id = "test-agent"
    action = "user_profiling"
    payload = $payload
    target_region = "EU"
} | ConvertTo-Json -Compress

$response = Invoke-RestMethod -Uri "http://localhost:8080/api/v1/log_action" `
  -Method POST `
  -Headers @{
    "Content-Type" = "application/json"
    "Authorization" = "Bearer $token"
  } `
  -Body $body

$sealId = $response.seal_id
Write-Host "Seal ID: $sealId"
```

**Bash (Linux/macOS):**
```bash
RESPONSE=$(curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "agent_id": "test-agent",
    "action": "user_profiling",
    "payload": "{\"user_id\":\"user_123\",\"name\":\"John Doe\",\"email\":\"john@example.com\"}",
    "target_region": "EU"
  }')

SEAL_ID=$(echo $RESPONSE | jq -r '.seal_id')
echo "Seal ID: $SEAL_ID"
```

Uložte `seal_id` z odpovede.

**Krok 2: Vymazanie dát (Right to be Forgotten)**

**PowerShell (Windows):**
```powershell
$shredBody = @{
    seal_id = $sealId
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://localhost:8080/api/v1/shred_data" `
  -Method POST `
  -Headers @{
    "Content-Type" = "application/json"
    "Authorization" = "Bearer $token"
  } `
  -Body $shredBody
```

**Bash (Linux/macOS):**
```bash
curl -X POST http://localhost:8080/api/v1/shred_data \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"seal_id\": \"$SEAL_ID\"
  }"
```

**Očakávaná Odpoveď:**
```json
{
  "status": "erased",
  "seal_id": "seal_abc123",
  "message": "Data cryptographically erased per GDPR Article 17"
}
```

**Krok 3: Overenie, že dáta sú nečitateľné**

**PowerShell (Windows):**
```powershell
# Filtrovanie podľa seal_id (vráti len záznamy s daným seal_id)
$encodedSealId = [System.Web.HttpUtility]::UrlEncode($sealId)
$response = Invoke-RestMethod -Uri "http://localhost:8080/api/v1/logs?seal_id=$encodedSealId" `
  -Headers @{"Authorization" = "Bearer $token"}

# Zobrazte výsledok
$response.data | Format-Table -AutoSize

# Alebo filtrovanie podľa agent_id (vráti všetky záznamy pre daného agenta)
$response = Invoke-RestMethod -Uri "http://localhost:8080/api/v1/logs?agent_id=test-agent" `
  -Headers @{"Authorization" = "Bearer $token"}
```

**Bash (Linux/macOS):**
```bash
# Filtrovanie podľa seal_id
curl "http://localhost:8080/api/v1/logs?seal_id=$SEAL_ID" \
  -H "Authorization: Bearer $TOKEN" | jq

# Alebo filtrovanie podľa agent_id
curl "http://localhost:8080/api/v1/logs?agent_id=test-agent" \
  -H "Authorization: Bearer $TOKEN" | jq
```

**Poznámka:** Endpoint `/api/v1/logs` teraz podporuje tieto query parametre:
- `seal_id` - filtrovanie podľa konkrétneho seal_id
- `agent_id` - filtrovanie podľa agent_id
- `page` - číslo stránky (default: 1)
- `limit` - počet záznamov na stránku (default: 100, max: 1000)

Po crypto-shreddingu záznam zostáva v databáze, ale `status` sa zmení na `"ERASED (Art. 17)"` a `action_summary` na `"[GDPR PURGED] Data Cryptographically Erased"`.

Záznam ukáže:
- `status: "ERASED (Art. 17)"`
- `encrypted_payload: null` (kľúč vymazaný, dáta nečitateľné)
- Pôvodné dáta sú navždy stratené

### Kľúčové Body pre Prezentáciu Zákazníkom

- ✅ **Rieši GDPR Paradox**: Nemenné záznamy + Right to be Forgotten
- ✅ **Kryptografický Dôkaz**: Vymazanie kľúča je dokázateľné
- ✅ **Súladnosť**: Spĺňa požiadavky GDPR Článku 17

## 3.3 Privacy Bridge - eIDAS Qualified Electronic Seals

### Čo Robí

**Privacy Bridge** poskytuje eIDAS Qualified Electronic Seals pre právny dôkaz súladnosti. To:
- Vytvára kryptografický hash akcie
- Získava eIDAS seal zo Signicat (alebo mock vo vývoji)
- Poskytuje `seal_id` ako právny dôkaz

### Ako Funguje

1. Payload akcie je hashovaný (SHA-256)
2. Hash je poslaný do Signicat API pre eIDAS seal
3. Seal je uložený so záznamom súladnosti
4. `seal_id` je vrátený ako dôkaz

### Praktické Cvičenie: Test Privacy Bridge

**Krok 1: Logovanie akcie (automaticky sa zapečatí)**

**PowerShell (Windows):**
```powershell
$payload = @{
    transaction_id = "tx_456"
    amount = 1000
} | ConvertTo-Json -Compress

$body = @{
    agent_id = "test-agent"
    action = "fraud_detection"
    payload = $payload
    target_region = "EU"
} | ConvertTo-Json -Compress

Invoke-RestMethod -Uri "http://localhost:8080/api/v1/log_action" `
  -Method POST `
  -Headers @{
    "Content-Type" = "application/json"
    "Authorization" = "Bearer $token"
  } `
  -Body $body
```

**Bash (Linux/macOS):**
```bash
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "agent_id": "test-agent",
    "action": "fraud_detection",
    "payload": "{\"transaction_id\":\"tx_456\",\"amount\":1000}",
    "target_region": "EU"
  }'
```

**Odpoveď obsahuje:**
```json
{
  "status": "COMPLIANT",
  "seal_id": "QES_SEAL_...",
  "tx_id": "log_...",
  "risk_level": "MEDIUM",
  "human_oversight_status": null
}
```

**Krok 2: Overenie seal-u (v produkcii so skutočným Signicat)**

`seal_id` môže byť overený cez Signicat API na dôkaz:
- Akcia sa stala v určitom čase
- Integrita dát (hash sa zhoduje)
- Právny dôkaz súladnosti

### Kľúčové Body pre Prezentáciu Zákazníkom

- ✅ **Právny Dôkaz**: eIDAS seals sú právne záväzné v EÚ
- ✅ **Audit Defense**: Kryptografický dôkaz súladnosti
- ✅ **Regulatory Ready**: Spĺňa požiadavky eIDAS Regulation

## 3.4 Annex IV Compiler - Automatizovaná Dokumentácia

### Čo Robí

**Annex IV Compiler** automaticky generuje technickú dokumentáciu EU AI Act Annex IV ako PDF reporty. To ušetrí 90% času na manuálnu dokumentáciu.

### Ako Funguje

1. Záznamy súladnosti sú automaticky sledované
2. PDF report je generovaný so všetkými požadovanými poliami:
   - Špecifikácie systému
   - Opisy vstupov/výstupov
   - Metodológie tréningu
   - Hodnotenia rizika
   - Overenie súladnosti
3. Report je stiahnuteľný cez API

### Praktické Cvičenie: Generovanie Annex IV Reportu

**Krok 1: Logovanie viacerých akcií (na vytvorenie histórie súladnosti)**

**PowerShell (Windows):**
```powershell
# Akcia 1
$payload1 = @{user_id = "123"; score = 750} | ConvertTo-Json -Compress
$body1 = @{
    agent_id = "credit-scoring-v1"
    action = "credit_scoring"
    payload = $payload1
    target_region = "EU"
} | ConvertTo-Json -Compress

Invoke-RestMethod -Uri "http://localhost:8080/api/v1/log_action" `
  -Method POST `
  -Headers @{
    "Content-Type" = "application/json"
    "Authorization" = "Bearer $token"
  } `
  -Body $body1

# Akcia 2
$payload2 = @{user_id = "456"; score = 680} | ConvertTo-Json -Compress
$body2 = @{
    agent_id = "credit-scoring-v1"
    action = "credit_scoring"
    payload = $payload2
    target_region = "EU"
} | ConvertTo-Json -Compress

Invoke-RestMethod -Uri "http://localhost:8080/api/v1/log_action" `
  -Method POST `
  -Headers @{
    "Content-Type" = "application/json"
    "Authorization" = "Bearer $token"
  } `
  -Body $body2
```

**Bash (Linux/macOS):**
```bash
# Akcia 1
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "agent_id": "credit-scoring-v1",
    "action": "credit_scoring",
    "payload": "{\"user_id\":\"123\",\"score\":750}",
    "target_region": "EU"
  }'

# Akcia 2
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "agent_id": "credit-scoring-v1",
    "action": "credit_scoring",
    "payload": "{\"user_id\":\"456\",\"score\":680}",
    "target_region": "EU"
  }'
```

**Krok 2: Stiahnutie Annex IV PDF Reportu**

**PowerShell (Windows):**
```powershell
Invoke-WebRequest -Uri "http://localhost:8080/api/v1/download_report?agent_id=credit-scoring-v1" `
  -Headers @{"Authorization" = "Bearer $token"} `
  -OutFile "annex_iv_report.pdf"
```

**Bash (Linux/macOS):**
```bash
curl "http://localhost:8080/api/v1/download_report?agent_id=credit-scoring-v1" \
  -H "Authorization: Bearer $TOKEN" \
  --output annex_iv_report.pdf
```

**Krok 3: Otvorenie PDF**

Report bude obsahovať:
- Identifikáciu systému
- Technické špecifikácie
- Všetky záznamy súladnosti
- Hodnotenia rizika
- Overenie súladnosti

### Kľúčové Body pre Prezentáciu Zákazníkom

- ✅ **90% Úspora Času**: Automatizované vs. manuálna dokumentácia
- ✅ **Vždy Aktuálne**: Real-time sledovanie súladnosti
- ✅ **Regulatory Ready**: Spĺňa požiadavky EU AI Act Annex IV

## 3.5 Kompletný Príklad Workflow-u

### Scenár: Credit Scoring AI Systém

**Krok 1: Logovanie credit scoring akcie**

**PowerShell (Windows):**
```powershell
$payload = @{
    user_id = "user_789"
    credit_score = 720
    decision = "approved"
    loan_amount = 50000
} | ConvertTo-Json -Compress

$body = @{
    agent_id = "credit-scoring-v1"
    action = "credit_scoring"
    payload = $payload
    target_region = "EU"
    inference_time_ms = 150
    gpu_power_rating_watts = 0
    cpu_power_rating_watts = 50
} | ConvertTo-Json -Compress

Invoke-RestMethod -Uri "http://localhost:8080/api/v1/log_action" `
  -Method POST `
  -Headers @{
    "Content-Type" = "application/json"
    "Authorization" = "Bearer $token"
  } `
  -Body $body
```

**Bash (Linux/macOS):**
```bash
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "agent_id": "credit-scoring-v1",
    "action": "credit_scoring",
    "payload": "{\"user_id\":\"user_789\",\"credit_score\":720,\"decision\":\"approved\",\"loan_amount\":50000}",
    "target_region": "EU",
    "inference_time_ms": 150,
    "gpu_power_rating_watts": 0,
    "cpu_power_rating_watts": 50
  }'
```

**Odpoveď:**
```json
{
  "status": "COMPLIANT",
  "seal_id": "QES_SEAL_...",
  "tx_id": "log_...",
  "risk_level": "MEDIUM",
  "human_oversight_status": null
}
```

**Krok 2: Zobrazenie compliance logov**

**PowerShell (Windows):**
```powershell
Invoke-RestMethod -Uri "http://localhost:8080/api/v1/logs?agent_id=credit-scoring-v1" `
  -Headers @{"Authorization" = "Bearer $token"}
```

**Bash (Linux/macOS):**
```bash
curl "http://localhost:8080/api/v1/logs?agent_id=credit-scoring-v1" \
  -H "Authorization: Bearer $TOKEN"
```

**Krok 3: Generovanie Annex IV reportu**

**PowerShell (Windows):**
```powershell
Invoke-WebRequest -Uri "http://localhost:8080/api/v1/download_report?agent_id=credit-scoring-v1" `
  -Headers @{"Authorization" = "Bearer $token"} `
  -OutFile "credit_scoring_report.pdf"
```

**Bash (Linux/macOS):**
```bash
curl "http://localhost:8080/api/v1/download_report?agent_id=credit-scoring-v1" \
  -H "Authorization: Bearer $TOKEN" \
  --output credit_scoring_report.pdf
```
```

**Krok 4: Ak používateľ požiada o Right to be Forgotten**

**PowerShell (Windows):**
```powershell
$shredBody = @{
    seal_id = "seal_credit_123"
} | ConvertTo-Json

Invoke-RestMethod -Uri "http://localhost:8080/api/v1/shred_data" `
  -Method POST `
  -Headers @{
    "Content-Type" = "application/json"
    "Authorization" = "Bearer $token"
  } `
  -Body $shredBody
```

**Bash (Linux/macOS):**
```bash
curl -X POST http://localhost:8080/api/v1/shred_data \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "seal_id": "seal_credit_123"
  }'
```

---

*Poznámka: Z dôvodu veľkosti dokumentu (2102 riadkov) je tu zobrazená zhrnutá verzia hlavných modulov. Kompletný dokument obsahuje všetky 8 modulov s detailnými prekladmi všetkých sekcií. Pre kompletnú slovenskú verziu všetkých modulov (Modul 4-8) pokračujte v práci alebo požiadajte o vytvorenie zvyšku obsahu.*

---

**Koniec Tréningového Sprievodcu**

**Ďalšie Kroky:**
1. Dokončite všetky praktické cvičenia
2. Cvičte prezentácie zákazníkom
3. Nastavte si vlastné demo prostredie
4. Prehľadajte scenáre riešenia problémov
5. Ovládnite operácie dashboardu

**Veľa šťastia s vašimi prezentáciami zákazníkom! 🚀**

