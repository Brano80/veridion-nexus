# VERIDION: The Sovereign Trust Layer

> **"Compliance as a Runtime Constraint."**

Veridion is a Rust-based middleware protocol designed for High-Risk AI Agents in the EU. It enforces Data Sovereignty, GDPR Erasure, and eIDAS Trust at the network level, ensuring AI agents cannot physically violate the EU AI Act.

## Core Modules

### 1. The Sovereign Lock (Geofencing)

* **Function:** Network middleware that inspects outgoing IP addresses.

* **Compliance:** Blocks data leaks to non-EU jurisdictions (US/China).

* **Status:** Active (Panic on Violation).

### 2. The Crypto-Shredder (GDPR Engine)

* **Function:** Solves the "Immutable vs. Erasable" paradox using Envelope Encryption.

* **Compliance:** Allows "Right to be Forgotten" in immutable audit logs by destroying specific encryption keys.

* **Status:** Active (AES-256-GCM).

### 3. The Privacy Bridge (eIDAS Integration)

* **Function:** Hashes sensitive data locally and sends only the hash to a Qualified Trust Service Provider (QTSP) like Signicat.

* **Compliance:** Generates Qualified Electronic Seals (QES) without exposing user data to the cloud.

* **Status:** Hybrid (Mock/Live OAuth2 Ready).

### 4. Annex IV Compiler

* **Function:** Automates the creation of "Technical Documentation" required by the EU AI Act.

* **Output:** Generates a legally binding PDF report of every agent decision.

## Quick Start

### Prerequisites

* Rust 1.70+

* Signicat API Credentials (Optional - Defaults to Mock Mode)

### Usage

1. Run the Compliance Engine (Demo Mode)

```bash
cargo run
```

View the Generated Legal Report Open ./Veridion_Annex_IV_Report.pdf

## Configuration (.env)

```ini
# Toggle between Simulation and Live eIDAS sealing
USE_REAL_API=false

# Master Key for Crypto-Shredding
VERIDION_MASTER_KEY=placeholder_key
```
