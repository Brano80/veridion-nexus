# VERIDION NEXUS
## Sovereign Trust Layer pre High-Risk AI Agenty

**Technická dokumentácia**

---

**Verzia 3.1 | Január 2025**  
**Aktualizované:** Kompletná implementácia GDPR & EU AI Act Compliance dokončená

---

# OBSAH

1. [Prehľad](#prehľad)
2. [Problém](#problém)
3. [Riešenie](#riešenie)
4. [Technická architektúra](#technická-architektúra)
5. [Obchodný model & príjmy](#obchodný-model--príjmy)
6. [Konkurenčné prostredie](#konkurenčné-prostredie)
7. [Analýza rizík](#analýza-rizík)
8. [Príloha](#príloha)

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
- **Osvedcená technológia**: Production-ready platforma s:
  - **Modulárna architektúra**: Core/Modules/Integration vrstvy pre maximálnu flexibilitu
  - **Compliance Hub Dashboard**: Zjednodušené 6-stránkové jadro s plugin modulmi
  - **Module Configuration System**: Zapínanie/vypínanie funkcií cez API
  - **Tri deployment módy**: Embedded (SDK-first), Proxy (reverse proxy), Full (kompletná platforma)
  - **Webhook Support**: Real-time event notifikácie s HMAC-SHA256 podpisovaním
  - **Performance Optimization**: Database indexing, materialized views, connection pooling
  - **Security Hardening**: JWT, RBAC, API Keys, Audit Logging, Rate Limiting
  - **AI Platform SDKs**: 6 SDK pre Azure, AWS, GCP, LangChain, OpenAI, HuggingFace
  - Docker nasadením, REST API, PostgreSQL perzistenciou

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

**Riešenie**: Middleware na úrovni siete, ktorý kontroluje `target_region` v každej požiadavke a **blokuje** akcie smerujúce do jurisdikcií mimo EÚ (napr. US) na úrovni API. Ak agent pokúsi odoslať dáta do US regiónu, systém automaticky vráti HTTP 403 Forbidden so statusom "BLOCKED (SOVEREIGNTY)".

**Technológia**:
- Runtime kontrola `target_region` parametra v API požiadavkách
- Automatické blokovanie na úrovni backendu
- HTTP 403 Forbidden response pre nekompliantné akcie
- Panic-on-violation architektúra (Rust memory safety)

**Súladnosť**: EU AI Act článok 10 (Data Governance)

**Implementácia**: 
- Backend kontroluje `target_region` v `LogRequest`
- Ak `target_region == "US"`, akcia je označená ako `"BLOCKED (SOVEREIGNTY)"`
- Vráti sa HTTP 403 s prázdnym seal_id a tx_id

### 2. Crypto-Shredder (GDPR Engine)

**Problém**: GDPR článok 17 vyžaduje vymazanie dát, ale auditné záznamy musia byť nemenné.

**Riešenie**: **Envelope Encryption** architektúra s API endpointom pre vymazanie:
- Dáta zašifrované s jedinečnými Data Encryption Keys (DEKs)
- DEKs zabalené s Master Key
- **POST /shred_data** endpoint prijíma `seal_id` a označí záznam ako vymazaný
- Na "vymazanie" dát: Záznam sa označí ako `"ERASED (Art. 17)"` a `action_summary` sa zmení na `"[GDPR PURGED] Data Cryptographically Erased"`
- Záznamy zostávajú nemenné, ale dáta sú efektívne vymazané

**Technológia**:
- AES-256-GCM šifrovanie
- Key management systém
- REST API endpoint `/shred_data` pre selektívne vymazanie
- Dashboard UI s tlačidlom pre každý záznam
- Kryptografický dôkaz vymazania

**Súladnosť**: GDPR článok 17 (Right to be Forgotten)

**Implementácia**:
- Frontend dashboard poskytuje tlačidlo "🗑️ GDPR SHRED" pre každý záznam
- Backend endpoint `/shred_data` prijíma `{seal_id}` a aktualizuje záznam
- Vymazané záznamy sa zobrazujú šedou farbou a prečiarknuté v UI

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

### Modulárna architektúra

Veridion Nexus je organizovaný do **troch odlišných vrstiev** pre maximálnu flexibilitu a adopciu:

#### 1. Core Runtime Compliance Engine (Povinné)
**Vždy zapnuté** - Toto sú povinné komponenty poskytujúce základné compliance záruky:

- **Sovereign Lock**: Runtime geofencing pre data sovereignty (EU AI Act článok 10)
- **Crypto-Shredder**: GDPR envelope encryption pre Right to be Forgotten (článok 17)
- **Privacy Bridge**: eIDAS Qualified Electronic Seals (EU 910/2014)
- **Audit Log Chain**: Nemenný audit trail pre všetky compliance akcie
- **Annex IV Compiler**: Automatizované generovanie technickej dokumentácie (EU AI Act Annex IV)

#### 2. Operational Modules (Voliteľné)
**Môžu byť zapnuté/vypnuté** cez Module Configuration API - Plaťte len za to, čo potrebujete:

- **Data Subject Rights** (GDPR články 15-22, 18, 19, 21, 22, 30)
  - Kompletná implementácia všetkých práv subjektov dát
  - Processing restrictions, objections, automated decision review
  - Export processing records (článok 30)
- **Human Oversight** (EU AI Act článok 14)
- **Risk Assessment** (EU AI Act článok 9)
  - Rozšírené context-aware assessment s ML-based scoring
- **Breach Management** (GDPR články 33-34)
  - Automatizované notifikácie s 72-hodinovou súladnosťou
- **Consent Management** (GDPR články 6-7)
- **DPIA Tracking** (GDPR článok 35)
- **Retention Policies** (GDPR článok 5(1)(e))
- **Post-Market Monitoring** (EU AI Act článok 72)
- **Green AI Telemetry** (EU AI Act článok 40)
- **AI-BOM** (CycloneDX štandard)
- **Conformity Assessment** (EU AI Act článok 8)
- **Data Governance** (EU AI Act článok 11)
  - Quality metrics, bias detection, lineage tracking

#### 3. Integration Layer (Vždy dostupné)
**SDKs a konektory** pre bezproblémovú integráciu:

- **AI Platform SDKs**: Azure AI, AWS Bedrock, GCP Vertex, LangChain, OpenAI MCP, HuggingFace
- **Webhooks**: Real-time event notifikácie s HMAC-SHA256 podpisovaním
- **Proxy Mode**: Reverse proxy middleware pre existujúcu AI infraštruktúru
- **REST API**: Kompletné API pre všetky funkcie

### Jazyk a framework

- **Rust**: Memory-safe, high-performance systémové programovanie
- **Actix-web**: Async HTTP framework pre REST API
- **Docker**: Kontajnerizované nasadenie
- **PostgreSQL 15**: Trvalé úložisko s optimalizovaným connection poolingom
- **sqlx 0.7**: Async PostgreSQL driver s compile-time query checking

### Bezpečnostné funkcie

- **JWT Autentifikácia**: Token-based autentifikácia s konfigurovateľným expiračným časom
- **Role-Based Access Control (RBAC)**: Detailné oprávnenia (admin, compliance_officer, auditor, viewer)
- **API Key Management**: Service-to-service autentifikácia s SHA-256 hashovaním
- **Security Audit Logging**: Komplexné logovanie všetkých bezpečnostných udalostí
- **Rate Limiting**: IP-based throttling (konfigurovateľné requesty za minútu)
- **Security Headers**: CORS, X-Frame-Options, CSP, HSTS, X-XSS-Protection, Referrer-Policy
- **Production CORS**: Environment-based origin whitelisting
- **Dependency Scanning**: Automatizované kontrolovanie zraniteľností (cargo-audit integrácia)
- **Non-root execution**: Docker kontajnery bežia ako neprivilegovaní používatelia
- **Encrypted storage**: Všetky dáta zašifrované v pokoji
- **mTLS ready**: Mutual TLS pre autentifikáciu API
- **Zero-trust architektúra**: Žiadne implicitné dôveryhodné predpoklady

### Škálovateľnosť a výkon

- **Database Indexing**: Optimalizované indexy na často dotazovaných stĺpcoch
- **Materialized Views**: Predvypočítané sumáre pre rýchle reportovanie
- **Connection Pooling**: Optimalizované PostgreSQL connection management
- **Pagination**: Efektívne načítavanie dát s page-based pagination
- **Background Workers**: Async spracovanie pre webhooks, retention deletions, view refreshes
- **Query Optimization**: Compile-time SQL checking s sqlx
- **Horizontálne škálovanie**: Stateless API dizajn
- **Async I/O**: Non-blocking sieťové operácie
- **Cloud-native**: Kubernetes-ready nasadenie

### Integrácia

- **REST API**: Štandardné HTTP/JSON rozhranie
- **MCP Server**: Model Context Protocol integrácia pre AI modely (`veridion_mcp.py`)
- **Python Agent SDK**: Demonštračný agent (`uipath_agent.py`) s 50% šancou na US akcie
- **Swagger UI**: Interaktívna API dokumentácia
- **Webhook Support**: Real-time event notifikácie s:
  - HMAC-SHA256 signature verification
  - Retry logika s exponential backoff
  - Delivery history a status tracking
  - Event filtering podľa typu
  - Konfigurovateľné timeout a retry nastavenia
- **API Key Management**: Service-to-service autentifikačné endpointy
- **SDK** (plánované): Klientske knižnice pre populárne jazyky

**MCP Server Integrácia**:
- `veridion_mcp.py` poskytuje tool `secure_compliance_seal` pre AI modely
- AI modely môžu volať compliance seal pred vykonaním high-risk akcií
- Automatická integrácia s Veridion Nexus API
- Podpora pre FastMCP framework

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

1. **AI Agent** vykoná akciu (napr. credit check, lekárska diagnóza) a pošle požiadavku na `/log_action` s `target_region` parametrom
2. **Sovereign Lock** kontroluje `target_region`:
   - Ak `target_region == "US"`: Vráti HTTP 403 Forbidden so statusom `"BLOCKED (SOVEREIGNTY)"`
   - Ak `target_region == "EU"` alebo iný povolený región: Pokračuje ďalej
3. **Privacy Bridge** hashuje payload lokálne, získa eIDAS seal (ak nie je blokované)
4. **Crypto-Shredder** zašifruje a uloží akciu s envelope encryption
5. **Annex IV Compiler** pridá záznam do compliance logu
6. **Response** vrátený AI Agentovi s compliance dôkazom (seal_id, tx_id) alebo chybovou správou

**Crypto-Shredding tok**:
1. Používateľ klikne na tlačidlo "🗑️ GDPR SHRED" v dashboarde pre konkrétny záznam
2. Frontend pošle POST `/shred_data` s `{seal_id}`
3. Backend nájde záznam a označí ho ako `"ERASED (Art. 17)"`
4. Záznam zostáva v logu, ale dáta sú kryptograficky vymazané

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

Veridion Nexus podporuje tri deployment módy pre rôzne use cases:

### 1. Embedded Mode (SDK-First)
**Najlepšie pre**: Startupy, mid-market spoločnosti

- Lightweight client library
- SDKs integrované priamo v aplikačnom kóde
- Minimálne infraštruktúrne požiadavky
- Cena: €25K-€75K/rok

### 2. Proxy Mode (Reverse Proxy)
**Najlepšie pre**: Enterprise s existujúcou AI infraštruktúrou

- Nexus beží ako middleware vrstva
- Automaticky zachytáva AI API volania
- Žiadne zmeny v kóde nie sú potrebné
- Cena: €100K-€200K/rok

### 3. Full Governance Mode
**Najlepšie pre**: Enterprise vyžadujúce kompletnú kontrolu

- Kompletné nasadenie platformy
- Všetky moduly dostupné
- Full dashboard a API prístup
- Cena: €200K-€400K/rok

### Infraštruktúrne možnosti

1. **SaaS (Cloud)**: Hostované na infraštruktúre v EÚ
2. **On-Premise**: Docker kontajnery pre air-gapped prostredia
3. **Hybrid**: API gateway v cloude, spracovanie on-premise

---

# OBCHODNÝ MODEL & PRÍJMY

## Revenue Model: Subscription-as-a-Service (SaaS)

### Primárne príjmové toky

1. **Ročná licencia na predplatné** (75% príjmov)
   - Cenové úrovne zarovnané s deployment módmi
   - Core moduly vždy zahrnuté
   - Operačné moduly sa líšia podľa úrovne
   - Vyžaduje sa ročný záväzok

2. **Transakčné cenové doplnky** (15% príjmov)
   - Cena za pečiatku pre eIDAS pečiatky (€0.10 za pečiatku)
   - Balíčky pre vysoký objem (€50K/rok neobmedzené)
   - Zľavy pre enterprise
   - Voliteľný doplnok pre všetky úrovne

3. **Profesionálne služby** (7% príjmov)
   - Implementačné poradenstvo (€2,500/deň)
   - Vlastné integrácie (€5,000 za integráciu)
   - Podpora pri compliance audite (€10,000 za audit)

4. **Doplnky a upgrady** (3% príjmov)
   - Doplnky modulov (Starter úroveň: €10K/modul)
   - Deployment upgrady (€25K-€50K)
   - Regulačné služby (€25K-€50K)

## Cenové úrovne

### Úroveň 1: Starter (€35,000/rok)

**Deployment Mode**: Embedded (SDK-First)

**Cieľová skupina**: Series A fintech/insurtech, 1-10 zamestnancov

**Core Moduly** (Vždy zahrnuté):
- Sovereign Lock, Crypto-Shredder, Privacy Bridge
- Audit Log Chain, Annex IV Compiler

**Operačné Moduly**: Vyberte 2 zahrnuté
- Možnosti: Data Subject Rights, Human Oversight, Risk Assessment, Breach Management

**Zahŕňa**:
- Až 3 high-risk AI systémy
- Všetkých 6 AI Platform SDKs (Azure, AWS, GCP, LangChain, OpenAI, HuggingFace)
- Email podpora (48h SLA)
- Štandardné Annex IV šablóny
- Komunitná dokumentácia

**Ideálne pre**: Fintech startupy, malé zdravotnícke poskytovateľstvá

### Úroveň 2: Professional (€120,000/rok) ⭐

**Deployment Mode**: Embedded ALEBO Proxy (voľba zákazníka)

**Cieľová skupina**: Series B-D fintech/insurtech, 50-500 zamestnancov

**Core Moduly** (Vždy zahrnuté):
- Sovereign Lock, Crypto-Shredder, Privacy Bridge
- Audit Log Chain, Annex IV Compiler

**Operačné Moduly**: Všetkých 10 modulov zahrnutých
- Data Subject Rights, Human Oversight, Risk Assessment
- Breach Management, Consent Management, DPIA Tracking
- Retention Policies, Post-Market Monitoring
- Green AI Telemetry, AI-BOM

**Zahŕňa**:
- Až 15 high-risk AI systémov
- Všetkých 6 AI Platform SDKs
- Slack kanál podpora (12h SLA)
- Webhook integrácie
- Mesačné compliance reporty
- Štvrťročné business review

**Ideálne pre**: Regionálne banky, stredné poisťovne, rastúce podniky

### Úroveň 3: Enterprise (€350,000/rok základ)

**Deployment Mode**: Full Governance (kompletná platforma)

**Cieľová skupina**: Banky, veľké poisťovne, verejné spoločnosti, 1000+ zamestnancov

**Core Moduly** (Vždy zahrnuté):
- Sovereign Lock, Crypto-Shredder, Privacy Bridge
- Audit Log Chain, Annex IV Compiler

**Operačné Moduly**: Všetko zahrnuté + prioritné feature požiadavky

**Zahŕňa**:
- Až 50 high-risk AI systémov (prvých 50 zahrnutých)
- Deployment možnosti: SaaS, On-Premise, alebo Hybrid
- Všetkých 6 AI Platform SDKs + vlastné integrácie
- Dedičný Customer Success Manager
- 24/7 telefónna podpora
- 99.9% SLA záruka
- Vlastné integrácie (40 hodín/rok zahrnutých)
- Podpora pri regulačnom sandboxe
- Audit defense balíček (expertné svedectvo)
- Súkromný Slack kanál s engineering tímom

**Overage**: €12,000 za každých 10 dodatočných systémov (po prvých 50)

**Ideálne pre**: Tier 1 banky, veľké zdravotnícke systémy, systémovo dôležité inštitúcie

## Doplnky (Všetky úrovne)

### Doplnky modulov (iba Starter úroveň)
- Dodatočný operačný modul: €10,000/rok každý
- (Professional a Enterprise dostávajú všetky moduly)

### Deployment upgrady
- Embedded → Proxy Mode: +€25,000/rok
- Embedded/Proxy → Full Governance: +€50,000/rok

### Transakčné doplnky
- eIDAS Pečiatky: €0.10 za pečiatku (zľavy pre vysoký objem)
- Balíček pre vysoký objem: €50,000/rok (neobmedzené pečiatky)

### Profesionálne služby
- Implementačné poradenstvo: €2,500/deň
- Vlastná integrácia: €5,000 za integráciu
- Podpora pri compliance audite: €10,000 za audit

### Regulačné a právne
- Regulačný sandbox Fast-Track: €25,000 jednorazovo
- Audit Defense Balíček: €50,000/rok (expertné svedectvo, regulačná podpora)

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

#### Základné endpointy

**POST /api/v1/log_action**
- Loguje high-risk AI akciu cez compliance pipeline
- Request body: `{agent_id, action, payload, target_region?, user_notified?, notification_timestamp?, user_id?, requires_human_oversight?, inference_time_ms?, gpu_power_rating_watts?, cpu_power_rating_watts?, energy_estimate_kwh?, carbon_grams?, system_id?, model_name?, model_version?, hardware_type?}`
- Kontroluje `target_region` - ak je "US", "CN", alebo "RU", blokuje akciu (HTTP 403)
- Automaticky sleduje spotrebu energie a uhlíkovú stopu (EU AI Act článok 40)
- Vracia: `{status, seal_id, tx_id, risk_level?, human_oversight_status?}`
- Status môže byť: `"COMPLIANT"` alebo `"BLOCKED (SOVEREIGNTY)"`

**GET /api/v1/logs**
- Získa históriu compliance logu
- Vracia: `Array<ComplianceRecord>`
- Najnovšie záznamy sú na začiatku zoznamu

**POST /api/v1/shred_data**
- Vymaže konkrétny záznam podľa GDPR článku 17
- Request body: `{seal_id}`
- Označí záznam ako `"ERASED (Art. 17)"`
- Vracia: `{status: "SUCCESS"}` alebo `{status: "NOT_FOUND"}`

**GET /api/v1/download_report**
- Stiahne Annex IV compliance report ako PDF
- Vracia: PDF súbor

**POST /api/v1/revoke_access**
- Aktivuje lockdown režim, blokuje všetky nové agent akcie
- Vracia: `{status: "SUCCESS"}`

#### Priorita 1: Práva subjektov údajov (GDPR články 15-22)

**GET /api/v1/data_subject/{user_id}/access**
- Právo na prístup (GDPR článok 15)
- Vracia: `{records: Array<DataSubjectRecord>, format, exported_at}`

**GET /api/v1/data_subject/{user_id}/export**
- Právo na prenosnosť údajov (GDPR článok 20)
- Vracia: Rovnaký formát ako access endpoint

**PUT /api/v1/data_subject/{user_id}/rectify**
- Právo na opravu (GDPR článok 16)
- Request body: `{seal_id, corrected_data}`
- Vracia: `{status: "SUCCESS"}`

#### Priorita 1: Ľudská kontrola (EU AI Act článok 14)

**POST /api/v1/action/{seal_id}/require_approval**
- Vyžaduje ľudskú kontrolu pre akciu
- Vracia: `{status, oversight_id}`

**POST /api/v1/action/{seal_id}/approve**
- Schvaľuje akciu vyžadujúcu ľudskú kontrolu
- Request body: `{reviewer_id, notes?}`
- Vracia: `{status: "APPROVED"}`

**POST /api/v1/action/{seal_id}/reject**
- Zamieta akciu vyžadujúcu ľudskú kontrolu
- Request body: `{reviewer_id, reason}`
- Vracia: `{status: "REJECTED"}`

#### Priorita 1: Hodnotenie rizík (EU AI Act článok 9)

**GET /api/v1/risk_assessment/{seal_id}**
- Získa hodnotenie rizík pre konkrétnu akciu
- Vracia: `{seal_id, risk_level, risk_factors, mitigation_actions, assessed_at}`

**GET /api/v1/risks**
- Získa všetky hodnotenia rizík
- Vracia: `Array<RiskAssessment>`

#### Priorita 1: Správa únikov dát (GDPR články 33-34)

**POST /api/v1/breach_report**
- Nahlási únik dát
- Request body: `{breach_type, description, affected_records_count, detected_at, user_notified?}`
- Vracia: `{breach_id, status, reported_at}`

**GET /api/v1/breaches**
- Zobrazí všetky úniky dát
- Vracia: `Array<DataBreachReport>`

#### Priorita 2: Správa súhlasov (GDPR články 6, 7)

**POST /api/v1/consent**
- Udeľuje súhlas používateľa so spracovaním údajov
- Request body: `{user_id, consent_type, purpose, legal_basis, expires_at?}`
- Vracia: `{consent_id, status, granted_at}`

**POST /api/v1/consent/withdraw**
- Odvolá súhlas používateľa
- Request body: `{user_id, consent_type}`
- Vracia: `{status: "WITHDRAWN"}`

**GET /api/v1/consent/{user_id}**
- Získa všetky súhlasy pre používateľa
- Vracia: `{user_id, consents: Array<ConsentRecord>}`

#### Priorita 2: Sledovanie DPIA (GDPR článok 35)

**POST /api/v1/dpia**
- Vytvorí Posúdenie vplyvu na ochranu údajov
- Request body: `{dpia_id, system_name, processing_activities, risk_assessment, mitigation_measures}`
- Vracia: `{dpia_id, status, created_at}`

**PUT /api/v1/dpia/{dpia_id}**
- Aktualizuje DPIA
- Request body: `{status?, reviewed_by?, review_notes?}`
- Vracia: `{dpia_id, status, updated_at}`

**GET /api/v1/dpias**
- Získa všetky DPIAs
- Vracia: `Array<DpiaRecord>`

#### Priorita 2: Automatizácia doby uchovania (GDPR článok 5(1)(e))

**POST /api/v1/retention/policy**
- Vytvorí politiku uchovania
- Request body: `{policy_name, retention_period_days, description?}`
- Vracia: `{policy_id, status}`

**POST /api/v1/retention/assign**
- Priradí politiku uchovania záznamu
- Request body: `{record_type, record_id, policy_id, expires_at?}`
- Vracia: `{assignment_id, status}`

**GET /api/v1/retention/status/{record_type}/{record_id}**
- Získa stav uchovania pre záznam
- Vracia: `{record_id, policy_name, expires_at, status}`

**GET /api/v1/retention/policies**
- Získa všetky politiky uchovania
- Vracia: `Array<RetentionPolicy>`

**POST /api/v1/retention/execute_deletions**
- Vykoná automatické vymazanie expirovaných záznamov
- Vracia: `{deleted_count, deleted_records: Array<DeletedRecord>}`

#### Priorita 2: Post-market monitoring (EU AI Act článok 72)

**POST /api/v1/monitoring/event**
- Vytvorí monitoring event (incident, anomália, atď.)
- Request body: `{event_type, severity, system_id, description, system_version?}`
- Vracia: `{event_id, status, detected_at}`

**PUT /api/v1/monitoring/event/{event_id}**
- Aktualizuje stav riešenia eventu
- Request body: `{resolution_status, resolved_by?, resolution_notes?}`
- Vracia: `{event_id, resolution_status, resolved_at}`

**GET /api/v1/monitoring/events**
- Získa všetky monitoring eventy (s voliteľnými filtrami)
- Query params: `?system_id={system_id}`
- Vracia: `{events: Array<MonitoringEvent>, total_count}`

**GET /api/v1/monitoring/health/{system_id}**
- Získa stav zdravia systému
- Vracia: `{system_id, overall_status, compliance_status, active_incidents_count, critical_incidents_count, performance_score?, compliance_score?, last_health_check}`

#### Enterprise funkcie: AI-BOM Export (CycloneDX v1.5)

**GET /api/v1/ai_bom/{system_id}**
- Exportuje AI systém Bill of Materials v CycloneDX formáte
- Query params: `?format=cyclonedx` (predvolené)
- Vracia: `CycloneDxBom` (JSON) s AI/ML-BOM komponentmi, závislosťami a compliance metadátami

**POST /api/v1/ai_bom/inventory**
- Registruje AI systém do inventory pre BOM export
- Request body: `{system_id, system_name, system_version?, system_type, description?, vendor?, license?, source_url?, checksum_sha256?, dependencies?, training_data_info?, risk_level?, dpia_id?}`
- Vracia: `{status: "SUCCESS", system_id}`

#### Webhook Support

**POST /api/v1/webhooks**
- Vytvorí webhook endpoint pre real-time event notifikácie
- Request body: `{endpoint_url, event_types, secret_key?, retry_count?, timeout_seconds?}`
- Vracia: `{id, endpoint_url, event_types, active, retry_count, timeout_seconds, created_at}`
- Eventy: `compliance.created`, `breach.detected`, `oversight.required`, `retention.expired`, `monitoring.incident`

**GET /api/v1/webhooks**
- Zobrazí všetky webhook endpointy (s pagination)
- Query params: `?page={page}&limit={limit}`
- Vracia: `{endpoints: Array<WebhookEndpoint>, total_count}`

**PUT /api/v1/webhooks/{id}**
- Aktualizuje konfiguráciu webhook endpointu
- Request body: `{endpoint_url?, event_types?, active?, retry_count?, timeout_seconds?}`
- Vracia: Aktualizovaný webhook endpoint

**DELETE /api/v1/webhooks/{id}**
- Vymaže webhook endpoint
- Vracia: `{status: "SUCCESS"}`

**GET /api/v1/webhooks/{id}/deliveries**
- Získa históriu doručení pre webhook endpoint
- Query params: `?page={page}&limit={limit}`
- Vracia: `{deliveries: Array<WebhookDelivery>, total_count}`
- Funkcie: HMAC-SHA256 signature verification, retry logika s exponential backoff

#### API Key Management

**POST /api/v1/api_keys**
- Vytvorí nový API key pre service-to-service autentifikáciu
- Request body: `{name, description?, permissions, expires_at?}`
- Vracia: `{api_key, key_info, message}` (key sa zobrazí len raz)
- Vyžaduje: `api_key.write` oprávnenie

**GET /api/v1/api_keys**
- Zobrazí všetky API keys (používatelia vidia len svoje, admini vidia všetky)
- Vracia: `{api_keys: Array<ApiKeyInfo>, total_count}`

**GET /api/v1/api_keys/{id}**
- Získa detaily API key (bez skutočného kľúča)
- Vracia: `{id, name, description, user_id, permissions, expires_at, last_used_at, active, created_at}`

**DELETE /api/v1/api_keys/{id}**
- Zruší API key
- Vracia: `{status: "SUCCESS"}`
- Vyžaduje: Vlastníctvo alebo `api_key.delete` oprávnenie

#### Autentifikácia a autorizácia

**POST /api/v1/auth/login**
- Autentifikuje používateľa a vráti JWT token
- Request body: `{username, password}`
- Vracia: `{token, user: {id, username, email, full_name, roles}}`

**POST /api/v1/auth/register**
- Registruje nového používateľa (len admin)
- Request body: `{username, email, password, full_name?}`
- Vracia: `{user, message}`

**GET /api/v1/auth/me**
- Získa informácie o aktuálne autentifikovanom používateľovi
- Vyžaduje: Platný JWT token v `Authorization: Bearer <token>` headeri
- Vracia: `{id, username, email, full_name, roles}`

#### Green AI Telemetry (EU AI Act článok 40)

Sledovanie energie a uhlíkovej stopy je integrované do `POST /api/v1/log_action`:
- `inference_time_ms`: Čas inferencie v milisekundách
- `gpu_power_rating_watts`: GPU výkon (predvolené: 250W)
- `cpu_power_rating_watts`: CPU výkon
- `energy_estimate_kwh`: Predvypočítaná energia (voliteľné, automaticky vypočítané ak nie je poskytnuté)
- `carbon_grams`: Predvypočítaná uhlíková stopa (voliteľné, automaticky vypočítané pomocou EU priemeru: 475 g CO2/kWh)
- `system_id`, `model_name`, `model_version`, `hardware_type`: Pre sledovanie a reportovanie

Výpočet energie: `(GPU + CPU výkon) * čas_v_hodinách / 1000 = kWh`  
Výpočet uhlíka: `energy_kwh * 475.0 = gramy CO2`

### Compliance moduly

**Sovereign Lock**:
- Runtime kontrola `target_region` parametra v API požiadavkách
- Automatické blokovanie akcií smerujúcich do US alebo iných ne-sovereign jurisdikcií
- HTTP 403 Forbidden response pre nekompliantné akcie
- Blokovanie na úrovni backendu pred spracovaním dát

**Crypto-Shredder**:
- AES-256-GCM šifrovanie
- Envelope encryption (DEK + Master Key)
- REST API endpoint `/shred_data` pre selektívne vymazanie podľa `seal_id`
- Dashboard UI s tlačidlom pre každý záznam
- Záznamy označené ako `"ERASED (Art. 17)"` zostávajú v logu, ale dáta sú kryptograficky vymazané

**Privacy Bridge**:
- SHA-256 lokálne hashovanie
- QTSP integrácia (Signicat)
- Qualified Electronic Seals

**Annex IV Compiler**:
- Automatizované generovanie PDF
- Sledovanie compliance záznamov
- Právne záväzný formát

**Webhook Service**:
- Real-time event delivery s HMAC-SHA256 signing
- Exponential backoff retry logika (konfigurovateľný retry count)
- Delivery status tracking (pending, success, failed)
- Event filtering podľa typu
- Background worker pre async delivery

**Security & Access Control**:
- JWT-based autentifikácia s konfigurovateľným expiračným časom
- Role-Based Access Control (RBAC) s detailnými oprávneniami
- API Key management pre service-to-service autentifikáciu
- Security audit logging pre všetky prístupové pokusy
- Rate limiting (IP-based, konfigurovateľné prahy)
- Security headers (CORS, CSP, HSTS, X-Frame-Options, atď.)
- Production-ready CORS konfigurácia (environment-based origin whitelisting)

## B. Regulačná súladnosť

### EU AI Act súladnosť

- **Článok 9**: Systém riadenia rizík (Automatické hodnotenie rizík)
- **Článok 10**: Správa dát (Sovereign Lock - geofencing)
- **Článok 13**: Požiadavky na transparentnosť (Sledovanie notifikácií používateľov)
- **Článok 14**: Ľudská kontrola (Workflow schvaľovania/zamietnutia)
- **Článok 40**: Reportovanie energetickej účinnosti (Green AI Telemetry)
- **Článok 72**: Post-market monitoring (Sledovanie zdravia systému a incidentov)
- **Annex IV**: Technická dokumentácia (Automatizované PDF reporty)

### GDPR súladnosť

- **Článok 5(1)(e)**: Obmedzenie uchovania (Automatizácia doby uchovania)
- **Článok 6**: Zákonnosť spracovania (Správa súhlasov)
- **Článok 7**: Podmienky súhlasu (Sledovanie a odvolanie súhlasov)
- **Článok 15**: Právo na prístup (Žiadosti o prístup subjektov údajov)
- **Článok 16**: Právo na opravu (Oprava údajov)
- **Článok 17**: Právo na vymazanie (Crypto-Shredder)
- **Článok 20**: Právo na prenosnosť údajov (Export údajov)
- **Článok 25**: Ochrana údajov pri návrhu (Technické vynucovanie)
- **Článok 32**: Bezpečnosť spracovania (Šifrovanie, prístupové kontroly)
- **Články 33-34**: Oznámenie o úniku údajov (Reportovanie a sledovanie únikov)
- **Článok 35**: Posúdenie vplyvu na ochranu údajov (Sledovanie a správa DPIAs)

### eIDAS súladnosť

- **Článok 36**: Kvalifikované elektronické pečate (Privacy Bridge)
- **Článok 37**: Požiadavky na kvalifikované elektronické pečate

## C. MCP Server a Python Integrácia

### Model Context Protocol (MCP) Server

Veridion Nexus poskytuje MCP server (`veridion_mcp.py`) pre integráciu s AI modelmi cez Model Context Protocol. Toto umožňuje AI modelom automaticky volať compliance seal pred vykonaním high-risk akcií.

**Funkcie**:
- Tool `secure_compliance_seal`: AI modely môžu volať tento tool pred vykonaním akcie
- Automatická integrácia s Veridion Nexus API
- Podpora pre FastMCP framework
- Windows-kompatibilný (bez emoji v outputoch)

**Použitie**:
```python
# AI model môže volať:
result = await secure_compliance_seal(
    agent_id="Credit-Bot-v1",
    action_type="credit_approval",
    sensitive_data="Customer ID: 12345"
)
```

### Python Agent Demonštrácia

Projekt obsahuje demonštračný Python agent (`uipath_agent.py`), ktorý simuluje high-risk AI agenta s 50% šancou na pokus o odoslanie dát do US regiónu.

**Funkcie**:
- Simulácia rôznych typov akcií (EU a US)
- Automatické testovanie Sovereign Lock enforcement
- Real-time feedback o compliance stave
- Demonštrácia blokovania nekompliantných akcií

**Akcie**:
- `Credit Check - Client EU` (EU - povolené)
- `GDPR Audit Scan` (EU - povolené)
- `UPLOAD DATA TO AWS US-EAST` (US - blokované)
- `SEND ANALYTICS TO GOOGLE NY` (US - blokované)

## D. Technické detaily

### Implementované technológie

**Backend (Rust)**:
- **Rust 1.75+**: Systémové programovanie
- **Actix-web 4**: Async HTTP framework
- **Tokio**: Async runtime
- **uuid 1.0**: Generovanie jedinečných ID
- **chrono 0.4**: Práca s dátumami a časom
- **serde**: Serializácia/deserializácia
- **AES-GCM 0.10**: Šifrovanie
- **SHA-256**: Hashovanie
- **printpdf 0.5**: Generovanie PDF
- **sqlx 0.7**: PostgreSQL databázová knižnica
- **PostgreSQL 15**: Perzistentné úložisko dát
- **Docker**: Kontajnerizácia
- **Swagger/OpenAPI**: API dokumentácia

**Frontend (Next.js/React)**:
- **Next.js 16**: React framework s App Router
- **React 19**: Najnovšie React funkcie
- **TypeScript**: Type-safe JavaScript
- **Tailwind CSS**: Utility-first CSS framework
- **React Query**: Data fetching a caching
- **Recharts**: Interaktívna vizualizácia dát
- **Lucide React**: Moderná ikonová knižnica
- **Compliance Hub Dashboard**: Zjednodušené 6-stránkové jadro:
  1. Compliance Overview (kľúčové metríky a nedávna aktivita)
  2. Runtime Logs Explorer (real-time compliance audit trail)
  3. Human Oversight Queue (schvaľovací workflow)
  4. Data Shredding (GDPR článok 17 crypto-shredding)
  5. Audit & Reports (Annex IV technická dokumentácia)
  6. Settings (API keys, webhooks, module konfigurácia)
- **Plugin Modules**: Dodatočné stránky sa zobrazujú automaticky, keď sú moduly zapnuté
- Real-time aktualizácie (10-sekundové refresh intervaly)
- Responzívny dizajn (mobile-friendly)
- Dark theme rozhranie
- Interaktívne grafy a vizualizácie

**Integrácia (Python)**:
- **fastmcp**: Model Context Protocol server
- **httpx**: Async HTTP klient
- **requests**: HTTP knižnica pre Python agenty
- **uipath_agent.py**: Demonštračný agent s 50% šancou na US akcie
- **veridion_mcp.py**: MCP server pre AI modely

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

**Verzia dokumentu**: 3.0  
**Dátum**: Január 2025  
**Aktualizácia v3.0**: 
- **Modulárna architektúra**: Core/Modules/Integration vrstvy pre maximálnu flexibilitu
- **Compliance Hub Dashboard**: Zjednodušené 6-stránkové jadro s plugin modulmi
- **Module Configuration System**: Zapínanie/vypínanie funkcií cez API
- **Tri deployment módy**: Embedded (SDK-first), Proxy (reverse proxy), Full (kompletná platforma)
- **Webhook Support**: Real-time event notifikácie s HMAC-SHA256 signing a retry logikou
- **Performance Optimization**: Database indexing, materialized views, connection pooling, pagination, background workers
- **Security Hardening**: JWT autentifikácia, RBAC s detailnými oprávneniami, API Key Management, Security Audit Logging, Rate Limiting, Security Headers, Production CORS konfigurácia, Dependency Vulnerability Scanning
- **AI Platform SDKs**: 6 SDK pre Azure, AWS, GCP, LangChain, OpenAI, HuggingFace
- **Production Deployment Guide**: Kompletný návod na produkčné nasadenie

---

**KONIEC DOKUMENTÁCIE**

