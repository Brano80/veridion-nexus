# VERIDION NEXUS
## Sovereign Trust Layer for High-Risk AI Agents

> **"Compliance as a Runtime Constraint."**

Veridion Nexus is a Rust-based middleware platform designed for High-Risk AI Agents in the EU. It enforces Data Sovereignty, GDPR Erasure, and eIDAS Trust at the network level, ensuring AI agents cannot physically violate the EU AI Act, GDPR, and eIDAS regulations.

## 🎯 Core Value Proposition

**Runtime Enforcement of Compliance** - Instead of relying on policies and audits, Veridion Nexus provides **technical guarantees** that make it physically impossible for AI agents to violate EU regulations.

## 🏗️ Architecture

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

- Data Subject Rights (GDPR Articles 15-22)
- Human Oversight (EU AI Act Article 14)
- Risk Assessment (EU AI Act Article 9)
- Breach Management (GDPR Articles 33-34)
- Consent Management (GDPR Articles 6-7)
- DPIA Tracking (GDPR Article 35)
- Retention Policies (GDPR Article 5(1)(e))
- Post-Market Monitoring (EU AI Act Article 72)
- Green AI Telemetry (EU AI Act Article 40)
- AI-BOM (CycloneDX Standard)

### 3. Integration Layer (Always Available)

**SDKs and connectors** for seamless integration:

- **AI Platform SDKs**: Azure AI, AWS Bedrock, GCP Vertex, LangChain, OpenAI MCP, HuggingFace
- **Webhooks**: Real-time event notifications with HMAC-SHA256 signing
- **Proxy Mode**: Reverse proxy middleware for existing AI infrastructure
- **REST API**: Complete API for all features

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed architecture documentation.

## 🚀 Quick Start

### Prerequisites

