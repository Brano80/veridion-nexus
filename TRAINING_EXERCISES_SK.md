# Veridion Nexus - Praktické Cvičenia
## Hands-On Tréningové Scenáre

Tento dokument obsahuje praktické cvičenia na zvládnutie operácií Veridion Nexus.

---

## Cvičenie 1: Základné Nastavenie & Overenie

### Cieľ
Nastaviť Veridion Nexus a overiť, že funguje správne.

### Kroky

1. **Spustenie platformy:**
```bash
docker-compose up --build
```

2. **Overenie zdravia:**
```bash
curl http://localhost:8080/health
```

Očakávané: `{"status":"healthy","service":"veridion-nexus","version":"1.0.0"}`

3. **Kontrola Swagger UI:**
Otvorte: `http://localhost:8080/swagger-ui/`

4. **Overenie databázy:**
```bash
docker-compose exec db psql -U veridion -d veridion_nexus -c "\dt"
```

Očakávané: Zoznam tabuliek (compliance_records, users, atď.)

### Kritériá Úspechu
- ✅ Health check vráti "healthy"
- ✅ Swagger UI sa načíta
- ✅ Databáza má tabuľky

---

## Cvičenie 2: Demonštrácia Sovereign Lock

### Cieľ
Demonštrovať, ako Sovereign Lock blokuje ne-EU regióny.

### Kroky

1. **Pokus o logovanie akcie s US regiónom (má zlyhať):**
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

Očakávané: Chyba `SOVEREIGN_LOCK_VIOLATION`

2. **Pokus s EU regiónom (má uspieť):**
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

Očakávané: Úspech s `seal_id` a `tx_id`

### Kritériá Úspechu
- ✅ US región je zablokovaný
- ✅ EU región je povolený
- ✅ Odpoveď obsahuje seal_id

---

## Cvičenie 3: Kompletný Compliance Workflow

### Cieľ
Vykonať kompletný compliance workflow od logovania po reportovanie.

### Kroky

1. **Logovanie viacerých akcií:**
```bash
# Akcia 1
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

# Akcia 2
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

Uložte hodnoty `seal_id` z odpovedí.

2. **Zobrazenie compliance logov:**
```bash
curl http://localhost:8080/api/v1/logs?agent_id=credit-scoring-v1 \
  -H "X-API-Key: test-key"
```

3. **Generovanie Annex IV reportu:**
```bash
curl http://localhost:8080/api/v1/download_report?agent_id=credit-scoring-v1 \
  -H "X-API-Key: test-key" \
  --output credit_scoring_report.pdf
```

4. **Vykonanie Right to be Forgotten:**
```bash
curl -X POST http://localhost:8080/api/v1/shred_data \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "seal_id": "seal_id_from_step_1"
  }'
```

5. **Overenie, že dáta sú vymazané:**
```bash
curl http://localhost:8080/api/v1/logs?seal_id=seal_id_from_step_1 \
  -H "X-API-Key: test-key"
```

Očakávané: Záznam ukazuje `status: "ERASED (Art. 17)"`

### Kritériá Úspechu
- ✅ Akcie sú úspešne zalogované
- ✅ Logy sú zobraziteľné
- ✅ PDF report je vygenerovaný
- ✅ Dáta môžu byť vymazané
- ✅ Vymazané dáta ukazujú správny stav

---

## Cvičenie 4: Správa Modulov

### Cieľ
Naučiť sa, ako zapnúť/vypnúť operačné moduly.

### Kroky

1. **Zoznam všetkých modulov:**
```bash
curl http://localhost:8080/api/v1/modules \
  -H "X-API-Key: test-key"
```

2. **Zapnutie modulu Human Oversight:**
```bash
curl -X POST http://localhost:8080/api/v1/modules/human_oversight/enable \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{}'
```

3. **Kontrola stavu modulu:**
```bash
curl http://localhost:8080/api/v1/modules/human_oversight/status \
  -H "X-API-Key: test-key"
```

Očakávané: `{"enabled": true}`

4. **Vypnutie modulu:**
```bash
curl -X POST http://localhost:8080/api/v1/modules/human_oversight/disable \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{}'
```

### Kritériá Úspechu
- ✅ Môžem zobraziť zoznam modulov
- ✅ Môžem zapnúť modul
- ✅ Môžem skontrolovať stav
- ✅ Môžem vypnúť modul

---

## Cvičenie 5: Human Oversight Workflow

### Cieľ
Cvičiť workflow schvaľovania human oversight.

### Predpoklady
- Human Oversight modul zapnutý (Cvičenie 4)

### Kroky

1. **Logovanie akcie:**
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

Uložte `seal_id`.

2. **Vyžadovanie schválenia:**
```bash
curl -X POST http://localhost:8080/api/v1/action/{seal_id}/require_approval \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "reason": "High-risk transaction detected",
    "reviewer_role": "compliance_officer"
  }'
