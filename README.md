# VERIDION NEXUS

**Runtime Compliance for EU AI Act** • December 2027 Deadline

Make EU AI Act violations technically impossible, not just policy violations.

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](LICENSE)

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

---

## The Problem

Traditional compliance = policies + audits → 6-12 months → €260K → **Violations still possible**

Veridion Nexus = 3-line integration → 2 weeks → **Violations technically impossible**

**Target**: Fintech & insurtech companies preparing for December 2027 EU AI Act deadline

---

## Core Value Proposition

**Runtime Enforcement of Compliance** - Instead of relying on policies and audits, Veridion Nexus provides **technical guarantees** that make it physically impossible for AI agents to violate EU regulations.

## Architecture

Veridion Nexus is organized into **three distinct layers** for maximum flexibility and adoption:

### 1. Core Runtime Compliance Engine (Mandatory)

**Always enabled** - These are the mandatory components that provide core compliance guarantees:

- **Sovereign Lock** - Runtime geofencing for data sovereignty (EU AI Act Article 10)
- **Crypto-Shredder** - GDPR envelope encryption for Right to be Forgotten (Article 17)
- **Privacy Bridge** - eIDAS Qualified Electronic Seals (EU 910/2014)
- **Audit Log Chain** - Immutable audit trail for all compliance actions
- **Annex IV Compiler** - Automated technical documentation generation (EU AI Act Annex IV)

### 2. Operational Modules (Optional)

**Can be enabled/disabled** via Module Configuration API - Pay only for what you need:

- **Data Subject Rights** (GDPR Articles 15-22, 18, 19, 21, 22, 30)
  - Complete implementation: Access, Export, Rectification, Erasure
  - Processing Restrictions (Article 18)
  - Processing Objections (Article 21)
  - Automated Decision Review (Article 22)
  - Processing Records Export (Article 30)
- **Human Oversight** (EU AI Act Article 14)
- **Risk Assessment** (EU AI Act Article 9)
  - Enhanced context-aware assessment with ML-based scoring
- **Breach Management** (GDPR Articles 33-34)
  - Automated notifications with 72-hour compliance
- **Consent Management** (GDPR Articles 6-7)
- **DPIA Tracking** (GDPR Article 35)
- **Retention Policies** (GDPR Article 5(1)(e))
- **Post-Market Monitoring** (EU AI Act Article 72)
- **Green AI Telemetry** (EU AI Act Article 40)
- **AI-BOM** (CycloneDX Standard)
- **Conformity Assessment** (EU AI Act Article 8)
- **Data Governance** (EU AI Act Article 11)
  - Quality metrics, bias detection, lineage tracking
- **Notification Service** (GDPR Article 33, EU AI Act Article 13)
  - SMTP email, Twilio SMS, in-app notifications
  - Multi-language support, user preferences

### 3. Integration Layer (Always Available)

**SDKs and connectors** for seamless integration:

- **AI Platform SDKs**: Azure AI, AWS Bedrock, GCP Vertex, LangChain, OpenAI MCP, HuggingFace
- **Webhooks**: Real-time event notifications with HMAC-SHA256 signing
- **Proxy Mode**: Reverse proxy middleware for existing AI infrastructure
- **REST API**: Complete API for all features

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed architecture documentation.

## Quick Start

**Free Community Edition** - Run Veridion Nexus locally with Docker. No license required for development, testing, or internal use.

### Prerequisites

- **Docker** (Recommended - easiest way to get started)
- **Rust 1.70+** (Optional - only needed if building from source)
- **Signicat API Credentials** (Optional - system works in mock mode by default)

### Local Development

1. **Setup environment:**
   ```bash
   # Auto-generate .env with secure random values
   ./setup_env.sh
   
   # Or manually:
   # cp .env.example .env
   # nano .env  # Edit values
   ```

2. **Start the system:**
   ```bash
   docker-compose up --build
   ```

3. **Test it works:**
   ```bash
   ./test_system.sh
   ```

4. **Access the API:**
   - Health: http://localhost:8080/health
   - Swagger: http://localhost:8080/swagger-ui/
   - Dashboard: http://localhost:3000 (run `cd dashboard && npm install && npm run dev`)

