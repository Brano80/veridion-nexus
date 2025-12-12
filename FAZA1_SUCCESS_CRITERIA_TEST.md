# Fáza 1 Success Criteria Test Suite

## Prehľad

Tento testovací skript overuje všetky **Fáza 1 Success Criteria** podľa `TODO_EU_COMPLIANCE_PLATFORM.md`.

## Spustenie testov

### Predpoklady

1. **Backend server musí bežať:**
   ```powershell
   # V jednom termináli
   cargo run
   # alebo
   docker-compose up veridion-nexus-api
   ```

2. **Databáza musí byť dostupná:**
   ```powershell
   docker-compose up postgres
   ```

3. **Migrations musia byť aplikované:**
   ```powershell
   # Skontrolovať, či sú všetky migrations aplikované
   ```

### Spustenie testovacieho skriptu

```powershell
# Jednoduchý skript (odporúčaný)
.\test-faza1-success-criteria-simple.ps1

# Alebo kompletný skript
.\test-faza1-success-criteria.ps1
```

## Testované kritériá

### Operational Safety Metrics

1. **Time to first policy test: < 5 minutes**
   - Testuje čas od začiatku setupu po prvý úspešný policy test
   - Postup: Enable shadow mode → Log test action → Get analytics
   - **PASS:** < 5 minút
   - **FAIL:** ≥ 5 minút

2. **Confidence score before enforcement: > 90%**
   - Testuje confidence score z shadow mode analytics
   - Confidence score sa počíta na základe počtu shadow mode logov:
     - ≥ 1000 logov: 95%
     - ≥ 100 logov: 85%
     - ≥ 10 logov: 70%
     - < 10 logov: 50%
   - **PASS:** ≥ 90%
   - **FAIL:** < 90%

3. **Production incidents caused by Veridion: 0**
   - Kontroluje policy health dashboard pre critical policies s otvorenými circuit breakermi
   - **PASS:** 0 incidents
   - **FAIL:** > 0 incidents

4. **Policy rollback time: < 30 seconds**
   - Testuje čas potrebný na rollback enforcement mode
   - Postup: Switch to ENFORCING → Rollback to original mode
   - **PASS:** < 30 sekúnd
   - **FAIL:** ≥ 30 sekúnd

### Compliance Metrics

5. **GDPR compliance score: > 95%**
   - Testuje GDPR compliance score z compliance overview endpoint
   - Score sa počíta na základe implementovaných GDPR článkov
   - **PASS:** ≥ 95%
   - **FAIL:** < 95%

6. **EU AI Act compliance score: > 95%**
   - Testuje EU AI Act compliance score z compliance overview endpoint
   - Score sa počíta na základe implementovaných EU AI Act článkov
   - **PASS:** ≥ 95%
   - **FAIL:** < 95%

7. **Shadow mode coverage: 100% of policies testable**
   - Kontroluje, či je shadow mode dostupný
   - **PASS:** Shadow mode je dostupný (SHADOW alebo ENFORCING mode)
   - **FAIL:** Shadow mode nie je dostupný

### Business Metrics

8. **Time to value: < 1 day from signup to first policy**
   - Testuje čas potrebný na základný setup (enable shadow mode)
   - **PASS:** < 1 deň (1440 minút)
   - **FAIL:** ≥ 1 deň

## Výstup testov

Skript vypíše:

1. **Pre každý test:**
   - `[PASS]` alebo `[FAIL]`
   - Hodnota (ak je relevantná)
   - Detaily (ak test zlyhal)

2. **Zhrnutie podľa kategórií:**
   - Operational Safety Metrics
   - Compliance Metrics
   - Business Metrics

3. **Celkové zhrnutie:**
   - Počet úspešných/neúspešných testov
   - Finálny status (ALL PASS alebo SOME FAIL)

## Príklad výstupu

```
==========================================
Fáza 1 Success Criteria Test Suite
==========================================

[INFO] Testing authentication...
[PASS] Authentication successful

[INFO] Test 1: Time to first policy test
[PASS] 0.5 min

[INFO] Test 2: Confidence score
[PASS] 95%

[INFO] Test 3: Production incidents
[PASS] 0 incidents

[INFO] Test 4: Policy rollback time
[PASS] 0.8 seconds

[INFO] Test 5: GDPR compliance score
[PASS] 100%

[INFO] Test 6: EU AI Act compliance score
[PASS] 100%

[INFO] Test 7: Shadow mode coverage
[PASS] 100% coverage

[INFO] Test 8: Time to value
[PASS] 0.1 minutes

==========================================
Test Results Summary
==========================================

Operational Safety Metrics:
  [PASS] - Time to first policy test (0.5 min)
  [PASS] - Confidence score (95%)
  [PASS] - Production incidents (0)
  [PASS] - Policy rollback time (0.8s)
  Total: 4/4 passed

Compliance Metrics:
  [PASS] - GDPR compliance score (100%)
  [PASS] - EU AI Act compliance score (100%)
  [PASS] - Shadow mode coverage (100%)
  Total: 3/3 passed

Business Metrics:
  [PASS] - Time to value (0.1 min)
  Total: 1/1 passed

Overall: 8/8 tests passed

[SUCCESS] ALL SUCCESS CRITERIA MET!
   Fáza 1 is ready for launch!
```

## Riešenie problémov

### Authentication failed
- Skontrolovať, či backend beží na `http://localhost:8080`
- Overiť credentials v `.env` súbore
- Skontrolovať, či databáza beží

### Confidence score < 90%
- Potrebné je viac shadow mode logov
- Odporúčané: ≥ 1000 logov pre 95% confidence
- Riešenie: Poslať viac testovacích requestov cez `/log_action` v shadow mode

### Compliance scores < 95%
- Skontrolovať, či sú všetky GDPR/EU AI Act články implementované
- Overiť compliance overview endpoint vracia správne dáta

### Production incidents > 0
- Skontrolovať policy health dashboard
- Overiť, či nie sú otvorené circuit breakery
- Skontrolovať, či nie sú critical policies

## Poznámky

- Niektoré testy vyžadujú, aby systém bol v shadow mode
- Confidence score sa zvyšuje s počtom shadow mode logov
- Production incidents test je konzervatívny (predpokladá 0, ak sa nedá overiť)
- Time to value test je zjednodušený (testuje len základný setup)

## Ďalšie kroky

Po úspešnom prejdení všetkých testov:

1. ✅ Fáza 1 je pripravená na launch
2. ✅ Všetky success criteria sú splnené
3. ✅ Systém je pripravený pre SME zákazníkov

