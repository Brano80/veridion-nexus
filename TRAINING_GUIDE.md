# Veridion Nexus - Complete Training Program
## Master the Platform for Customer Presentations

**Version 1.1 | January 2025**  
**Updated:** Complete GDPR & EU AI Act Compliance Features

---

## 📋 Table of Contents

1. [Module 1: Platform Overview & Architecture](#module-1-platform-overview--architecture)
2. [Module 2: Setup & First Steps](#module-2-setup--first-steps)
3. [Module 3: Core Features - Hands-On](#module-3-core-features---hands-on)
4. [Module 4: Operational Modules](#module-4-operational-modules)
5. [Module 5: Integration & SDKs](#module-5-integration--sdks)
6. [Module 6: Dashboard Operations](#module-6-dashboard-operations)
7. [Module 7: Customer Presentation Scenarios](#module-7-customer-presentation-scenarios)
8. [Module 8: Troubleshooting & Support](#module-8-troubleshooting--support)

---

# Module 1: Platform Overview & Architecture

## 🎯 Learning Objectives

By the end of this module, you will:
- Understand what Veridion Nexus is and why it exists
- Know the 3-layer architecture
- Understand deployment modes
- Be able to explain the value proposition

## 1.1 What is Veridion Nexus?

**Veridion Nexus** is a **runtime compliance enforcement platform** for High-Risk AI systems in the EU. Unlike traditional compliance tools that rely on policies and audits, Veridion Nexus provides **technical guarantees** that make it physically impossible for AI agents to violate EU regulations.

### Key Differentiators

1. **Runtime Enforcement** (not just monitoring)
   - Blocks violations at the network level
   - Prevents data from leaving EU/EEA jurisdictions
   - Enforces compliance before actions occur

2. **EU-First Architecture**
   - Built specifically for EU AI Act, GDPR, and eIDAS
   - Not a generic compliance tool adapted for EU

3. **Technical Guarantees**
   - Cryptographic proofs of compliance
   - Immutable audit trails
   - Automated documentation generation

## 1.2 The Problem We Solve

### The Compliance Crisis

**EU AI Act** (effective 2026) requires:
- Data sovereignty (Article 10): Data must stay in EU/EEA
- Technical documentation (Annex IV): Every AI decision documented
- Human oversight (Article 14): High-risk actions need approval
- Risk management (Article 9): Continuous risk assessment

**GDPR** requires:
- Right to be Forgotten (Article 17): Data deletion on request
- But audit logs must be immutable (security requirement)
- **Paradox**: Can't delete from immutable logs

**eIDAS** requires:
- Qualified Electronic Seals for legal proof
- Cryptographic signatures for compliance evidence

### Current Solutions Fail Because:

- ❌ **Process-based**: Rely on policies, not technical enforcement
- ❌ **Reactive**: Detect violations after they happen
- ❌ **Generic**: Not built for EU AI Act specifics
- ❌ **Expensive**: Custom solutions cost €500K-€2M

### Veridion Nexus Solution:

- ✅ **Technical enforcement**: Blocks violations at network level
- ✅ **Proactive**: Prevents violations before they occur
- ✅ **EU-specific**: Built for EU AI Act, GDPR, eIDAS
- ✅ **Cost-effective**: 70% cheaper than custom solutions

## 1.3 Three-Layer Architecture

### Layer 1: Core Runtime Compliance Engine (Mandatory)

**Always enabled** - These provide the core compliance guarantees:

1. **Sovereign Lock** (`src/core/sovereign_lock.rs`)
   - Runtime geofencing for data sovereignty
   - Blocks non-EU/EEA jurisdictions
   - EU AI Act Article 10 compliance

2. **Crypto-Shredder** (`src/core/crypto_shredder.rs`)
   - GDPR envelope encryption
   - Right to be Forgotten (Article 17)
   - AES-256-GCM encryption

3. **Privacy Bridge** (`src/core/privacy_bridge.rs`)
   - eIDAS Qualified Electronic Seals
   - Hash-based sealing
   - Signicat integration

4. **Audit Log Chain** (integrated in routes)
   - Immutable audit trail
   - Compliance record storage
   - Real-time logging

5. **Annex IV Compiler** (`src/core/annex_iv.rs`)
   - Automated technical documentation
   - PDF report generation
   - EU AI Act Annex IV compliance

### Layer 2: Operational Modules (Optional)

**Can be enabled/disabled** via Module Configuration API:

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

### Layer 3: Integration Layer (Always Available)

**SDKs and connectors** for seamless integration:

- **AI Platform SDKs**: Azure AI, AWS Bedrock, GCP Vertex, LangChain, OpenAI MCP, HuggingFace
- **Webhooks**: Real-time event notifications with HMAC-SHA256 signing
- **Proxy Mode**: Reverse proxy middleware for existing AI infrastructure
- **REST API**: Complete API for all features

## 1.4 Deployment Modes

### Mode 1: Embedded (SDK-First)
- **Best for**: Startups, mid-market companies
- **How it works**: SDKs integrated directly in application code
- **Pricing**: €35K-€120K/year
- **Example**: Python app uses `VeridionAzureAI` wrapper

### Mode 2: Proxy (Reverse Proxy)
- **Best for**: Enterprise with existing AI infrastructure
- **How it works**: Nexus runs as middleware, intercepts AI API calls
- **Pricing**: €120K-€350K/year
- **Example**: All AI calls go through Nexus proxy automatically

### Mode 3: Full Governance (Complete Platform)
- **Best for**: Enterprise requiring complete control
- **How it works**: Complete platform deployment with all modules
- **Pricing**: €350K+/year
- **Example**: Full dashboard, all modules, on-premise option

## 1.5 Value Proposition Summary

**For Customers:**
- ✅ **Compliance Guarantee**: Technical enforcement, not just policies
- ✅ **Cost Savings**: 70% cheaper than custom solutions
- ✅ **Time Savings**: 90% reduction in compliance documentation time
- ✅ **Risk Reduction**: Prevents violations before they occur
- ✅ **EU-Specific**: Built for EU AI Act, not adapted

**For You (Sales):**
- ✅ **Clear Differentiation**: Only runtime enforcement platform
- ✅ **Modular Pricing**: Customers pay only for what they need
- ✅ **Multiple Entry Points**: SDK, Proxy, or Full deployment
- ✅ **High Switching Costs**: Deep integration creates lock-in

---

# Module 2: Setup & First Steps

## 🎯 Learning Objectives

By the end of this module, you will:
- Be able to set up Veridion Nexus locally
- Understand environment configuration
- Know how to verify the installation
- Access Swagger UI and Dashboard

## 2.1 Prerequisites

### Required:
- **Docker** and **Docker Compose** (recommended)
- OR **Rust 1.70+** and **PostgreSQL 14+** (manual setup)
- **Git** (to clone repository)

### Optional:
- **Signicat API Credentials** (for real eIDAS sealing)
- **Python 3.8+** (for SDK testing)

## 2.2 Quick Start with Docker (Recommended)

### Step 1: Clone Repository
```bash
git clone https://github.com/Brano80/veridion-nexus.git
cd veridion-nexus
```

### Step 2: Start Services
```bash
docker-compose up --build
```

This will:
- Build the Rust API server
- Start PostgreSQL database
- Run database migrations
- Start the API on port 8080

### Step 3: Verify Installation

**Health Check:**
```bash
curl http://localhost:8080/health
```

Expected response:
```json
{
  "status": "healthy",
  "service": "veridion-nexus",
  "version": "1.0.0"
}
```

**Swagger UI:**
Open in browser: `http://localhost:8080/swagger-ui/`

You should see the interactive API documentation.

## 2.3 Environment Configuration

### Create `.env` File

Create a `.env` file in the project root:

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

### Key Environment Variables Explained

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection string |
| `JWT_SECRET` | Yes | Secret for JWT token signing (min 32 chars) |
| `ALLOWED_ORIGINS` | No | CORS allowed origins (comma-separated) |
| `PORT` | No | API server port (default: 8080) |
| `RUST_LOG` | No | Log level: trace, debug, info, warn, error |

## 2.4 Manual Setup (Without Docker)

### Step 1: Install PostgreSQL
```bash
# Ubuntu/Debian
sudo apt-get install postgresql-14

# macOS
brew install postgresql@14

# Windows
# Download from https://www.postgresql.org/download/windows/
```

### Step 2: Create Database
```bash
# Connect to PostgreSQL
psql -U postgres

# Create database and user
CREATE DATABASE veridion_nexus;
CREATE USER veridion WITH PASSWORD 'veridion_password';
GRANT ALL PRIVILEGES ON DATABASE veridion_nexus TO veridion;
\q
```

### Step 3: Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Step 4: Build and Run
```bash
# Set environment variables
export DATABASE_URL=postgresql://veridion:veridion_password@localhost:5432/veridion_nexus
export JWT_SECRET=your-secret-key-minimum-32-characters-long

# Run migrations
sqlx migrate run

# Build and run
cargo build --release
cargo run
```

## 2.5 Verify Installation

### Test 1: Health Check
```bash
curl http://localhost:8080/health
```

### Test 2: Swagger UI
Open: `http://localhost:8080/swagger-ui/`

You should see all API endpoints documented.

### Test 3: Database Connection
```bash
# Check if migrations ran
docker-compose exec db psql -U veridion -d veridion_nexus -c "\dt"
```

You should see tables like:
- `compliance_records`
- `users`
- `roles`
- `api_keys`
- etc.

## 2.6 Dashboard Setup (Optional)

### Start Dashboard
```bash
cd dashboard
npm install --legacy-peer-deps
npm run dev
```

Dashboard will be available at: `http://localhost:3000`

### Dashboard Features:
- Compliance Overview
- Runtime Logs Explorer
- Human Oversight Queue
- Module Management
- Settings

## 2.7 Common Setup Issues

### Issue: Database Connection Failed
**Solution:**
```bash
# Check if PostgreSQL is running
docker-compose ps

# Check database logs
docker-compose logs db

# Restart services
docker-compose restart
```

### Issue: Port 8080 Already in Use
**Solution:**
```bash
# Change port in docker-compose.yml or .env
PORT=8081

# Or stop the conflicting service
# Find process using port 8080
lsof -i :8080  # macOS/Linux
netstat -ano | findstr :8080  # Windows
```

### Issue: Migrations Failed
**Solution:**
```bash
# Run migrations manually
docker-compose exec api sqlx migrate run

# Or reset database (WARNING: deletes all data)
docker-compose down -v
docker-compose up --build
```

---

# Module 3: Core Features - Hands-On

## 🎯 Learning Objectives

By the end of this module, you will:
- Understand and demonstrate Sovereign Lock
- Know how Crypto-Shredder works
- Be able to use Privacy Bridge for eIDAS sealing
- Generate Annex IV PDF reports
- Use the API for core compliance operations

## 3.1 Sovereign Lock - Data Sovereignty Enforcement

### What It Does

**Sovereign Lock** enforces EU AI Act Article 10 (Data Governance) by blocking any AI action that would send data to non-EU/EEA jurisdictions.

### How It Works

1. Every AI action includes a `target_region` parameter
2. Sovereign Lock checks if `target_region` is in EU/EEA
3. If not, the action is **blocked** with `SOVEREIGN_LOCK_VIOLATION` error
4. If yes, the action proceeds

### Practical Exercise: Test Sovereign Lock

**Step 1: Try to log action with non-EU region**

```bash
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "agent_id": "test-agent",
    "action_type": "credit_scoring",
    "payload": {"user_id": "123", "score": 750},
    "target_region": "us-east-1"
  }'
```

**Expected Response (Error):**
```json
{
  "error": "SOVEREIGN_LOCK_VIOLATION",
  "message": "Data sovereignty violation: target_region 'us-east-1' is not in EU/EEA"
}
```

**Step 2: Try with EU region**

```bash
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "agent_id": "test-agent",
    "action_type": "credit_scoring",
    "payload": {"user_id": "123", "score": 750},
    "target_region": "eu-west-1"
  }'
```

**Expected Response (Success):**
```json
{
  "seal_id": "seal_abc123",
  "tx_id": "tx_xyz789",
  "status": "sealed",
  "message": "Action logged and sealed successfully"
}
```

### Key Points for Customer Presentation

- ✅ **Technical Guarantee**: Physically impossible to send data outside EU
- ✅ **Real-time Enforcement**: Blocks at network level, not after the fact
- ✅ **EU AI Act Compliant**: Meets Article 10 requirements automatically

## 3.2 Crypto-Shredder - GDPR Right to be Forgotten

### What It Does

**Crypto-Shredder** solves the GDPR paradox:
- Audit logs must be **immutable** (security requirement)
- GDPR requires **Right to be Forgotten** (Article 17)
- **Solution**: Envelope encryption - encrypt data, delete the key

### How It Works

1. When logging action, data is encrypted with a unique key
2. Key is stored separately from encrypted data
3. When "Right to be Forgotten" is requested:
   - Key is deleted (cryptographic erasure)
   - Data remains in log but is unreadable
   - Compliance record shows "ERASED (Art. 17)"

### Practical Exercise: Test Crypto-Shredder

**Step 1: Log an action (data gets encrypted)**

```bash
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "agent_id": "test-agent",
    "action_type": "user_profiling",
    "payload": {"user_id": "user_123", "name": "John Doe", "email": "john@example.com"},
    "target_region": "eu-west-1"
  }'
```

Save the `seal_id` from the response.

**Step 2: Shred the data (Right to be Forgotten)**

```bash
curl -X POST http://localhost:8080/api/v1/shred_data \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "seal_id": "seal_abc123"
  }'
```

**Expected Response:**
```json
{
  "status": "erased",
  "seal_id": "seal_abc123",
  "message": "Data cryptographically erased per GDPR Article 17"
}
```

**Step 3: Verify data is unreadable**

```bash
curl http://localhost:8080/api/v1/logs?seal_id=seal_abc123 \
  -H "X-API-Key: your-api-key"
```

The record will show:
- `status: "ERASED (Art. 17)"`
- `encrypted_payload: null` (key deleted, data unreadable)
- Original data is gone forever

### Key Points for Customer Presentation

- ✅ **Solves GDPR Paradox**: Immutable logs + Right to be Forgotten
- ✅ **Cryptographic Proof**: Key deletion is provable
- ✅ **Compliance**: Meets GDPR Article 17 requirements

## 3.3 Privacy Bridge - eIDAS Qualified Electronic Seals

### What It Does

**Privacy Bridge** provides eIDAS Qualified Electronic Seals for legal proof of compliance. It:
- Creates cryptographic hash of the action
- Gets eIDAS seal from Signicat (or mock in development)
- Provides `seal_id` as legal proof

### How It Works

1. Action payload is hashed (SHA-256)
2. Hash is sent to Signicat API for eIDAS seal
3. Seal is stored with compliance record
4. `seal_id` is returned as proof

### Practical Exercise: Test Privacy Bridge

**Step 1: Log action (gets sealed automatically)**

```bash
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "agent_id": "test-agent",
    "action_type": "fraud_detection",
    "payload": {"transaction_id": "tx_456", "amount": 1000},
    "target_region": "eu-west-1"
  }'
```

**Response includes:**
```json
{
  "seal_id": "seal_abc123",
  "tx_id": "tx_xyz789",
  "status": "sealed",
  "eidas_seal_url": "https://api.signicat.com/seals/seal_abc123"
}
```

**Step 2: Verify seal (in production with real Signicat)**

The `seal_id` can be verified with Signicat API to prove:
- Action occurred at specific time
- Data integrity (hash matches)
- Legal compliance proof

### Key Points for Customer Presentation

- ✅ **Legal Proof**: eIDAS seals are legally binding in EU
- ✅ **Audit Defense**: Cryptographic proof of compliance
- ✅ **Regulatory Ready**: Meets eIDAS Regulation requirements

## 3.4 Annex IV Compiler - Automated Documentation

### What It Does

**Annex IV Compiler** automatically generates EU AI Act Annex IV technical documentation as PDF reports. This saves 90% of manual documentation time.

### How It Works

1. Compliance records are automatically tracked
2. PDF report is generated with all required fields:
   - System specifications
   - Input/output descriptions
   - Training methodologies
   - Risk assessments
   - Compliance verification
3. Report is downloadable via API

### Practical Exercise: Generate Annex IV Report

**Step 1: Log several actions (to build compliance history)**

```bash
# Action 1
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "agent_id": "credit-scoring-v1",
    "action_type": "credit_scoring",
    "payload": {"user_id": "123", "score": 750},
    "target_region": "eu-west-1"
  }'

# Action 2
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "agent_id": "credit-scoring-v1",
    "action_type": "credit_scoring",
    "payload": {"user_id": "456", "score": 680},
    "target_region": "eu-west-1"
  }'
```

**Step 2: Download Annex IV PDF Report**

```bash
curl http://localhost:8080/api/v1/download_report?agent_id=credit-scoring-v1 \
  -H "X-API-Key: your-api-key" \
  --output annex_iv_report.pdf
```

**Step 3: Open the PDF**

The report will contain:
- System identification
- Technical specifications
- All compliance records
- Risk assessments
- Compliance verification

### Key Points for Customer Presentation

- ✅ **90% Time Savings**: Automated vs. manual documentation
- ✅ **Always Up-to-Date**: Real-time compliance tracking
- ✅ **Regulatory Ready**: Meets EU AI Act Annex IV requirements

## 3.5 Complete Workflow Example

### Scenario: Credit Scoring AI System

**Step 1: Log credit scoring action**

```bash
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "agent_id": "credit-scoring-v1",
    "action_type": "credit_scoring",
    "payload": {
      "user_id": "user_789",
      "credit_score": 720,
      "decision": "approved",
      "loan_amount": 50000
    },
    "target_region": "eu-west-1",
    "inference_time_ms": 150,
    "gpu_power_rating_watts": 0,
    "cpu_power_rating_watts": 50
  }'
```

**Response:**
```json
{
  "seal_id": "seal_credit_123",
  "tx_id": "tx_credit_456",
  "status": "sealed",
  "energy_estimate_kwh": 0.00000208,
  "carbon_grams": 0.00099
}
```

**Step 2: View compliance logs**

```bash
curl http://localhost:8080/api/v1/logs?agent_id=credit-scoring-v1 \
  -H "X-API-Key: your-api-key"
```

**Step 3: Generate Annex IV report**

```bash
curl http://localhost:8080/api/v1/download_report?agent_id=credit-scoring-v1 \
  -H "X-API-Key: your-api-key" \
  --output credit_scoring_report.pdf
```

**Step 4: If user requests Right to be Forgotten**

```bash
curl -X POST http://localhost:8080/api/v1/shred_data \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "seal_id": "seal_credit_123"
  }'
```

---

# Module 4: Operational Modules

## 🎯 Learning Objectives

By the end of this module, you will:
- Understand all operational modules
- Know how to enable/disable modules
- Be able to demonstrate each module's functionality
- Understand when to use each module

## 4.1 Module Management

### List All Modules

```bash
curl http://localhost:8080/api/v1/modules \
  -H "X-API-Key: your-api-key"
```

**Response:**
```json
{
  "modules": [
    {
      "name": "data_subject_rights",
      "display_name": "Data Subject Rights",
      "description": "GDPR Articles 15-22 compliance",
      "enabled": true,
      "category": "operational"
    },
    {
      "name": "human_oversight",
      "display_name": "Human Oversight",
      "description": "EU AI Act Article 14 compliance",
      "enabled": false,
      "category": "operational"
    }
    // ... more modules
  ]
}
```

### Enable a Module

```bash
curl -X POST http://localhost:8080/api/v1/modules/human_oversight/enable \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{}'
```

### Disable a Module

```bash
curl -X POST http://localhost:8080/api/v1/modules/human_oversight/disable \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{}'
```

## 4.2 Data Subject Rights (GDPR Articles 15-22)

### Features:
- **Right to Access** (Article 15)
- **Right to Rectification** (Article 16)
- **Right to Erasure** (Article 17) - via Crypto-Shredder
- **Right to Data Portability** (Article 20)

### Practical Exercise: Data Subject Access Request

**Step 1: User requests access to their data**

```bash
curl http://localhost:8080/api/v1/data_subject/user_123/access \
  -H "X-API-Key: your-api-key"
```

**Response:**
```json
{
  "user_id": "user_123",
  "records": [
    {
      "seal_id": "seal_abc123",
      "action_type": "credit_scoring",
      "timestamp": "2025-01-15T10:30:00Z",
      "status": "active"
    }
  ]
}
```

### Practical Exercise: Data Portability

**Step 2: User requests data export (portable format)**

```bash
curl http://localhost:8080/api/v1/data_subject/user_123/export \
  -H "X-API-Key: your-api-key"
```

**Response:** JSON file with all user's data in portable format

### Practical Exercise: Data Rectification

**Step 3: User requests correction of data**

```bash
curl -X PUT http://localhost:8080/api/v1/data_subject/user_123/rectify \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "seal_id": "seal_abc123",
    "corrections": {
      "email": "newemail@example.com"
    }
  }'
```

## 4.3 Human Oversight (EU AI Act Article 14)

### Features:
- Review queue for high-risk actions
- Approve/reject actions before execution
- Audit trail of all oversight decisions

### Practical Exercise: Human Oversight Workflow

**Step 1: Require approval for high-risk action**

```bash
curl -X POST http://localhost:8080/api/v1/action/seal_abc123/require_approval \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "reason": "High-risk credit decision",
    "reviewer_role": "compliance_officer"
  }'
```

**Step 2: View oversight queue**

```bash
curl http://localhost:8080/api/v1/oversight/queue \
  -H "X-API-Key: your-api-key"
```

**Step 3: Approve action**

```bash
curl -X POST http://localhost:8080/api/v1/action/seal_abc123/approve \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "reviewer_id": "reviewer_001",
    "notes": "Approved after risk assessment"
  }'
```

**Step 4: Or reject action**

```bash
curl -X POST http://localhost:8080/api/v1/action/seal_abc123/reject \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "reviewer_id": "reviewer_001",
    "reason": "Risk score too low"
  }'
```

## 4.4 Risk Assessment (EU AI Act Article 9)

### Features:
- Automatic risk scoring
- Risk mitigation tracking
- Risk assessment history

### Practical Exercise: Risk Assessment

**Step 1: Get risk assessment for action**

```bash
curl http://localhost:8080/api/v1/risk_assessment/seal_abc123 \
  -H "X-API-Key: your-api-key"
```

**Response:**
```json
{
  "seal_id": "seal_abc123",
  "risk_level": "high",
  "risk_score": 8.5,
  "risk_factors": [
    "High financial impact",
    "Automated decision-making",
    "Personal data processing"
  ],
  "mitigation_measures": [
    "Human oversight required",
    "Additional verification needed"
  ]
}
```

**Step 2: Get all risk assessments**

```bash
curl http://localhost:8080/api/v1/risks?risk_level=high \
  -H "X-API-Key: your-api-key"
```

## 4.5 Breach Management (GDPR Articles 33-34)

### Features:
- Breach reporting
- Notification tracking
- Breach history

### Practical Exercise: Report Data Breach

**Step 1: Report a breach**

```bash
curl -X POST http://localhost:8080/api/v1/breach_report \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "breach_type": "unauthorized_access",
    "description": "Unauthorized access to user database",
    "affected_users": 150,
    "discovery_time": "2025-01-15T14:30:00Z",
    "severity": "high"
  }'
```

**Response:**
```json
{
  "breach_id": "breach_001",
  "status": "reported",
  "notification_deadline": "2025-01-17T14:30:00Z",
  "message": "Breach reported. Notification to authorities required within 72 hours."
}
```

**Step 2: List all breaches**

```bash
curl http://localhost:8080/api/v1/breaches \
  -H "X-API-Key: your-api-key"
```

## 4.6 Consent Management (GDPR Articles 6-7)

### Features:
- Consent tracking
- Consent withdrawal
- Consent history

### Practical Exercise: Consent Management

**Step 1: Grant consent**

```bash
curl -X POST http://localhost:8080/api/v1/consent \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "user_id": "user_123",
    "consent_type": "data_processing",
    "purpose": "credit_scoring",
    "granted": true
  }'
```

**Step 2: Withdraw consent**

```bash
curl -X POST http://localhost:8080/api/v1/consent/withdraw \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "user_id": "user_123",
    "consent_type": "data_processing"
  }'
```

**Step 3: Get user consents**

```bash
curl http://localhost:8080/api/v1/consent/user_123 \
  -H "X-API-Key: your-api-key"
```

## 4.7 Other Operational Modules

### DPIA Tracking (GDPR Article 35)
- Create DPIA: `POST /api/v1/dpia`
- Update DPIA: `PUT /api/v1/dpia/{dpia_id}`
- List DPIAs: `GET /api/v1/dpias`

### Retention Policies (GDPR Article 5(1)(e))
- Create policy: `POST /api/v1/retention/policy`
- Assign policy: `POST /api/v1/retention/assign`
- Check status: `GET /api/v1/retention/status/{record_type}/{record_id}`

### Post-Market Monitoring (EU AI Act Article 72)
- Create event: `POST /api/v1/monitoring/event`
- Update resolution: `PUT /api/v1/monitoring/event/{event_id}`
- List events: `GET /api/v1/monitoring/events`

### Green AI Telemetry (EU AI Act Article 40)
- Automatic energy/carbon tracking in `log_action`
- Fields: `inference_time_ms`, `gpu_power_rating_watts`, `cpu_power_rating_watts`
- Calculates: `energy_estimate_kwh`, `carbon_grams`

### AI-BOM (CycloneDX Standard)
- Register system: `POST /api/v1/ai_bom/inventory`
- Export BOM: `GET /api/v1/ai_bom/{system_id}`

---

# Module 5: Integration & SDKs

## 🎯 Learning Objectives

By the end of this module, you will:
- Understand how to use Python SDKs
- Know how to configure webhooks
- Understand Proxy Mode
- Be able to authenticate with API

## 5.1 Python SDKs Overview

### Supported Platforms:
- Azure AI
- AWS Bedrock
- GCP Vertex AI
- LangChain
- OpenAI MCP
- HuggingFace

### Installation

```bash
# Install all SDKs
pip install veridion-nexus-sdks[all]

# Or install specific platform
pip install veridion-nexus-sdks[azure]
pip install veridion-nexus-sdks[aws]
pip install veridion-nexus-sdks[gcp]
pip install veridion-nexus-sdks[langchain]
pip install veridion-nexus-sdks[openai]
pip install veridion-nexus-sdks[huggingface]
```

## 5.2 Azure AI SDK Example

### Setup

```python
from sdks.azure_ai import VeridionAzureAI
from azure.core.credentials import AzureKeyCredential
import asyncio

async def main():
    # Create Veridion-wrapped Azure AI client
    client = VeridionAzureAI(
        endpoint="https://your-endpoint.openai.azure.com/",
        credential=AzureKeyCredential("your-azure-key"),
        veridion_api_url="http://localhost:8080",
        veridion_api_key="your-veridion-key",
        agent_id="my-azure-agent"
    )
    
    # Use normally - compliance is automatic
    response = await client.complete(
        messages=[{"role": "user", "content": "Hello!"}],
        model="gpt-4"
    )
    
    print(response)
    await client.close()

asyncio.run(main())
```

### What Happens Automatically:
1. AI call is made to Azure
2. Call is logged to Veridion Nexus
3. Sovereign Lock checks region
4. Privacy Bridge creates eIDAS seal
5. Crypto-Shredder encrypts data
6. Energy/carbon is tracked

## 5.3 LangChain SDK Example

```python
from sdks.langchain import wrap_langchain_llm
from langchain.llms import OpenAI

# Create your LangChain LLM
llm = OpenAI(temperature=0.7)

# Wrap it with Veridion compliance
veridion_llm = wrap_langchain_llm(
    llm=llm,
    veridion_api_url="http://localhost:8080",
    veridion_api_key="your-veridion-key",
    agent_id="my-langchain-agent"
)

# Use it normally - compliance is automatic
response = veridion_llm("What is GDPR?")
print(response)
```

## 5.4 Webhook Configuration

### Register Webhook

```bash
curl -X POST http://localhost:8080/api/v1/webhooks \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "url": "https://your-app.com/webhooks/veridion",
    "events": ["action_logged", "breach_reported", "oversight_required"],
    "secret": "your-webhook-secret"
  }'
```

**Response:**
```json
{
  "id": "webhook_001",
  "url": "https://your-app.com/webhooks/veridion",
  "events": ["action_logged", "breach_reported", "oversight_required"],
  "status": "active",
  "secret": "your-webhook-secret"
}
```

### Webhook Events

Available events:
- `action_logged` - New compliance record created
- `breach_reported` - Data breach reported
- `oversight_required` - Human oversight needed
- `data_shredded` - Right to be Forgotten executed
- `risk_assessed` - Risk assessment completed

### Webhook Payload Example

```json
{
  "event_type": "action_logged",
  "timestamp": "2025-01-15T10:30:00Z",
  "data": {
    "seal_id": "seal_abc123",
    "agent_id": "credit-scoring-v1",
    "action_type": "credit_scoring"
  },
  "signature": "hmac-sha256-signature"
}
```

### Verify Webhook Signature

```python
import hmac
import hashlib

def verify_webhook_signature(payload, signature, secret):
    expected_signature = hmac.new(
        secret.encode(),
        payload.encode(),
        hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(expected_signature, signature)
```

## 5.5 API Authentication

### Method 1: API Keys (Service-to-Service)

**Create API Key:**
```bash
curl -X POST http://localhost:8080/api/v1/api_keys \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-jwt-token" \
  -d '{
    "name": "production-api-key",
    "expires_at": "2026-01-15T00:00:00Z"
  }'
```

**Response:**
```json
{
  "api_key": "vn_live_abc123xyz789...",
  "api_key_id": "key_001",
  "name": "production-api-key",
  "created_at": "2025-01-15T10:00:00Z",
  "expires_at": "2026-01-15T00:00:00Z"
}
```

**⚠️ IMPORTANT: Save the API key immediately - it's only shown once!**

**Use API Key:**
```bash
curl http://localhost:8080/api/v1/logs \
  -H "X-API-Key: vn_live_abc123xyz789..."
```

### Method 2: JWT Tokens (User Authentication)

**Login:**
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "your-password"
  }'
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "user_001",
    "username": "admin",
    "roles": ["admin"]
  }
}
```

**Use JWT Token:**
```bash
curl http://localhost:8080/api/v1/logs \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

## 5.6 Proxy Mode Setup

### What is Proxy Mode?

Proxy Mode runs Veridion Nexus as a reverse proxy middleware. All AI API calls go through Nexus, which:
- Automatically logs actions
- Enforces data sovereignty
- Adds compliance layer
- No code changes needed

### Configuration Example

```yaml
# docker-compose.yml
services:
  veridion-proxy:
    image: veridion-nexus:latest
    environment:
      - DEPLOYMENT_MODE=proxy
      - PROXY_TARGET=https://api.openai.com
      - PROXY_PORT=8080
    ports:
      - "8080:8080"
```

### Usage

Instead of calling OpenAI directly:
```python
# OLD: Direct call
response = openai.ChatCompletion.create(...)
```

Call through Veridion proxy:
```python
# NEW: Through proxy (automatic compliance)
response = openai.ChatCompletion.create(
    api_base="http://localhost:8080/proxy/openai",
    ...
)
```

All compliance is handled automatically!

---

# Module 6: Dashboard Operations

## 🎯 Learning Objectives

By the end of this module, you will:
- Navigate the Compliance Hub dashboard
- Use Runtime Logs Explorer
- Manage modules via UI
- Configure settings

## 6.1 Dashboard Overview

### Access Dashboard

1. Start dashboard:
```bash
cd dashboard
npm run dev
```

2. Open browser: `http://localhost:3000`

### Dashboard Structure

**Core Pages (Always Available):**
- **Compliance Overview** - Main dashboard with metrics
- **Runtime Logs** - All compliance records
- **Audit Reports** - Annex IV PDF generation
- **Data Shredding Actions** - Right to be Forgotten queue
- **Settings** - API keys, webhooks, configuration

**Plugin Pages (Based on Enabled Modules):**
- Human Oversight Queue (if `human_oversight` enabled)
- Risk Assessment (if `risk_assessment` enabled)
- Data Breaches (if `breach_management` enabled)
- Consent Management (if `consent` enabled)
- DPIA Tracking (if `dpia` enabled)
- Retention Policies (if `retention` enabled)
- Post-Market Monitoring (if `monitoring` enabled)
- Green AI Telemetry (if `green_ai` enabled)
- AI-BOM Viewer (if `ai_bom` enabled)

## 6.2 Compliance Overview Page

### What You See:
- **Total Compliance Records** - Count of all logged actions
- **Active Systems** - Number of AI systems being monitored
- **Risk Level Distribution** - Chart of risk levels
- **Recent Activity** - Latest compliance records
- **System Health** - Status of all monitored systems

### Practical Exercise: View Overview

1. Navigate to: `http://localhost:3000`
2. You'll see the main dashboard
3. Metrics update every 10 seconds automatically

## 6.3 Runtime Logs Explorer

### Features:
- Filter by agent_id, action_type, date range
- Search by seal_id, tx_id
- Export to CSV
- View detailed record information
- Shred data (Right to be Forgotten)

### Practical Exercise: Explore Logs

1. Navigate to: `http://localhost:3000/runtime-logs`
2. Use filters:
   - Agent ID: `credit-scoring-v1`
   - Date Range: Last 7 days
   - Action Type: `credit_scoring`
3. Click on a record to see details
4. Click "Shred Data" to execute Right to be Forgotten

## 6.4 Module Management

### Enable/Disable Modules

1. Navigate to: `http://localhost:3000/settings`
2. Scroll to "Module Configuration"
3. Toggle modules on/off
4. Changes take effect immediately

### View Module Status

Modules show:
- **Enabled/Disabled** status
- **Description** of what the module does
- **Category** (core/operational)

## 6.5 Settings Page

### API Keys Management

1. Navigate to: `http://localhost:3000/settings`
2. Section: "API Keys"
3. Click "Create New API Key"
4. Copy the key (shown only once!)
5. View all keys and revoke if needed

### Webhook Configuration

1. Navigate to: `http://localhost:3000/settings`
2. Section: "Webhooks"
3. Click "Add Webhook"
4. Enter URL, select events, set secret
5. View delivery history

### System Configuration

- API URL: `http://localhost:8080`
- Database status
- System health
- Version information

---

# Module 7: Customer Presentation Scenarios

## 🎯 Learning Objectives

By the end of this module, you will:
- Know how to present to different customer types
- Have ready demo scenarios
- Understand common objections
- Be able to explain pricing

## 7.1 Presentation to Fintech Startup (Starter Tier)

### Customer Profile:
- **Size**: 1-10 employees
- **Stage**: Series A
- **Budget**: €35K-€75K/year
- **Pain Points**: Need compliance but limited budget

### Demo Flow (15 minutes):

**1. Problem Statement (2 min)**
- "EU AI Act requires compliance by 2026"
- "Manual compliance is expensive and time-consuming"
- "You need technical enforcement, not just policies"

**2. Solution Overview (3 min)**
- Show architecture diagram
- Explain Core vs. Modules
- Emphasize: "Start with Core, add modules as you grow"

**3. Live Demo (8 min)**

**Demo 1: Sovereign Lock**
```bash
# Show blocking non-EU region
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: demo-key" \
  -d '{
    "agent_id": "demo-agent",
    "action_type": "credit_scoring",
    "payload": {"user_id": "123"},
    "target_region": "us-east-1"
  }'
# Show error: SOVEREIGN_LOCK_VIOLATION

# Show allowing EU region
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: demo-key" \
  -d '{
    "agent_id": "demo-agent",
    "action_type": "credit_scoring",
    "payload": {"user_id": "123"},
    "target_region": "eu-west-1"
  }'
# Show success with seal_id
```

**Demo 2: Dashboard**
- Open `http://localhost:3000`
- Show compliance overview
- Show runtime logs
- Show Annex IV report generation

**Demo 3: SDK Integration**
```python
# Show Python SDK example
from sdks.langchain import wrap_langchain_llm
# Show how easy integration is
```

**4. Pricing Discussion (2 min)**
- Starter: €35K/year
- Includes: Core modules + 2 operational modules
- Embedded mode (SDK-first)
- All 6 SDKs included
- Email support

### Key Talking Points:
- ✅ "Start small, scale as you grow"
- ✅ "70% cheaper than custom solutions"
- ✅ "Technical guarantee, not just policies"
- ✅ "90% time savings on compliance documentation"

### Common Objections & Responses:

**Objection**: "€35K is too expensive for a startup"
**Response**: "Compare to €500K-€2M for custom solution. Plus, you get compliance guarantee and 90% time savings on documentation."

**Objection**: "We can build this ourselves"
**Response**: "That's €500K-€2M and 6-12 months. We're production-ready now, and you get ongoing updates for regulatory changes."

**Objection**: "We're not ready for compliance yet"
**Response**: "EU AI Act enforcement starts 2026. Better to be ready now than face €35M fines later."

## 7.2 Presentation to Enterprise Bank (Enterprise Tier)

### Customer Profile:
- **Size**: 1000+ employees
- **Stage**: Established bank
- **Budget**: €350K+/year
- **Pain Points**: Regulatory audits, multiple AI systems, need for control

### Demo Flow (30 minutes):

**1. Problem Statement (5 min)**
- Regulatory audit preparation
- Multiple AI systems to manage
- Need for complete audit trail
- Data sovereignty requirements

**2. Solution Overview (5 min)**
- Full Governance Mode
- All modules included
- On-premise option
- Dedicated support

**3. Live Demo (18 min)**

**Demo 1: Complete Compliance Workflow**
- Log action
- Show human oversight queue
- Show risk assessment
- Show Annex IV report
- Show audit trail

**Demo 2: Multi-System Management**
- Show multiple agent_ids
- Show system health dashboard
- Show centralized compliance view

**Demo 3: Advanced Features**
- Webhook integration
- API key management
- RBAC demonstration
- Audit logging

**4. Pricing Discussion (2 min)**
- Enterprise: €350K/year base
- Includes: All modules, 50 systems, on-premise option
- Overage: €12K per 10 additional systems
- Dedicated Customer Success Manager
- 24/7 support, 99.9% SLA

### Key Talking Points:
- ✅ "Complete audit defense package"
- ✅ "Regulatory sandbox support included"
- ✅ "Expert testimony available"
- ✅ "On-premise deployment option"

### Common Objections & Responses:

**Objection**: "We have existing compliance tools"
**Response**: "Veridion Nexus is runtime enforcement, not just monitoring. It prevents violations, not just detects them. Works alongside your existing tools."

**Objection**: "We need on-premise"
**Response**: "Enterprise tier includes on-premise option. We provide Docker containers for air-gapped environments."

**Objection**: "Integration is complex"
**Response**: "We provide SDKs for all major platforms, plus Proxy Mode for zero-code integration. Dedicated support included."

## 7.3 Presentation to Healthcare Provider (Professional Tier)

### Customer Profile:
- **Size**: 50-500 employees
- **Stage**: Growing healthcare system
- **Budget**: €100K-€300K/year
- **Pain Points**: Patient data sovereignty, medical device compliance

### Demo Flow (20 minutes):

**1. Problem Statement (3 min)**
- Patient data sovereignty (GDPR)
- Medical device regulation
- Audit trail requirements
- HIPAA considerations (if applicable)

**2. Solution Overview (4 min)**
- Professional tier features
- All 10 modules included
- Embedded or Proxy mode
- Healthcare-specific compliance

**3. Live Demo (11 min)**

**Demo 1: Patient Data Protection**
- Show Sovereign Lock blocking non-EU
- Show Crypto-Shredder for Right to be Forgotten
- Show data subject rights

**Demo 2: Medical Device Compliance**
- Show Annex IV documentation
- Show risk assessment
- Show human oversight

**Demo 3: Audit Trail**
- Show complete audit log
- Show eIDAS seals
- Show PDF reports

**4. Pricing Discussion (2 min)**
- Professional: €120K/year
- Includes: All modules, 15 systems
- Slack support, monthly reports
- Quarterly business reviews

### Key Talking Points:
- ✅ "Patient data stays in EU"
- ✅ "Complete audit trail for medical devices"
- ✅ "Automated compliance documentation"
- ✅ "GDPR Right to be Forgotten support"

## 7.4 Pricing Presentation

### Starter Tier (€35,000/year)
**Best for**: Startups, Series A fintech/insurtech

**Includes:**
- Core modules (always)
- 2 operational modules (your choice)
- Up to 3 high-risk AI systems
- All 6 SDKs
- Email support (48h SLA)

**Add-ons:**
- Additional module: €10K/year each
- Deployment upgrade: €25K-€50K/year

### Professional Tier (€120,000/year) ⭐
**Best for**: Series B-D, 50-500 employees

**Includes:**
- Core modules (always)
- All 10 operational modules
- Up to 15 high-risk AI systems
- All 6 SDKs
- Slack support (12h SLA)
- Webhook integrations
- Monthly compliance reports
- Quarterly business reviews

### Enterprise Tier (€350,000/year base)
**Best for**: Banks, large insurers, 1000+ employees

**Includes:**
- Core modules (always)
- All operational modules + priority features
- Up to 50 systems (first 50 included)
- Deployment: SaaS, On-Premise, or Hybrid
- All 6 SDKs + custom integrations
- Dedicated Customer Success Manager
- 24/7 phone support
- 99.9% SLA guarantee
- Custom integrations (40 hours/year)
- Regulatory sandbox support
- Audit defense package

**Overage**: €12,000 per 10 additional systems

### Add-Ons (All Tiers)
- Module add-ons: €10K/year (Starter only)
- Deployment upgrades: €25K-€50K/year
- eIDAS Seals: €0.10 per seal (volume discounts)
- High-Volume Package: €50K/year (unlimited seals)
- Professional Services: €2,500/day
- Regulatory Sandbox Fast-Track: €25K one-time
- Audit Defense Package: €50K/year

---

# Module 8: Troubleshooting & Support

## 🎯 Learning Objectives

By the end of this module, you will:
- Know how to diagnose common issues
- Understand log analysis
- Be able to troubleshoot performance
- Know security best practices

## 8.1 Common Issues & Solutions

### Issue: API Returns 500 Error

**Symptoms:**
- API calls return HTTP 500
- No specific error message

**Diagnosis:**
```bash
# Check API logs
docker-compose logs api

# Check database connection
docker-compose exec db psql -U veridion -d veridion_nexus -c "SELECT 1"
```

**Solutions:**
1. **Database connection issue**: Check `DATABASE_URL` in `.env`
2. **Migration not run**: Run `sqlx migrate run`
3. **Out of memory**: Increase Docker memory limit
4. **Port conflict**: Change `PORT` in `.env`

### Issue: Sovereign Lock Blocks Valid EU Region

**Symptoms:**
- `SOVEREIGN_LOCK_VIOLATION` for EU regions
- Action should be allowed

**Diagnosis:**
```bash
# Check target_region format
# Valid formats: "eu-west-1", "eu-central-1", "eu-north-1", etc.
```

**Solutions:**
1. **Region format**: Use AWS/GCP region codes (e.g., "eu-west-1")
2. **Case sensitivity**: Use lowercase
3. **Check allowed regions**: Verify region is in EU/EEA list

### Issue: Webhook Not Delivering

**Symptoms:**
- Webhook registered but no deliveries
- Delivery status shows "failed"

**Diagnosis:**
```bash
# Check webhook deliveries
curl http://localhost:8080/api/v1/webhooks/{id}/deliveries \
  -H "X-API-Key: your-api-key"
```

**Solutions:**
1. **URL unreachable**: Verify webhook URL is accessible
2. **SSL certificate**: Check if HTTPS certificate is valid
3. **Secret mismatch**: Verify webhook secret matches
4. **Rate limiting**: Check if webhook endpoint has rate limits

### Issue: Dashboard Not Loading

**Symptoms:**
- Dashboard shows blank page
- API errors in browser console

**Diagnosis:**
```bash
# Check dashboard logs
cd dashboard
npm run dev
# Check browser console for errors
```

**Solutions:**
1. **API URL**: Verify `NEXT_PUBLIC_API_URL` in dashboard `.env`
2. **CORS**: Check `ALLOWED_ORIGINS` includes dashboard URL
3. **API running**: Verify API is running on correct port
4. **Network**: Check firewall/network settings

## 8.2 Log Analysis

### API Logs

**View logs:**
```bash
# Docker
docker-compose logs -f api

# Manual
# Logs go to stdout/stderr
# Set RUST_LOG=debug for detailed logs
```

**Log Levels:**
- `trace` - Very detailed
- `debug` - Debug information
- `info` - General information (default)
- `warn` - Warnings
- `error` - Errors only

### Database Logs

**View PostgreSQL logs:**
```bash
docker-compose logs db
```

**Check slow queries:**
```sql
-- Enable slow query log in postgresql.conf
log_min_duration_statement = 1000  -- Log queries > 1 second
```

### Application Logs

**Compliance records:**
```bash
curl http://localhost:8080/api/v1/logs \
  -H "X-API-Key: your-api-key"
```

**Audit logs:**
```sql
SELECT * FROM security_audit_logs 
ORDER BY created_at DESC 
LIMIT 100;
```

## 8.3 Performance Tuning

### Database Optimization

**Check indexes:**
```sql
-- List all indexes
SELECT tablename, indexname 
FROM pg_indexes 
WHERE schemaname = 'public';
```

**Analyze tables:**
```sql
ANALYZE compliance_records;
ANALYZE users;
-- etc.
```

**Refresh materialized views:**
```sql
REFRESH MATERIALIZED VIEW daily_compliance_summary;
REFRESH MATERIALIZED VIEW system_health_summary;
```

### API Performance

**Connection pooling:**
- Default: 5-20 connections
- Adjust in `src/database.rs`:
```rust
.max_connections(20)
.min_connections(5)
```

**Rate limiting:**
- Default: 100 requests/minute
- Adjust in `.env`:
```bash
RATE_LIMIT_REQUESTS_PER_MINUTE=200
```

### Caching (Future Enhancement)

Currently no caching, but can be added:
- Redis for session storage
- In-memory cache for frequently accessed data

## 8.4 Security Best Practices

### API Keys

**Do:**
- ✅ Rotate API keys regularly
- ✅ Use different keys for different environments
- ✅ Revoke unused keys immediately
- ✅ Store keys securely (environment variables, secrets manager)

**Don't:**
- ❌ Commit API keys to git
- ❌ Share keys between team members
- ❌ Use production keys in development

### JWT Tokens

**Do:**
- ✅ Use strong `JWT_SECRET` (min 32 characters)
- ✅ Set appropriate expiration times
- ✅ Rotate secrets periodically

**Don't:**
- ❌ Use weak secrets
- ❌ Expose tokens in URLs
- ❌ Store tokens in localStorage (use httpOnly cookies)

### Database Security

**Do:**
- ✅ Use strong database passwords
- ✅ Restrict database access (firewall rules)
- ✅ Enable SSL for database connections
- ✅ Regular backups

**Don't:**
- ❌ Expose database to public internet
- ❌ Use default passwords
- ❌ Store credentials in code

### CORS Configuration

**Production:**
```bash
ALLOWED_ORIGINS=https://yourdomain.com,https://api.yourdomain.com
```

**Development:**
```bash
ALLOWED_ORIGINS=http://localhost:3000,http://localhost:8080
```

**Never:**
```bash
ALLOWED_ORIGINS=*  # Only in development!
```

## 8.5 Support Resources

### Documentation
- **README.md** - Quick start guide
- **ARCHITECTURE.md** - Architecture details
- **DEPLOYMENT_GUIDE.md** - Production deployment
- **ENV_VARIABLES.md** - Environment configuration
- **Swagger UI** - Interactive API docs

### Getting Help

**For Customers:**
- **Starter**: Email support (48h SLA)
- **Professional**: Slack channel (12h SLA)
- **Enterprise**: Dedicated Customer Success Manager (24/7)

**For Internal:**
- Check logs first
- Review documentation
- Test in development environment
- Escalate to engineering if needed

---

# Appendix: Quick Reference

## API Endpoints Quick Reference

### Core Compliance
- `POST /api/v1/log_action` - Log AI action
- `GET /api/v1/logs` - Get compliance logs
- `POST /api/v1/shred_data` - Right to be Forgotten
- `GET /api/v1/download_report` - Annex IV PDF

### Data Subject Rights
- `GET /api/v1/data_subject/{user_id}/access` - Right to access
- `GET /api/v1/data_subject/{user_id}/export` - Data portability
- `PUT /api/v1/data_subject/{user_id}/rectify` - Rectification

### Human Oversight
- `POST /api/v1/action/{seal_id}/require_approval` - Require approval
- `POST /api/v1/action/{seal_id}/approve` - Approve action
- `POST /api/v1/action/{seal_id}/reject` - Reject action

### Risk Assessment
- `GET /api/v1/risk_assessment/{seal_id}` - Get risk assessment
- `GET /api/v1/risks` - List all risks

### Breach Management
- `POST /api/v1/breach_report` - Report breach
- `GET /api/v1/breaches` - List breaches

### Module Management
- `GET /api/v1/modules` - List modules
- `POST /api/v1/modules/{name}/enable` - Enable module
- `POST /api/v1/modules/{name}/disable` - Disable module

### Authentication
- `POST /api/v1/auth/login` - Login (get JWT)
- `POST /api/v1/auth/register` - Register user
- `GET /api/v1/auth/me` - Get current user

### API Keys
- `POST /api/v1/api_keys` - Create API key
- `GET /api/v1/api_keys` - List API keys
- `GET /api/v1/api_keys/{id}` - Get API key details
- `POST /api/v1/api_keys/{id}/revoke` - Revoke API key

### Webhooks
- `POST /api/v1/webhooks` - Register webhook
- `GET /api/v1/webhooks` - List webhooks
- `PUT /api/v1/webhooks/{id}` - Update webhook
- `DELETE /api/v1/webhooks/{id}` - Delete webhook
- `GET /api/v1/webhooks/{id}/deliveries` - Get delivery history

## Environment Variables Quick Reference

```bash
# Required
DATABASE_URL=postgresql://user:password@host:5432/database
JWT_SECRET=your-secret-key-minimum-32-characters

# Optional
PORT=8080
RUST_LOG=info
ALLOWED_ORIGINS=http://localhost:3000
RATE_LIMIT_REQUESTS_PER_MINUTE=100
```

## Common curl Commands

### Health Check
```bash
curl http://localhost:8080/health
```

### Log Action
```bash
curl -X POST http://localhost:8080/api/v1/log_action \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-key" \
  -d '{"agent_id":"test","action_type":"test","payload":{},"target_region":"eu-west-1"}'
```

### Get Logs
```bash
curl http://localhost:8080/api/v1/logs \
  -H "X-API-Key: your-key"
```

### Shred Data
```bash
curl -X POST http://localhost:8080/api/v1/shred_data \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-key" \
  -d '{"seal_id":"seal_abc123"}'
```

---

**End of Training Guide**

**Next Steps:**
1. Complete all practical exercises
2. Practice customer presentations
3. Set up your own demo environment
4. Review troubleshooting scenarios
5. Master the dashboard operations

**Good luck with your customer presentations! 🚀**