**That's it!** You now have a fully functional compliance platform running locally. All core features are available in the Community Edition.

## API Documentation

### Core Endpoints

#### Compliance Logging
- `POST /api/v1/log_action` - Log a high-risk AI action
- `GET /api/v1/logs` - Retrieve compliance log history
- `GET /api/v1/download_report?format=pdf|json|xml` - Download Annex IV report (PDF/JSON/XML)

#### GDPR Compliance (Priority 1 & 3)
- `GET /api/v1/data_subject/{user_id}/access` - Right to access (GDPR Article 15)
- `GET /api/v1/data_subject/{user_id}/export` - Data portability (GDPR Article 20)
- `PUT /api/v1/data_subject/{user_id}/rectify` - Right to rectification (GDPR Article 16)
- `POST /api/v1/shred_data` - Right to be forgotten (GDPR Article 17)
- `POST /api/v1/data_subject/{user_id}/restrict` - Right to restriction (GDPR Article 18)
- `POST /api/v1/data_subject/{user_id}/lift_restriction` - Lift processing restriction
- `GET /api/v1/data_subject/{user_id}/restrictions` - Get processing restrictions
- `POST /api/v1/data_subject/{user_id}/object` - Right to object (GDPR Article 21)
- `POST /api/v1/data_subject/{user_id}/withdraw_objection` - Withdraw objection
- `GET /api/v1/data_subject/{user_id}/objections` - Get processing objections
- `POST /api/v1/automated_decision/{seal_id}/request_review` - Request human review (GDPR Article 22)
- `POST /api/v1/automated_decision/{seal_id}/appeal` - Appeal automated decision
- `GET /api/v1/automated_decisions` - Get automated decisions
- `GET /api/v1/processing_records` - Records of processing activities (GDPR Article 30)
- `GET /api/v1/processing_records/export` - Export processing records (CSV)

#### EU AI Act Compliance (Priority 1 & 3)
- `POST /api/v1/action/{seal_id}/require_approval` - Require human oversight
- `POST /api/v1/action/{seal_id}/approve` - Approve action (Human Oversight)
- `POST /api/v1/action/{seal_id}/reject` - Reject action (Human Oversight)
- `GET /api/v1/risk_assessment/{seal_id}` - Get risk assessment (EU AI Act Article 9)
- `GET /api/v1/risks` - Get all risk assessments
- `POST /api/v1/conformity_assessments` - Create/update conformity assessment (EU AI Act Article 8)
- `GET /api/v1/conformity_assessments` - Get conformity assessments
- `POST /api/v1/data_quality/metrics` - Record data quality metric (EU AI Act Article 11)
- `POST /api/v1/data_quality/bias` - Record data bias detection
- `POST /api/v1/data_quality/lineage` - Record data lineage
- `GET /api/v1/data_quality/report/{seal_id}` - Get data quality report

#### Data Breach Management (Priority 1)
- `POST /api/v1/breach_report` - Report data breach (GDPR Articles 33-34)
- `GET /api/v1/breaches` - List all breaches

#### Notification Service (Priority 1 & 3)
- `POST /api/v1/data_subject/{user_id}/notification_preferences` - Set notification preferences
- `GET /api/v1/data_subject/{user_id}/notification_preferences` - Get notification preferences

#### System Management
- `POST /api/v1/revoke_access` - System lockdown (Kill switch)
- `GET /health` - Health check

#### AI-BOM Export (CycloneDX v1.5)
- `GET /api/v1/ai_bom/{system_id}` - Export AI system Bill of Materials in CycloneDX format
- `POST /api/v1/ai_bom/inventory` - Register AI system in inventory for BOM export

#### Green AI Telemetry (EU AI Act Article 40)
- Energy and carbon tracking integrated into `POST /api/v1/log_action`
  - Fields: `inference_time_ms`, `gpu_power_rating_watts`, `cpu_power_rating_watts`
  - Automatic calculation of `energy_estimate_kwh` and `carbon_grams`
  - EU average grid carbon intensity: 475 g CO2/kWh

### Interactive API Documentation

**Swagger UI** is available at: `http://localhost:8080/swagger-ui/`