- **Rust 1.70+** ([Install Rust](https://rustup.rs/))
- **Docker** (Optional, for containerized deployment)
- **Signicat API Credentials** (Optional - system works in mock mode by default)

### Local Development

1. **Clone and build:**
   ```bash
   git clone <repository-url>
   cd veridion-nexus
   cargo build
   ```

2. **Set environment variables (optional):**
   ```bash
   export VERIDION_MASTER_KEY=your_master_key_here
   export USE_REAL_API=false  # Set to true for live eIDAS sealing
   export SIGNICAT_CLIENT_ID=your_client_id
   export SIGNICAT_CLIENT_SECRET=your_client_secret
   ```

3. **Run the server:**
   ```bash
   cargo run
   ```

4. **Access the API:**
   - **Health Check:** http://localhost:8080/health
   - **Swagger UI:** http://localhost:8080/swagger-ui/
   - **API Base:** http://localhost:8080/api/v1

### Docker Deployment

```bash
docker-compose up --build
```

The API will be available at `http://localhost:8080`

## 📚 API Documentation

### Core Endpoints

#### Compliance Logging
- `POST /api/v1/log_action` - Log a high-risk AI action
- `GET /api/v1/logs` - Retrieve compliance log history
- `GET /api/v1/download_report` - Download Annex IV PDF report

#### GDPR Compliance (Priority 1)
- `GET /api/v1/data_subject/{user_id}/access` - Right to access (GDPR Article 15)
- `GET /api/v1/data_subject/{user_id}/export` - Data portability (GDPR Article 20)
- `PUT /api/v1/data_subject/{user_id}/rectify` - Right to rectification (GDPR Article 16)
- `POST /api/v1/shred_data` - Right to be forgotten (GDPR Article 17)

#### EU AI Act Compliance (Priority 1)
- `POST /api/v1/action/{seal_id}/require_approval` - Require human oversight
- `POST /api/v1/action/{seal_id}/approve` - Approve action (Human Oversight)
- `POST /api/v1/action/{seal_id}/reject` - Reject action (Human Oversight)
- `GET /api/v1/risk_assessment/{seal_id}` - Get risk assessment (EU AI Act Article 9)
- `GET /api/v1/risks` - Get all risk assessments

#### Data Breach Management (Priority 1)
- `POST /api/v1/breach_report` - Report data breach (GDPR Articles 33-34)
- `GET /api/v1/breaches` - List all breaches

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

## 🔒 Compliance Features

### GDPR Compliance
- ✅ **Article 15** - Right of access
- ✅ **Article 16** - Right to rectification
- ✅ **Article 17** - Right to erasure ("Right to be Forgotten")
- ✅ **Article 20** - Right to data portability
- ✅ **Article 25** - Data protection by design
- ✅ **Article 32** - Security of processing
- ✅ **Article 33-34** - Data breach notification

### EU AI Act Compliance
- ✅ **Article 9** - Risk management system
- ✅ **Article 10** - Data governance (Sovereign Lock)
- ✅ **Article 13** - Transparency requirements
- ✅ **Article 14** - Human oversight
- ✅ **Article 40** - Energy efficiency reporting (Green AI Telemetry)
- ✅ **Article 72** - Post-market monitoring
- ✅ **Annex IV** - Technical documentation (Automated PDF generation)

### eIDAS Compliance
- ✅ **Article 36** - Qualified Electronic Seals
- ✅ **Article 37** - Requirements for Qualified Electronic Seals

## 🧪 Testing

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

## 📖 Configuration

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

# Logging
RUST_LOG=info                         # Log level: trace, debug, info, warn, error
```

## 🔧 Project Structure

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

## 🚦 Status

### ✅ Implemented (MVP Ready)
- Core compliance modules (Sovereign Lock, Crypto-Shredder, Privacy Bridge, Annex IV)
- Priority 1: Data Subject Rights (GDPR Articles 15, 16, 17, 20)
- Priority 1: Human Oversight (EU AI Act Article 14)
- Priority 1: Risk Assessment (EU AI Act Article 9)
- Priority 1: Data Breach Reporting (GDPR Articles 33-34)
- **PostgreSQL persistence** - All data stored in database
- Swagger UI documentation
- Docker deployment with PostgreSQL
- Integration tests with database support

### ✅ Implemented (Priority 2)
- **Consent Management (GDPR Articles 6, 7)** - Grant, withdraw, and track user consents
  - Endpoints: `POST /api/v1/consent`, `POST /api/v1/consent/withdraw`, `GET /api/v1/consent/{user_id}`
  - Automatic consent checking in `log_action`
  - Consent history and audit trail
- **DPIA Tracking (GDPR Article 35)** - Data Protection Impact Assessment management
  - Endpoints: `POST /api/v1/dpia`, `PUT /api/v1/dpia/{dpia_id}`, `GET /api/v1/dpias`
  - Automatic consultation requirement detection for high-risk processing
  - DPIA history and approval workflow

### ✅ Implemented (Priority 2 - Continued)
- **Retention Period Automation (GDPR Article 5(1)(e))** - Automatic data deletion after retention periods
  - Endpoints: `POST /api/v1/retention/policy`, `POST /api/v1/retention/assign`, `GET /api/v1/retention/status/{record_type}/{record_id}`, `GET /api/v1/retention/policies`, `POST /api/v1/retention/execute_deletions`
  - Automatic deletion scheduling and execution
  - Crypto-shredding integration for compliance records
  - Retention exemptions support

### ✅ Implemented (Priority 2 - Completed)
- **Post-Market Monitoring (EU AI Act Article 72)** - Monitor AI systems after market release
  - Endpoints: `POST /api/v1/monitoring/event`, `PUT /api/v1/monitoring/event/{event_id}`, `GET /api/v1/monitoring/events`, `GET /api/v1/monitoring/health/{system_id}`
  - Automatic system health status tracking
  - Incident tracking and resolution management
  - Performance and compliance metrics monitoring

### ✅ Implemented (Enterprise Features)
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

[Add your license here]

## 🤝 Contributing

[Add contribution guidelines]

## 📞 Support

[Add support contact information]

---

**Built with ❤️ for EU Compliance**
