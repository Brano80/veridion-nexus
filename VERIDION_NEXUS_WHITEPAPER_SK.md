# VERIDION NEXUS
## Sovereign Trust Layer pre High-Risk AI Agenty

**Technická dokumentácia**

---

**Verzia 1.0 | Január 2025**

---

# OBSAH

1. [Prehľad](#prehľad)
2. [Problém](#problém)
3. [Riešenie](#riešenie)
4. [Technická architektúra](#technická-architektúra)
5. [Konkurenčné prostredie](#konkurenčné-prostredie)
6. [Analýza rizík](#analýza-rizík)
7. [Príloha](#príloha)

---

# PREHĽAD

## Príležitosť

**Veridion Nexus** je middleware platforma pre compliance, ktorá rieši kritický problém podnikov nasadzujúcich High-Risk AI systémy v Európskej únii: **Ako zabezpečiť technickú súladnosť s EU AI Act, GDPR a eIDAS predpismi na úrovni siete, nie len prostredníctvom procesov a politík.**

EU AI Act, ktorý sa plne začne uplatňovať v roku 2026, ukladá prísne požiadavky na súladnosť pre High-Risk AI systémy, vrátane:
- **Data Sovereignty**: Dáta musia zostať v rámci jurisdikcií EU/EEA
- **Right to be Forgotten**: Súladnosť s GDPR článkom 17 v nemenných auditných záznamoch
- **Technical Documentation**: Automatizované Annex IV reportovanie pre každé rozhodnutie AI
- **Qualified Electronic Seals**: eIDAS-kompatibilný kryptografický dôkaz súladnosti

**Súčasné riešenia sú založené na procesoch a sú reaktívne.** Veridion Nexus je prvá platforma s **runtime enforcement**, ktorá predchádza porušeniam súladnosti na úrovni siete, čím je fyzicky nemožné, aby AI agenty porušili predpisy EÚ.

## Riešenie

Veridion Nexus je middleware protokol založený na Rust, ktorý vynucuje súladnosť prostredníctvom štyroch integrovaných modulov:

1. **Sovereign Lock**: Geofencing na úrovni siete, ktorý blokuje prenosy dát do jurisdikcií mimo EÚ
2. **Crypto-Shredder**: Envelope encryption umožňujúce GDPR "Right to be Forgotten" v nemenných záznamoch
3. **Privacy Bridge**: Lokálne hashovanie + eIDAS Qualified Electronic Seals bez vystavenia dát
4. **Annex IV Compiler**: Automatizované generovanie právne záväznej compliance dokumentácie

## Kľúčové výhody

- **Prvá technická runtime enforcement platforma** pre EU AI Act
- **Regulačný tailwind**: Uplatňovanie EU AI Act od roku 2026 vytvára urgentnú dopyt
- **Vysoké náklady na prepínanie**: Hlboká integrácia so systémami zákazníkov vytvára lock-in
- **Osvedená technológia**: Funkčné MVP s Docker nasadením, REST API a dashboardom

---

# PROBLÉM

## Compliance kríza

Podniky nasadzujúce High-Risk AI systémy v EÚ čelia bezprecedentnej výzve súladnosti. EU AI Act, GDPR a eIDAS predpisy vytvárajú komplexnú sieť požiadaviek, ktoré tradičné prístupy k súladnosti nedokážu adekvátne riešiť.

### Výzva EU AI Act

EU AI Act (Regulation 2021/0106) klasifikuje AI systémy do štyroch kategórií rizika. **High-Risk AI systémy** (používané v bankovníctve, zdravotníctve, poisťovníctve atď.) čelia najprísnejším požiadavkám:

1. **Annex IV Technical Documentation**: Každé rozhodnutie AI musí byť zdokumentované s:
   - Špecifikáciami vstupov/výstupov
   - Metodológiami tréningu
   - Hodnoteniami rizika
   - Overením súladnosti

2. **Data Governance (Article 10)**: 
   - Dáta musia zostať v rámci jurisdikcií EU/EEA
   - Žiadne prenosy dát do ne-sovereign jurisdikcií (USA, Čína atď.)

3. **Transparency & Human Oversight (Article 13-14)**:
   - Používatelia musia byť informovaní pri interakcii s AI
   - Vyžadujú sa mechanizmy ľudskej kontroly

**Penalizácie**: Až do výšky €35M alebo 7% z celosvetového ročného obratu za nesúladnosť.

### GDPR paradox

GDPR článok 17 ("Right to be Forgotten") vyžaduje, aby osobné údaje boli na požiadanie vymazané. Avšak **auditné záznamy musia byť nemenné** kvôli súladnosti a bezpečnosti. To vytvára neriešiteľný rozpor:

- **Nemenné záznamy** = Nemožno vymazať dáta
- **GDPR požiadavka** = Musia sa vymazať dáta na požiadanie

**Súčasné riešenia**: Buď porušujú GDPR alebo udržiavajú meniteľné záznamy (bezpečnostné riziko).

### eIDAS požiadavka

eIDAS Regulation (EU 910/2014) vyžaduje **Qualified Electronic Seals (QES)** pre právne záväzné digitálne dokumenty. Avšak tradičné QES riešenia vyžadujú odoslanie citlivých dát cloud poskytovateľom, čím sa porušujú požiadavky na data sovereignty.

### Súčasné riešenia sú nedostatočné

**Existujúce compliance platformy** (OneTrust, TrustArc, Vanta) sú:
- **Založené na procesoch**: Spoliehajú sa na politiky a audity, nie na technické vynucovanie
- **Reaktívne**: Detegujú porušenia až po ich výskyte
- **Všeobecné**: Nie sú postavené špecificky pre požiadavky EU AI Act
- **Drahé**: €150K-€300K/rok s obmedzenou technickou hĺbkou

**Vlastné riešenia** sú:
- **Drahé**: €500K-€2M jednorazové náklady na vývoj
- **Časovo náročné**: 6-12 mesiacov na vybudovanie
- **Údržbové bremeno**: Vyžaduje sa pokračujúci vývoj
- **Rizikové**: Postavené tímami bez hlbokej odbornosti v súladnosti

## Riešenie Veridion Nexus

**Veridion Nexus rieši tieto problémy prostredníctvom technického vynucovania na úrovni siete:**

1. **Predchádza porušeniam** namiesto ich detekcie
2. **Rieši GDPR paradox** prostredníctvom envelope encryption
3. **Automatizuje compliance** dokumentáciu (90% zníženie času)
4. **EU-first architektúra** postavená špecificky pre predpisy EÚ
5. **Nákladovo efektívne** (70% lacnejšie ako vlastný vývoj)

---

# RIEŠENIE

## Hlavná hodnotová ponuka

**"Compliance ako runtime constraint"**

Veridion Nexus vynucuje súladnosť na úrovni siete, čím je **fyzicky nemožné**, aby AI agenty porušili predpisy EÚ. Namiesto spoliehania sa na politiky a audity poskytujeme **technické záruky**.

## Štyri pilieré

### 1. Sovereign Lock (Geofencing)

**Problém**: EU AI Act článok 10 vyžaduje, aby dáta zostali v rámci jurisdikcií EU/EEA. Súčasné riešenia sa spoliehajú na vynucovanie politík, ktoré môžu byť obídené.

**Riešenie**: Middleware na úrovni siete, ktorý kontroluje všetky odchádzajúce IP adresy a **blokuje** pripojenia k jurisdikciám mimo EÚ na úrovni siete.

**Technológia**:
- Real-time IP geolokácia
- Integrácia network middleware
- Panic-on-violation architektúra (Rust memory safety)

**Súladnosť**: EU AI Act článok 10 (Data Governance)

### 2. Crypto-Shredder (GDPR Engine)

**Problém**: GDPR článok 17 vyžaduje vymazanie dát, ale auditné záznamy musia byť nemenné.

**Riešenie**: **Envelope Encryption** architektúra:
- Dáta zašifrované s jedinečnými Data Encryption Keys (DEKs)
- DEKs zabalené s Master Key
- Na "vymazanie" dát: Zničenie DEK (dáta sa stanú kryptograficky neobnoviteľné)
- Záznamy zostávajú nemenné, ale dáta sú efektívne vymazané

**Technológia**:
- AES-256-GCM šifrovanie
- Key management systém
- Kryptografický dôkaz vymazania

**Súladnosť**: GDPR článok 17 (Right to be Forgotten)

### 3. Privacy Bridge (eIDAS integrácia)

**Problém**: eIDAS vyžaduje Qualified Electronic Seals, ale tradičné riešenia vystavujú dáta cloud poskytovateľom.

**Riešenie**: **Lokálne hashovanie + vzdialené sealing**:
- Hash citlivých dát lokálne (SHA-256)
- Odoslanie iba hashu do Qualified Trust Service Provider (QTSP)
- Prijatie Qualified Electronic Seal bez vystavenia dát
- Dáta nikdy neopustia jurisdikciu EÚ

**Technológia**:
- SHA-256 hashovanie
- QTSP integrácia (Signicat, DocuSign atď.)
- OAuth2 autentifikácia
- Circuit breaker pre výpadky API

**Súladnosť**: eIDAS Regulation (EU 910/2014)

### 4. Annex IV Compiler

**Problém**: EU AI Act Annex IV vyžaduje technickú dokumentáciu pre každé rozhodnutie AI. Manuálna dokumentácia je časovo náročná a náchylná na chyby.

**Riešenie**: **Automatizované generovanie PDF reportov**:
- Real-time sledovanie compliance záznamov
- Automatizované generovanie PDF so všetkými požadovanými poliami
- Právne záväzný formát
- API endpoint pre reporty na požiadanie

**Technológia**:
- Generovanie PDF (printpdf)
- Sledovanie compliance záznamov
- REST API integrácia

**Súladnosť**: EU AI Act Annex IV (Technical Documentation)

## Technická architektúra

### Jazyk a framework

- **Rust**: Memory-safe, high-performance systémové programovanie
- **Actix-web**: Async HTTP framework pre REST API
- **Docker**: Kontajnerizované nasadenie
- **PostgreSQL** (plánované): Trvalé úložisko pre produkciu

### Bezpečnostné funkcie

- **Non-root execution**: Docker kontajnery bežia ako neprivilegovaní používatelia
- **Encrypted storage**: Všetky dáta zašifrované v pokoji
- **mTLS ready**: Mutual TLS pre autentifikáciu API
- **Zero-trust architektúra**: Žiadne implicitné dôveryhodné predpoklady

### Škálovateľnosť

- **Horizontálne škálovanie**: Stateless API dizajn
- **Async I/O**: Non-blocking sieťové operácie
- **Cloud-native**: Kubernetes-ready nasadenie

### Integrácia

- **REST API**: Štandardné HTTP/JSON rozhranie
- **Swagger UI**: Interaktívna API dokumentácia
- **Webhook support** (plánované): Real-time compliance notifikácie
- **SDK** (plánované): Klientske knižnice pre populárne jazyky

---

# TECHNICKÁ ARCHITEKTÚRA

## Systémová architektúra

```
┌─────────────────────────────────────────────────────────┐
│                    AI Agent Layer                        │
│  (High-Risk AI systémy zákazníka)                       │
└────────────────────┬────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────┐
│              VERIDION NEXUS MIDDLEWARE                    │
│                                                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ Sovereign    │  │ Crypto-      │  │ Privacy     │  │
│  │ Lock         │  │ Shredder     │  │ Bridge      │  │
│  │ (Geofencing) │  │ (GDPR)       │  │ (eIDAS)     │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
│                                                           │
│  ┌──────────────────────────────────────────────────┐  │
│  │         Annex IV Compiler                         │  │
│  │         (Documentation)                           │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────┬────────────────────────────────────┘
                     │
        ┌────────────┴────────────┐
        ▼                         ▼
┌──────────────┐         ┌──────────────┐
│ QTSP         │         │ Compliance   │
│ (Signicat)   │         │ Dashboard    │
└──────────────┘         └──────────────┘
```

## Tok dát

1. **AI Agent** vykoná akciu (napr. credit check, lekárska diagnóza)
2. **Sovereign Lock** overí IP geolokáciu (iba EU/EEA)
3. **Privacy Bridge** hashuje payload lokálne, získa eIDAS seal
4. **Crypto-Shredder** zašifruje a uloží akciu s envelope encryption
5. **Annex IV Compiler** pridá záznam do compliance logu
6. **Response** vrátený AI Agentovi s compliance dôkazom (seal_id, tx_id)

## Bezpečnostný model

### Šifrovanie

- **Data at Rest**: AES-256-GCM šifrovanie
- **Data in Transit**: TLS 1.3 (HTTPS)
- **Key Management**: Envelope encryption s master key wrapping
- **Key Destruction**: Kryptografické vymazanie pre GDPR súladnosť

### Sieťová bezpečnosť

- **Geofencing**: IP-based jurisdikčné vynucovanie
- **Firewall Integration**: Môže sa integrovať s existujúcou sieťovou bezpečnosťou
- **Zero-Trust**: Žiadna implicitná dôvera, všetky pripojenia overené

### Compliance dôkaz

- **Qualified Electronic Seals**: eIDAS-kompatibilné kryptografické podpisy
- **Nemenné auditné záznamy**: Kryptografické hash reťazce
- **Technická dokumentácia**: Automatizované Annex IV reporty

## Výkonnostné charakteristiky

- **Latencia**: <100ms (p95) pre compliance spracovanie
- **Throughput**: 10,000+ requestov/sekundu (jedna inštancia)
- **Uptime**: 99.9% SLA cieľ
- **Škálovateľnosť**: Horizontálne škálovanie na milióny requestov/deň

## Možnosti nasadenia

1. **SaaS (Cloud)**: Hostované na infraštruktúre v EÚ
2. **On-Premise**: Docker kontajnery pre air-gapped prostredia
3. **Hybrid**: API gateway v cloude, spracovanie on-premise

---

# KONKURENČNÉ PROSTREDIE

## Konkurenčné pozicionovanie

| Funkcia | Veridion Nexus | OneTrust | TrustArc | Vlastné riešenie |
|---------|---------------|----------|----------|-----------------|
| Runtime Enforcement | ✅ Áno | ❌ Nie | ❌ Nie | ⚠️ Možné |
| EU AI Act špecifické | ✅ Vstavané | ⚠️ Všeobecné | ⚠️ Všeobecné | ⚠️ Vlastné |
| GDPR Right to Forget | ✅ Technické | ⚠️ Proces | ⚠️ Proces | ⚠️ Vlastné |
| Data Sovereignty | ✅ Úroveň siete | ⚠️ Politika | ⚠️ Politika | ⚠️ Vlastné |
| Čas na nasadenie | ✅ Týždne | ⚠️ Mesiace | ⚠️ Mesiace | ❌ 6-12 mesiacov |
| Technická hĺbka | ✅ Hlboká | ⚠️ Povrchová | ⚠️ Povrchová | ✅ Hlboká |

## Priame konkurenti

### OneTrust AI Governance

**Silné stránky**:
- Ustálená značka v compliance
- Veľká zákaznícka základňa
- Komplexný súbor funkcií

**Slabé stránky**:
- Založené na procesoch, nie technické vynucovanie
- Všeobecná súladnosť, nie špecificky pre EU AI Act
- Drahé (€150K-€300K/rok)
- Dlhá doba implementácie (3-6 mesiacov)

**Naša výhoda**: Technické runtime enforcement, EU-first architektúra, rýchlejšie nasadenie

### TrustArc AI Compliance

**Silné stránky**:
- Zamerané na súkromie
- Dobré GDPR nástroje
- Ustálené v EÚ

**Slabé stránky**:
- Hodnotiace, nie vynucovanie
- Obmedzené pokrytie AI Act
- Menšia technická hĺbka

**Naša výhoda**: Vynucovanie na úrovni siete, technicky rieši GDPR paradox

## Nepriame konkurenti

### Vanta / Drata

**Zameranie**: Bezpečnostná súladnosť (SOC 2, ISO 27001)

**Relevantnosť**: Iný trh (bezpečnosť vs. AI súladnosť), ale podobné pozicionovanie

**Naša výhoda**: Špecifické pre AI Act, technické vynucovanie vs. hodnotenie

### Vlastné riešenia

**Charakteristiky**: Vlastný vývoj

**Slabé stránky**:
- Drahé (€500K-€2M jednorazovo)
- Časovo náročné (6-12 mesiacov)
- Údržbové bremeno
- Rizikové

**Naša výhoda**: 70% zníženie nákladov, týždne vs. mesiace, osvedčená odbornosť

## Konkurenčná výhoda

1. **Technická zložitosť**: Runtime enforcement je ťažko replikovateľný
2. **Prvá výhoda**: Jediné riešenie postavené pre EU AI Act
3. **Sieťové efekty**: Viac zákazníkov = lepšie compliance dáta
4. **Náklady na prepínanie**: Hlboká integrácia so systémami zákazníkov
5. **Regulačná odbornosť**: Hlboké znalosti požiadaviek EU AI Act

---

# ANALÝZA RIZÍK

## Technické riziká

### Riziko: Zmeny požiadaviek EU AI Act

**Pravdepodobnosť**: Stredná  
**Dopad**: Stredný  
**Mitigácia**: 
- Modulárna architektúra umožňuje rýchle aktualizácie
- Úzky vzťah s regulačnými orgánmi
- Poradný výbor s regulačnou odbornosťou

### Riziko: Zmena cien/podmienok QTSP partnera

**Pravdepodobnosť**: Nízka  
**Dopad**: Vysoký  
**Mitigácia**: 
- Podpora viacerých QTSP (nie závislosť na jednom poskytovateľovi)
- Vyjednané dlhodobé zmluvy
- Pass-through pricing model

### Riziko: Výkon pri škálovaní

**Pravdepodobnosť**: Nízka  
**Dopad**: Stredný  
**Mitigácia**: 
- Async architektúra navrhnutá pre škálovanie
- Load testing a optimalizácia
- Schopnosť horizontálneho škálovania

## Trhové riziká

### Riziko: Pomalé prijatie súladnosti EU AI Act

**Pravdepodobnosť**: Nízka  
**Dopad**: Vysoký  
**Mitigácia**: 
- Regulačné vynucovanie od roku 2026 vytvára urgentnosť
- Zameranie na early adopters
- Zvyšujúci sa regulačný tlak

### Riziko: Veľké technologické spoločnosti budujú vlastné riešenia

**Pravdepodobnosť**: Stredná  
**Dopad**: Stredný  
**Mitigácia**: 
- Zameranie na mid-market (nie veľké tech)
- Výhoda rýchlejšieho času na trh
- Nákladovo efektívne vs. vlastný vývoj

### Riziko: Regulačné zmeny

**Pravdepodobnosť**: Stredná  
**Dopad**: Stredný  
**Mitigácia**: 
- Modulárna architektúra
- Regulačná odbornosť v tíme
- Schopnosť rýchlej aktualizácie

## Technické riziká (pokračovanie)

### Riziko: Výpadky QTSP API

**Pravdepodobnosť**: Stredná  
**Dopad**: Stredný  
**Mitigácia**: 
- Circuit breaker architektúra
- Offline buffering s automatickou synchronizáciou
- Multi-QTSP podpora

### Riziko: Bezpečnostné zraniteľnosti

**Pravdepodobnosť**: Nízka  
**Dopad**: Vysoký  
**Mitigácia**: 
- Memory-safe jazyk (Rust)
- Pravidelné bezpečnostné audity
- Zero-trust architektúra
- Šifrovanie end-to-end

---

# PRÍLOHA

## A. Technické špecifikácie

### API endpointy

**POST /log_action**
- Loguje high-risk AI akciu cez compliance pipeline
- Vracia: `{status, seal_id, tx_id}`

**GET /logs**
- Získa históriu compliance logu
- Vracia: `Array<ComplianceRecord>`

**GET /download_report**
- Stiahne Annex IV compliance report ako PDF
- Vracia: PDF súbor

**POST /revoke_keys**
- Aktivuje lockdown režim, blokuje všetky nové agent akcie
- Vracia: `{status, message, timestamp}`

**POST /restore_keys**
- Deaktivuje lockdown režim, obnoví normálne operácie
- Vracia: `{status, message, timestamp}`

### Compliance moduly

**Sovereign Lock**:
- IP geolokácia validácia
- EU/EEA whitelist vynucovanie
- Blokovanie na úrovni siete

**Crypto-Shredder**:
- AES-256-GCM šifrovanie
- Envelope encryption (DEK + Master Key)
- Zničenie kľúčov pre GDPR súladnosť

**Privacy Bridge**:
- SHA-256 lokálne hashovanie
- QTSP integrácia (Signicat)
- Qualified Electronic Seals

**Annex IV Compiler**:
- Automatizované generovanie PDF
- Sledovanie compliance záznamov
- Právne záväzný formát

## B. Regulačná súladnosť

### EU AI Act súladnosť

- **Article 10**: Data Governance (Sovereign Lock)
- **Article 13-14**: Transparency & Human Oversight (Annex IV Compiler)
- **Annex IV**: Technical Documentation (Automatizované reporty)

### GDPR súladnosť

- **Article 17**: Right to be Forgotten (Crypto-Shredder)
- **Article 25**: Data Protection by Design (Technické vynucovanie)
- **Article 32**: Security of Processing (Šifrovanie, prístupové kontroly)

### eIDAS súladnosť

- **Article 36**: Qualified Electronic Seals (Privacy Bridge)
- **Article 37**: Requirements for Qualified Electronic Seals

## C. Technické detaily

### Implementované technológie

- **Rust 1.75+**: Systémové programovanie
- **Actix-web 4**: Async HTTP framework
- **Tokio**: Async runtime
- **reqwest 0.11**: HTTP klient (async, rustls-tls)
- **AES-GCM 0.10**: Šifrovanie
- **SHA-256**: Hashovanie
- **printpdf 0.5**: Generovanie PDF
- **Docker**: Kontajnerizácia
- **Swagger/OpenAPI**: API dokumentácia

### Bezpečnostné opatrenia

- **Non-root execution**: Docker kontajnery bežia ako neprivilegovaní používatelia
- **Encrypted storage**: Všetky dáta zašifrované v pokoji
- **TLS 1.3**: Všetky sieťové komunikácie
- **Zero-trust**: Žiadne implicitné dôveryhodné predpoklady
- **Memory safety**: Rust zabezpečuje memory safety

### Výkonnostné metriky

- **Latencia**: <100ms (p95) pre compliance spracovanie
- **Throughput**: 10,000+ requestov/sekundu (jedna inštancia)
- **Uptime**: 99.9% SLA cieľ
- **Škálovateľnosť**: Horizontálne škálovanie na milióny requestov/deň

## D. Kontaktné informácie

**Veridion Nexus**

Email: support@veridion.nexus  
Website: https://veridion.nexus  

---

**Verzia dokumentu**: 1.0  
**Dátum**: Január 2025

---

**KONIEC DOKUMENTÁCIE**