## Compliance Features

### GDPR Compliance
- **Article 15** - Right of access
- **Article 16** - Right to rectification
- **Article 17** - Right to erasure ("Right to be Forgotten")
- **Article 18** - Right to restriction of processing
- **Article 19** - Notification of rectification/erasure
- **Article 20** - Right to data portability
- **Article 21** - Right to object
- **Article 22** - Automated decision-making (human review)
- **Article 25** - Data protection by design
- **Article 30** - Records of processing activities
- **Article 32** - Security of processing
- **Article 33-34** - Data breach notification

### EU AI Act Compliance
- **Article 8** - Conformity assessment
- **Article 9** - Risk management system (Enhanced with ML-based scoring)
- **Article 10** - Data governance (Sovereign Lock)
- **Article 11** - Data governance (Quality metrics, bias detection, lineage)
- **Article 13** - Transparency requirements (Notification service)
- **Article 14** - Human oversight
- **Article 40** - Energy efficiency reporting (Green AI Telemetry)
- **Article 72** - Post-market monitoring
- **Annex IV** - Technical documentation (Automated PDF/JSON/XML generation)

### eIDAS Compliance
- **Article 36** - Qualified Electronic Seals
- **Article 37** - Requirements for Qualified Electronic Seals

## Testing

### Run Unit Tests
```bash
cargo test
```

### Run Integration Tests
```bash
cargo test --test integration_test
```

### Test with Python Agent
```bash
python test_agent.py
```

## Configuration

### Environment Variables

```ini
# Master Key for Crypto-Shredding (Required)
VERIDION_MASTER_KEY=your_secure_master_key_here

# eIDAS Sealing Configuration (Optional)
USE_REAL_API=false                    # Set to true for live Signicat API
SIGNICAT_CLIENT_ID=your_client_id     # Signicat OAuth2 Client ID
SIGNICAT_CLIENT_SECRET=your_secret    # Signicat OAuth2 Client Secret
SIGNICAT_TOKEN_URL=https://api.signicat.com/auth/open/connect/token
SIGNICAT_API_URL=https://api.signicat.com/sign/documents

# Notification Service (Optional)
SMTP_HOST=smtp.example.com           # SMTP server hostname
SMTP_PORT=587                        # SMTP port (usually 587 for TLS)
SMTP_USERNAME=your_smtp_username     # SMTP authentication username
SMTP_PASSWORD=your_smtp_password     # SMTP authentication password
SMTP_FROM=noreply@veridion.nexus    # From email address
TWILIO_ACCOUNT_SID=your_account_sid  # Twilio Account SID
TWILIO_AUTH_TOKEN=your_auth_token    # Twilio Auth Token
TWILIO_FROM_NUMBER=+1234567890       # Twilio phone number

# Logging
RUST_LOG=info                         # Log level: trace, debug, info, warn, error
```

## Project Structure

```
veridion-nexus/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── lib.rs                  # Library exports (for testing)
│   ├── api_state.rs            # Application state management
│   ├── routes.rs               # API route handlers
│   ├── compliance_models.rs    # GDPR/EU AI Act data models
│   ├── crypto_shredder.rs      # GDPR Article 17 implementation
│   ├── privacy_bridge.rs       # eIDAS integration
│   └── annex_iv_compiler.rs    # PDF report generation
├── tests/
│   └── integration_test.rs    # Integration tests
├── Cargo.toml                  # Rust dependencies
├── Dockerfile                   # Docker build configuration
├── docker-compose.yml           # Docker Compose setup
├── sdks/                        # AI Platform SDKs
│   ├── azure_ai/               # Azure AI SDK
│   ├── aws_bedrock/            # AWS Bedrock SDK
│   ├── gcp_vertex/             # GCP Vertex AI SDK
│   ├── langchain/              # LangChain SDK
│   ├── openai_mcp/             # OpenAI MCP SDK
│   ├── huggingface/            # HuggingFace SDK
│   └── examples/               # Usage examples
└── README.md                    # This file
```

## Status

