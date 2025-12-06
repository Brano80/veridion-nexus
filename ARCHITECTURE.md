# Veridion Nexus - Modular Architecture

## Overview

Veridion Nexus is organized into three distinct layers:

1. **Core Runtime Compliance Engine** (Mandatory)
2. **Operational Modules** (Optional)
3. **Integration Layer** (Always Available)

## Architecture Layers

### 1. Core Runtime Compliance Engine (`src/core/`)

**Status**: Always enabled, cannot be disabled

These are the mandatory components that provide the core compliance guarantees:

- **Sovereign Lock** (`sovereign_lock.rs`)
  - Runtime geofencing for data sovereignty
  - Blocks non-EU/EEA jurisdictions
  - EU AI Act Article 10 compliance

- **Crypto-Shredder** (`crypto_shredder.rs`)
  - GDPR envelope encryption
  - Right to be Forgotten (Article 17)
  - AES-256-GCM encryption

- **Privacy Bridge** (`privacy_bridge.rs`)
  - eIDAS Qualified Electronic Seals
  - Hash-based sealing
  - Signicat integration

- **Audit Log Chain** (integrated in routes)
  - Immutable audit trail
  - Compliance record storage
  - Real-time logging

- **Annex IV Compiler** (`annex_iv.rs`)
  - Automated technical documentation
  - PDF report generation
  - EU AI Act Annex IV compliance

### 2. Operational Modules (`src/modules/`)

**Status**: Optional, can be enabled/disabled via Module Configuration API

These modules provide operational compliance features:

- **Data Subject Rights** (`data_subject_rights.rs`)
  - GDPR Articles 15-22
  - Access, export, rectification, erasure

- **Human Oversight** (`human_oversight.rs`)
  - EU AI Act Article 14
  - Review queue for high-risk actions

- **Risk Assessment** (`risk_assessment.rs`)
  - EU AI Act Article 9
  - Risk analysis and mitigation

- **Breach Management** (`breach_management.rs`)
  - GDPR Articles 33-34
  - Breach reporting and notification

- **Consent Management** (`consent.rs`)
  - GDPR Articles 6-7
  - Consent tracking and withdrawal

- **DPIA Tracking** (`dpia.rs`)
  - GDPR Article 35
  - Data Protection Impact Assessments

- **Retention Policies** (`retention.rs`)
  - GDPR Article 5(1)(e)
  - Automated data retention

- **Post-Market Monitoring** (`monitoring.rs`)
  - EU AI Act Article 72
  - System monitoring

- **Green AI Telemetry** (`green_ai.rs`)
  - EU AI Act Article 40
  - Energy and carbon tracking

- **AI-BOM** (`ai_bom.rs`)
  - CycloneDX standard
  - AI Bill of Materials

### 3. Integration Layer (`src/integration/`)

**Status**: Always available

- **Webhooks** (`webhooks.rs`)
  - Real-time event notifications
  - HMAC-SHA256 signing
  - Retry logic

- **Proxy Mode** (`proxy.rs`)
  - Reverse proxy middleware
  - Intercepts AI API calls
  - Adds compliance layer

- **SDKs** (`sdks/` directory)
  - Azure AI SDK
  - AWS Bedrock SDK
  - GCP Vertex AI SDK
  - LangChain SDK
  - OpenAI MCP SDK
  - HuggingFace SDK

## Module Configuration

Modules can be enabled/disabled via the Module Configuration API:

- `GET /api/v1/modules` - List all modules
- `POST /api/v1/modules/{name}/enable` - Enable a module
- `POST /api/v1/modules/{name}/disable` - Disable a module
- `GET /api/v1/modules/{name}/status` - Check module status

## Deployment Modes

### 1. Embedded Mode (SDK-First)
- Uses SDKs directly in application code
- Lightweight client library
- Best for: Startups, mid-market

### 2. Proxy Mode (Reverse Proxy)
- Nexus runs as middleware layer
- Intercepts AI API calls
- Best for: Enterprise with existing infrastructure

### 3. Full Governance Mode
- Complete platform deployment
- All modules available
- Best for: Enterprise requiring full control

## File Structure

```
src/
├── core/                    # Core Runtime Compliance Engine
│   ├── mod.rs
│   ├── sovereign_lock.rs
│   ├── crypto_shredder.rs
│   ├── privacy_bridge.rs
│   └── annex_iv.rs
├── modules/                 # Operational Modules
│   ├── mod.rs
│   ├── data_subject_rights.rs
│   ├── human_oversight.rs
│   ├── risk_assessment.rs
│   ├── breach_management.rs
│   ├── consent.rs
│   ├── dpia.rs
│   ├── retention.rs
│   ├── monitoring.rs
│   ├── green_ai.rs
│   └── ai_bom.rs
├── integration/            # Integration Layer
│   ├── mod.rs
│   ├── webhooks.rs
│   └── proxy.rs
├── routes/                 # API Routes
│   ├── auth.rs
│   ├── api_keys.rs
│   └── modules.rs
├── security/               # Security Features
│   ├── auth.rs
│   ├── rbac.rs
│   ├── api_keys.rs
│   ├── audit.rs
│   ├── headers.rs
│   └── rate_limit.rs
└── ...
```

## Benefits of Modular Architecture

1. **Reduced Complexity**: Customers only see features they need
2. **Modular Pricing**: Pay only for enabled modules
3. **Easier Integration**: Start with Core, add modules as needed
4. **Clear Value Proposition**: Core = mandatory, Modules = optional
5. **Simplified Onboarding**: Fewer features to learn initially