```

3. **Schválenie akcie:**
```bash
curl -X POST http://localhost:8080/api/v1/action/{seal_id}/approve \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "reviewer_id": "reviewer_001",
    "notes": "Approved after manual review"
  }'
```

4. **Alternatíva: Zamietnutie akcie:**
```bash
# (Namiesto schválenia môžete zamietnuť)
curl -X POST http://localhost:8080/api/v1/action/{seal_id}/reject \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{
    "reviewer_id": "reviewer_001",
    "reason": "Risk score too high"
  }'
```

### Kritériá Úspechu
- ✅ Môžem vyžadovať schválenie
- ✅ Môžem schváliť akciu
- ✅ Môžem zamietnuť akciu
- ✅ Schválenie/zamietnutie je zaznamenané

---

## Cvičenie 6: Risk Assessment

### Cieľ
Cvičiť funkcionalitu risk assessment.

### Kroky

1. **Logovanie high-risk akcie:**
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

Uložte `seal_id`.

2. **Získanie risk assessment:**
```bash
curl http://localhost:8080/api/v1/risk_assessment/{seal_id} \
  -H "X-API-Key: test-key"
```

3. **Zoznam všetkých high-risk assessmentov:**
```bash
curl http://localhost:8080/api/v1/risks?risk_level=high \
  -H "X-API-Key: test-key"
```

### Kritériá Úspechu
- ✅ Risk assessment je vygenerovaný
- ✅ Úroveň rizika je identifikovaná
- ✅ Môžem filtrovať podľa úrovne rizika

---

## Cvičenie 7: Reportovanie Data Breach

### Cieľ
Cvičiť workflow reportovania data breach.

### Kroky

1. **Reportovanie breach:**
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

Uložte `breach_id`.

2. **Zoznam všetkých breachov:**
```bash
curl http://localhost:8080/api/v1/breaches \
  -H "X-API-Key: test-key"
```

3. **Zobrazenie konkrétneho breachu:**
```bash
curl http://localhost:8080/api/v1/breaches/{breach_id} \
  -H "X-API-Key: test-key"
```

### Kritériá Úspechu
- ✅ Breach môže byť nahlásený
- ✅ Breach sa objaví v zozname
- ✅ Deadline na notifikáciu je vypočítaný (72 hodín)

---

## Cvičenie 8: Konfigurácia Webhook

### Cieľ
Nastaviť a otestovať webhook notifikácie.

### Kroky

1. **Vytvorenie test webhook endpointu** (použite webhook.site alebo podobné):
- Prejdite na https://webhook.site
- Skopírujte svoj jedinečný webhook URL

2. **Registrácia webhooku:**
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

Uložte `webhook_id`.

3. **Spustenie eventu (logovanie akcie):**
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

4. **Kontrola doručenia webhooku:**
- Prejdite na webhook.site a pozrite, či bol požiadavka prijatá
- Alebo skontrolujte históriu doručenia:

```bash
curl http://localhost:8080/api/v1/webhooks/{webhook_id}/deliveries \
  -H "X-API-Key: test-key"
```

5. **Zoznam všetkých webhookov:**
```bash
curl http://localhost:8080/api/v1/webhooks \
  -H "X-API-Key: test-key"
```

### Kritériá Úspechu
- ✅ Webhook je registrovaný
- ✅ Event spúšťa doručenie webhooku
- ✅ História doručenia je sledovaná
- ✅ Môžem overiť podpis webhooku

---

## Cvičenie 9: Správa API Kľúčov

### Cieľ
Cvičiť vytváranie a správu API kľúčov.

### Predpoklady
- JWT autentifikácia (potrebujete sa najprv prihlásiť)

### Kroky

1. **Prihlásenie pre získanie JWT tokenu:**
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "your-password"
  }'
```

Uložte `token`.

2. **Vytvorenie API kľúča:**
```bash
curl -X POST http://localhost:8080/api/v1/api_keys \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {token}" \
  -d '{
    "name": "production-api-key",
    "expires_at": "2026-01-15T00:00:00Z"
  }'
```

**⚠️ DÔLEŽITÉ: Uložte API kľúč okamžite - zobrazí sa len raz!**

3. **Zoznam všetkých API kľúčov:**
```bash
curl http://localhost:8080/api/v1/api_keys \
  -H "Authorization: Bearer {token}"
```

4. **Získanie detailov API kľúča:**
```bash
curl http://localhost:8080/api/v1/api_keys/{api_key_id} \
  -H "Authorization: Bearer {token}"
```