### Implemented (MVP Ready)
- Core compliance modules (Sovereign Lock, Crypto-Shredder, Privacy Bridge, Annex IV)
- Priority 1: Data Subject Rights (GDPR Articles 15, 16, 17, 20)
- Priority 1: Human Oversight (EU AI Act Article 14)
- Priority 1: Risk Assessment (EU AI Act Article 9)
- Priority 1: Data Breach Reporting (GDPR Articles 33-34)
- **PostgreSQL persistence** - All data stored in database
- Swagger UI documentation
- Docker deployment with PostgreSQL
- Integration tests with database support

### Implemented (Priority 2)
- **Consent Management (GDPR Articles 6, 7)** - Grant, withdraw, and track user consents
  - Endpoints: `POST /api/v1/consent`, `POST /api/v1/consent/withdraw`, `GET /api/v1/consent/{user_id}`
  - Automatic consent checking in `log_action`
  - Consent history and audit trail
- **DPIA Tracking (GDPR Article 35)** - Data Protection Impact Assessment management
  - Endpoints: `POST /api/v1/dpia`, `PUT /api/v1/dpia/{dpia_id}`, `GET /api/v1/dpias`
  - Automatic consultation requirement detection for high-risk processing
  - DPIA history and approval workflow

### Implemented (Priority 2 - Continued)
- **Retention Period Automation (GDPR Article 5(1)(e))** - Automatic data deletion after retention periods
  - Endpoints: `POST /api/v1/retention/policy`, `POST /api/v1/retention/assign`, `GET /api/v1/retention/status/{record_type}/{record_id}`, `GET /api/v1/retention/policies`, `POST /api/v1/retention/execute_deletions`
  - Automatic deletion scheduling and execution
  - Crypto-shredding integration for compliance records
  - Retention exemptions support

### Implemented (Priority 2 - Completed)
- **Post-Market Monitoring (EU AI Act Article 72)** - Monitor AI systems after market release
  - Endpoints: `POST /api/v1/monitoring/event`, `PUT /api/v1/monitoring/event/{event_id}`, `GET /api/v1/monitoring/events`, `GET /api/v1/monitoring/health/{system_id}`
  - Automatic system health status tracking
  - Incident tracking and resolution management
  - Performance and compliance metrics monitoring

### Implemented (Priority 3 - Complete GDPR & EU AI Act Compliance)
- **Notification Service (GDPR Article 33, EU AI Act Article 13)** - Multi-channel notification system
  - SMTP email notifications (lettre crate)
  - Twilio SMS notifications
  - In-app notifications
  - Multi-language support (English, Slovak, extensible)
  - User notification preferences
  - Retry logic with exponential backoff
  - Endpoints: `POST /api/v1/data_subject/{user_id}/notification_preferences`, `GET /api/v1/data_subject/{user_id}/notification_preferences`

- **GDPR Article 18 - Right to Restriction of Processing**
  - Endpoints: `POST /api/v1/data_subject/{user_id}/restrict`, `POST /api/v1/data_subject/{user_id}/lift_restriction`, `GET /api/v1/data_subject/{user_id}/restrictions`
  - Automatic enforcement in `log_action` endpoint
  - Restriction history and audit trail

- **GDPR Article 19 - Notification of Rectification/Erasure**
  - Automatic notifications to data recipients
  - Data recipients tracking
  - Notification status management

- **GDPR Article 21 - Right to Object**
  - Endpoints: `POST /api/v1/data_subject/{user_id}/object`, `POST /api/v1/data_subject/{user_id}/withdraw_objection`, `GET /api/v1/data_subject/{user_id}/objections`
  - Automatic enforcement in `log_action` endpoint
  - Objection workflow and status tracking

- **GDPR Article 22 - Automated Decision-Making**
  - Endpoints: `POST /api/v1/automated_decision/{seal_id}/request_review`, `POST /api/v1/automated_decision/{seal_id}/appeal`, `GET /api/v1/automated_decisions`
  - Automatic detection of automated decisions
  - Human review workflow
  - Appeal process

- **GDPR Article 30 - Records of Processing Activities**
  - Endpoints: `GET /api/v1/processing_records`, `GET /api/v1/processing_records/export`
  - CSV export for DPO reporting
  - Automatic record generation from compliance logs

- **EU AI Act Article 8 - Conformity Assessment**
  - Endpoints: `POST /api/v1/conformity_assessments`, `GET /api/v1/conformity_assessments`
  - Assessment tracking and expiration management
  - Automated notifications (30 days before expiration)
  - Multiple assessment types (self-assessment, third-party, notified body)

- **EU AI Act Article 9 - Enhanced Risk Assessment**
  - Context-aware risk assessment with ML-based scoring
  - Historical data analysis
  - Dynamic risk factors weighting
  - User behavior risk analysis
  - Risk prediction and mitigation suggestions

- **EU AI Act Article 11 - Data Governance Extension**
  - Endpoints: `POST /api/v1/data_quality/metrics`, `POST /api/v1/data_quality/bias`, `POST /api/v1/data_quality/lineage`, `GET /api/v1/data_quality/report/{seal_id}`
  - Data quality metrics tracking (completeness, accuracy, consistency, validity, timeliness)
  - Data bias detection (demographic, geographic, temporal, representation)
  - Data lineage tracking (source tracking, transformation history)

- **Annex IV Extended Reports**
  - Multi-format export: PDF, JSON, XML
  - Extended fields: lifecycle stages, training data sources, performance metrics, post-market monitoring, human oversight procedures, risk management measures
  - Endpoint: `GET /api/v1/download_report?format=pdf|json|xml`

- **Performance Optimization**
  - API response compression (actix-web-compress)
  - Database query optimization (existing indexes, materialized views)
  - Background job processing (tokio::spawn)
  - Connection pooling tuning (sqlx)

### Implemented (Enterprise Features)
- **AI-BOM Export (CycloneDX v1.5)** - Standardized AI/ML Bill of Materials export
  - Endpoints: `GET /api/v1/ai_bom/{system_id}`, `POST /api/v1/ai_bom/inventory`
  - CycloneDX format for enterprise security tool integration
  - AI system inventory tracking with dependencies
  - DPIA and compliance metadata in BOM
- **Green AI Telemetry (EU AI Act Article 40)** - Energy efficiency and carbon footprint tracking
  - Automatic energy calculation from inference time and power ratings
  - Carbon footprint tracking (EU grid average: 475 g CO2/kWh)
  - Integrated into `POST /api/v1/log_action` endpoint
  - ESG reporting ready

- **Webhook Support** - Real-time compliance event notifications
  - HMAC-SHA256 signed webhook deliveries
  - Configurable retry logic with exponential backoff
  - Event filtering by type
  - Delivery history tracking
  - Endpoints: `POST /api/v1/webhooks`, `GET /api/v1/webhooks`, `PUT /api/v1/webhooks/{id}`, `DELETE /api/v1/webhooks/{id}`, `GET /api/v1/webhooks/{id}/deliveries`

- **Comprehensive Dashboard** - Modern Next.js web interface
  - Real-time updates (10-second refresh)
  - All compliance features accessible through intuitive UI
  - Interactive charts and visualizations
  - Responsive design for desktop and mobile
  - Dark theme interface
  - Available at `http://localhost:3000` (when running `npm run dev` in `dashboard/` directory)

- **Performance Optimization** - Production-ready performance enhancements
  - **Database Indexes**: 20+ indexes on frequently queried columns
  - **Materialized Views**: Pre-aggregated reporting data (daily compliance, system health)
  - **Connection Pooling**: Optimized pool (5-20 connections) with health checks
  - **Pagination**: All list endpoints support `page` and `limit` parameters
  - **Background Workers**: Async processing for webhooks, retention deletions, and view refreshes
  - **Query Optimization**: Automatic table analysis and view refresh functions
  - **Expected Performance**: 50-90% faster queries, reduced memory usage, improved concurrency

- **Security Hardening** - Enterprise-grade security features
  - **JWT Authentication**: Token-based authentication with configurable expiration
  - **Role-Based Access Control (RBAC)**: Fine-grained permissions (admin, compliance_officer, auditor, viewer)
  - **API Key Management**: Service-to-service authentication with SHA-256 hashing
  - **Security Audit Logging**: Comprehensive logging of all security events
  - **Rate Limiting**: IP-based throttling (configurable requests per minute)
  - **Security Headers**: CORS, X-Frame-Options, CSP, HSTS, X-XSS-Protection, Referrer-Policy
  - **Production CORS**: Environment-based origin whitelisting
  - **Dependency Vulnerability Scanning**: Automated checking with cargo-audit