5. **Test API kľúča:**
```bash
curl http://localhost:8080/api/v1/logs \
  -H "X-API-Key: {api_key_from_step_2}"
```

6. **Zrušenie API kľúča:**
```bash
curl -X POST http://localhost:8080/api/v1/api_keys/{api_key_id}/revoke \
  -H "Authorization: Bearer {token}" \
  -d '{}'
```

### Kritériá Úspechu
- ✅ Môžem vytvoriť API kľúč
- ✅ API kľúč funguje pre autentifikáciu
- ✅ Môžem zobraziť zoznam API kľúčov
- ✅ Môžem zrušiť API kľúč

---

## Cvičenie 10: Operácie Dashboardu

### Cieľ
Ovládnuť navigáciu a operácie dashboardu.

### Kroky

1. **Spustenie dashboardu:**
```bash
cd dashboard
npm run dev
```

2. **Otvorenie dashboardu:**
Prejdite na: `http://localhost:3000`

3. **Dokončenie úloh dashboardu:**
   - Zobrazenie Compliance Overview
   - Navigácia na Runtime Logs
   - Filtrovanie logov podľa agent_id
   - Generovanie Annex IV reportu
   - Zobrazenie stavu modulov
   - Vytvorenie API kľúča cez UI
   - Konfigurácia webhooku cez UI

4. **Test správy modulov:**
   - Zapnutie modulu
   - Vypnutie modulu
   - Overenie, že modul sa objaví/zmizne v navigácii

### Kritériá Úspechu
- ✅ Dashboard sa načíta správne
- ✅ Môžem navigovať všetky stránky
- ✅ Môžem vykonávať operácie cez UI
- ✅ Správa modulov funguje

---

## Cvičenie 11: Python SDK Integrácia

### Cieľ
Integrovať Veridion Nexus s Python aplikáciou.

### Predpoklady
- Python 3.8+
- Veridion Nexus API beží

### Kroky

1. **Inštalácia SDK:**
```bash
pip install veridion-nexus-sdks[langchain]
```

2. **Vytvorenie test skriptu:**
```python
# test_veridion.py
from sdks.langchain import wrap_langchain_llm
from langchain.llms import OpenAI

# Vytvorenie LangChain LLM
llm = OpenAI(temperature=0.7)

# Zabalenie s Veridion súladnosťou
veridion_llm = wrap_langchain_llm(
    llm=llm,
    veridion_api_url="http://localhost:8080",
    veridion_api_key="your-api-key",
    agent_id="my-python-agent"
)

# Použitie normálne - súladnosť je automatická
response = veridion_llm("What is GDPR?")
print(response)
```

3. **Spustenie skriptu:**
```bash
python test_veridion.py
```

4. **Overenie compliance logovania:**
```bash
curl http://localhost:8080/api/v1/logs?agent_id=my-python-agent \
  -H "X-API-Key: your-api-key"
```

### Kritériá Úspechu
- ✅ SDK sa nainštaluje správne
- ✅ Skript beží bez chýb
- ✅ Záznamy súladnosti sú vytvorené
- ✅ Môžem zobraziť logy v API

---

## Cvičenie 12: Kompletný Scenár Demo pre Zákazníka

### Cieľ
Cvičiť kompletný scenár prezentácie zákazníkom.

### Scenár: Fintech Startup Demo

**Nastavenie:**
1. Spustiť Veridion Nexus
2. Spustiť Dashboard
3. Mať pripravené Swagger UI

**Demo Flow (15 minút):**

1. **Úvod (2 min)**
   - Vysvetliť požiadavky EU AI Act
   - Ukázať problém: Manuálna súladnosť je drahá

2. **Živé Demo: Sovereign Lock (3 min)**
   - Ukázať blokovanie ne-EU regiónu
   - Ukázať povolenie EU regiónu
   - Vysvetliť technickú záruku

3. **Živé Demo: Compliance Logovanie (3 min)**
   - Logovať credit scoring akciu
   - Ukázať dashboard s logmi
   - Ukázať generovanie Annex IV PDF

4. **Živé Demo: Right to be Forgotten (2 min)**
   - Ukázať vymazanie dát
   - Overiť, že dáta sú nečitateľné
   - Vysvetliť GDPR súladnosť

5. **SDK Integrácia (3 min)**
   - Ukázať príklad Python SDK
   - Vysvetliť automatickú súladnosť
   - Ukázať, aká jednoduchá je integrácia

6. **Diskusia o Cenovaní (2 min)**
   - Starter tier: €35K/rok
   - Čo je zahrnuté
   - Hodnotová propózia

### Kritériá Úspechu
- ✅ Demo prebieha plynule
- ✅ Všetky funkcie fungujú
- ✅ Môžem odpovedať na otázky
- ✅ Cenovanie je jasné

---