- **AI Platform SDKs** - Compliance integration for major AI platforms
  - **Azure AI SDK**: Automatic compliance logging for Azure OpenAI services
  - **AWS Bedrock SDK**: Bedrock integration with EU region enforcement
  - **GCP Vertex AI SDK**: Vertex AI integration with EU region enforcement
  - **LangChain SDK**: Wrapper for any LangChain LLM with automatic compliance
  - **OpenAI MCP SDK**: OpenAI API integration with Model Context Protocol support
  - **HuggingFace SDK**: Transformers pipelines with energy and carbon tracking
  - All SDKs available in `sdks/` directory
  - See `sdks/README.md` for installation and usage instructions

## 🔌 AI Platform SDKs

Veridion Nexus provides SDKs for seamless integration with major AI platforms. All SDKs automatically log operations to Veridion Nexus for compliance.

### Supported Platforms

- ✅ **Azure AI** - Azure OpenAI and Azure AI services
- ✅ **AWS Bedrock** - Amazon Bedrock models (EU regions only)
- ✅ **GCP Vertex AI** - Google Cloud Vertex AI (EU regions only)
- ✅ **LangChain** - Any LangChain-compatible LLM
- ✅ **OpenAI MCP** - OpenAI API with Model Context Protocol
- ✅ **HuggingFace** - Transformers pipelines

### Quick Start

```bash
# Install all SDKs
pip install veridion-nexus-sdks[all]

# Or install specific platform SDKs
pip install veridion-nexus-sdks[azure]
pip install veridion-nexus-sdks[aws]
pip install veridion-nexus-sdks[gcp]
pip install veridion-nexus-sdks[langchain]
pip install veridion-nexus-sdks[openai]
pip install veridion-nexus-sdks[huggingface]
```

### Example: LangChain Integration

```python
from sdks.langchain import wrap_langchain_llm
from langchain.llms import OpenAI

# Create your LangChain LLM
llm = OpenAI(temperature=0.7)

# Wrap it with Veridion compliance
veridion_llm = wrap_langchain_llm(
    llm=llm,
    veridion_api_url="http://localhost:8080",
    veridion_api_key="your-veridion-key"
)

# Use it normally - compliance is automatic
response = veridion_llm("What is GDPR?")
```

### Features

- ✅ **Automatic Compliance Logging**: All AI operations logged automatically
- ✅ **Data Sovereignty Enforcement**: Non-EU regions blocked (AWS, GCP)
- ✅ **Energy Tracking**: GPU/CPU power consumption and carbon footprint
- ✅ **Error Handling**: Errors logged without breaking your application
- ✅ **Async Support**: Non-blocking compliance logging

See `sdks/README.md` for complete documentation and examples.

## 📄 License

**AGPL-3.0** - See [LICENSE](LICENSE)

Open-source with copyleft. If you modify and deploy as SaaS, you must open-source changes.

---

## 📋 Licensing & Commercial Pricing

### Open Source (Community Edition)

**Free to use** - You can run Veridion Nexus locally or in your own infrastructure at no cost under the AGPL-3.0 license.

**What's included:**
- All core compliance modules (Sovereign Lock, Crypto-Shredder, Privacy Bridge, Annex IV)
- Full source code access
- Docker deployment
- Community documentation
- Local development and testing

**AGPL-3.0 Requirements:**
- If you modify the code and deploy it as a service, you must open-source your changes
- Perfect for internal use, development, and evaluation

### Commercial License

For production deployments where you need:
- **Commercial licensing** (no AGPL copyleft requirements)
- **Enterprise support** (SLA guarantees, dedicated support)
- **Professional services** (implementation consulting, custom integrations)
- **Priority features** (early access to new modules)

### Pricing Tiers

#### Tier 1: Starter (€35,000/year)

**Deployment Mode**: Embedded (SDK-First)

**Target**: Series A fintech/insurtech, 1-10 employees

**Core Modules** (Always Included):
- Sovereign Lock, Crypto-Shredder, Privacy Bridge
- Audit Log Chain, Annex IV Compiler

**Operational Modules**: Choose 2 included
- Options: Data Subject Rights, Human Oversight, Risk Assessment, Breach Management

**Includes**:
- Up to 3 high-risk AI systems
- All 6 AI Platform SDKs (Azure, AWS, GCP, LangChain, OpenAI, HuggingFace)
- Email support (48h SLA)
- Standard Annex IV templates
- Community documentation

**Ideal for**: Fintech startups, small healthcare providers

#### Tier 2: Professional (€120,000/year) ⭐

**Deployment Mode**: Embedded OR Proxy (customer choice)

**Target**: Series B-D fintech/insurtech, 50-500 employees

**Core Modules** (Always Included):
- Sovereign Lock, Crypto-Shredder, Privacy Bridge
- Audit Log Chain, Annex IV Compiler

**Operational Modules**: All 10 modules included
- Data Subject Rights, Human Oversight, Risk Assessment
- Breach Management, Consent Management, DPIA Tracking
- Retention Policies, Post-Market Monitoring
- Green AI Telemetry, AI-BOM

**Includes**:
- Up to 15 high-risk AI systems
- All 6 AI Platform SDKs
- Slack channel support (12h SLA)
- Webhook integrations
- Monthly compliance reports
- Quarterly business reviews

**Ideal for**: Regional banks, mid-size insurance companies, growing enterprises

#### Tier 3: Enterprise (€350,000/year base)

**Deployment Mode**: Full Governance (complete platform)

**Target**: Banks, large insurers, public companies, 1000+ employees

**Core Modules** (Always Included):
- Sovereign Lock, Crypto-Shredder, Privacy Bridge
- Audit Log Chain, Annex IV Compiler

**Operational Modules**: All included + priority feature requests

**Includes**:
- Up to 50 high-risk AI systems (first 50 included)
- Deployment Options: SaaS, On-Premise, or Hybrid
- All 6 AI Platform SDKs + custom integrations
- Dedicated Customer Success Manager
- 24/7 phone support
- 99.9% SLA guarantee
- Custom integrations (40 hours/year included)
- Regulatory sandbox application support
- Audit defense package (expert testimony)
- Private Slack channel with engineering team

**Overage**: €12,000 per 10 additional systems (after first 50)

**Ideal for**: Tier 1 banks, large healthcare systems, systemically important institutions

### Early Adopter Program

**First 10 customers get 50% off** their first year subscription.

**Design Partners**: Looking for 2-3 beta customers for free pilot program.

### Add-Ons (All Tiers)

- **Module Add-Ons** (Starter tier only): Additional Operational Module: €10,000/year each
- **Deployment Upgrades**: Embedded → Proxy Mode: +€25,000/year | Embedded/Proxy → Full Governance: +€50,000/year
- **Transaction-Based**: eIDAS Seals: €0.10 per seal (volume discounts available) | High-Volume Package: €50,000/year (unlimited seals)
- **Professional Services**: Implementation Consulting: €2,500/day | Custom Integration: €5,000 per integration | Compliance Audit Support: €10,000 per audit

### Contact for Commercial Licensing

📧 **Sales Inquiry**: sales@veridion.nexus

💼 **Partnership**: sales@veridion.nexus

---

## 🚀 Get Started

### For Developers

- 📖 [Architecture Docs](ARCHITECTURE.md)

- 🐳 [Deployment Guide](DEPLOYMENT_GUIDE.md)  

- 💻 [Quick Start](#quick-start) (scroll up)

**Run it locally for free** - The Community Edition is fully functional for development and testing. See [Quick Start](#quick-start) above.

### For Companies

- 📧 **Contact**: sales@veridion.nexus

- 💼 **Early Adopter Program**: First 10 customers get 50% off

- 🤝 **Design Partners**: Looking for 2-3 beta customers (free pilot)

---

**Built for EU AI Act Compliance**  

*Preparing companies for December 2027 deadline*