## Cvičenie 13: Cvičenie Riešenia Problémov

### Cieľ
Cvičiť diagnostikovanie a riešenie bežných problémov.

### Scenáre

**Scenár 1: API Vracia 500 Chybu**
- Skontrolovať logy: `docker-compose logs api`
- Overiť pripojenie k databáze
- Skontrolovať premenné prostredia
- Opraviť problém

**Scenár 2: Zlyhanie Pripojenia k Databáze**
- Skontrolovať, či PostgreSQL beží
- Overiť DATABASE_URL
- Skontrolovať logy databázy
- Opraviť pripojenie

**Scenár 3: Webhook Nedoručuje**
- Skontrolovať, či je webhook URL dostupný
- Overiť webhook secret
- Skontrolovať históriu doručenia
- Opraviť konfiguráciu

**Scenár 4: Dashboard sa Nenačítava**
- Skontrolovať, či API beží
- Overiť CORS konfiguráciu
- Skontrolovať konzolu prehliadača
- Opraviť problém

### Kritériá Úspechu
- ✅ Môžem diagnostikovať problémy
- ✅ Môžem opraviť problémy
- ✅ Rozumiem analýze logov
- ✅ Viem, kde nájsť pomoc

---

## Cvičenie 14: Testovanie Výkonu

### Cieľ
Testovať výkon platformy pod zaťažením.

### Kroky

1. **Test čas odozvy API:**
```bash
time curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: test-key" \
  -d '{"agent_id":"test","action_type":"test","payload":{},"target_region":"eu-west-1"}'
```

2. **Load test (najprv inštalujte Apache Bench):**
```bash
# Inštalácia: apt-get install apache2-utils (Linux) alebo brew install httpd (macOS)

ab -n 100 -c 10 -H "X-API-Key: test-key" \
  -p test_payload.json -T application/json \
  http://localhost:8080/api/v1/log_action
```

3. **Monitorovanie výkonu databázy:**
```sql
-- Kontrola pomalých dotazov
SELECT * FROM pg_stat_statements 
ORDER BY total_time DESC 
LIMIT 10;
```

4. **Kontrola connection poolu:**
```sql
-- Kontrola aktívnych pripojení
SELECT count(*) FROM pg_stat_activity;
```

### Kritériá Úspechu
- ✅ API odpovedá rýchlo (<100ms)
- ✅ Môže zvládnuť konkurentné požiadavky
- ✅ Výkon databázy je dobrý
- ✅ Žiadne memory leaks

---

## Cvičenie 15: Bezpečnostný Audit

### Cieľ
Cvičiť bezpečnostné best practices.

### Kroky

1. **Kontrola bezpečnosti API kľúčov:**
   - Overiť, že kľúče nie sú v kóde
   - Skontrolovať, či sú kľúče rotované
   - Overiť, že nepoužívané kľúče sú zrušené

2. **Kontrola JWT konfigurácie:**
   - Overiť, že JWT_SECRET je silný (32+ znakov)
   - Skontrolovať expiráciu tokenu
   - Overiť, že tokeny nie sú exponované

3. **Kontrola CORS konfigurácie:**
   - Overiť, že ALLOWED_ORIGINS je nastavené (nie *)
   - Skontrolovať produkciu vs. vývoj

4. **Kontrola bezpečnosti databázy:**
   - Overiť silné heslá
   - Skontrolovať, či je SSL povolené
   - Overiť pravidlá firewallu

5. **Kontrola audit logovania:**
```sql
SELECT * FROM security_audit_logs 
ORDER BY created_at DESC 
LIMIT 100;
```

### Kritériá Úspechu
- ✅ Všetky bezpečnostné kontroly prejdú
- ✅ Žiadne prihlasovacie údaje v kóde
- ✅ CORS je správne nakonfigurovaný
- ✅ Audit logy fungujú

---

## Finálne Hodnotenie

### Dokončenie Všetkých Cvičení

Ohodnoťte sa na každom cvičení:
- ✅ **Zvládnuté**: Môžem urobiť bez referencie
- ⚠️ **Oboznámené**: Môžem urobiť s referenciou
- ❌ **Potrebuje Cvičenie**: Potrebujem viac času

### Pripravenosť na Prezentáciu Zákazníkom

**Môžete:**
- ✅ Nastaviť Veridion Nexus od začiatku?
- ✅ Predviesť všetky jadrové funkcie?
- ✅ Vysvetliť hodnotovú propóziu?
- ✅ Zvládnuť otázky zákazníkov?
- ✅ Riešiť bežné problémy?
- ✅ Predstaviť cenovanie s istotou?

**Ak áno na všetko, ste pripravení na prezentácie zákazníkom! 🎉**

---

**Veľa šťastia s vaším tréningom!**

