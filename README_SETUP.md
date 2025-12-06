# Veridion Nexus - Setup Instructions

Complete setup guide for Veridion Nexus compliance middleware platform.

## Project Overview

Veridion Nexus is a Rust-based middleware that enforces GDPR, EU AI Act, and eIDAS compliance at the network level for High-Risk AI systems.

## Prerequisites

- **Rust 1.70+** - [Install from rustup.rs](https://rustup.rs/)
- **PostgreSQL 16+** - Required for data persistence
- **Docker & Docker Compose** (Recommended, for containerized deployment)
- **Signicat API Credentials** (Optional - system works in mock mode by default)

## Quick Start

### Option 1: Local Development (Recommended for Development)

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd veridion-nexus
   ```

2. **Set up PostgreSQL database:**
   ```bash
   # Create database (if not using Docker)
   createdb veridion_nexus
   
   # Or use existing PostgreSQL instance
   ```

3. **Set environment variables:**
   ```bash
   # Required
   export VERIDION_MASTER_KEY=your_secure_master_key_here
   export DATABASE_URL=postgresql://user:password@localhost:5432/veridion_nexus
   
   # Optional (for live eIDAS sealing)
   export USE_REAL_API=false
   export SIGNICAT_CLIENT_ID=your_client_id
   export SIGNICAT_CLIENT_SECRET=your_secret
   ```

4. **Build and run:**
   ```bash
   cargo build
   cargo run
   ```

5. **Access the API:**
   - Health: http://localhost:8080/health
   - Swagger UI: http://localhost:8080/swagger-ui/
   - API: http://localhost:8080/api/v1

### Option 2: Docker Deployment (Recommended for Production)

1. **Set environment variable (optional, for master key):**
   ```bash
   export VERIDION_MASTER_KEY=your_secure_master_key_here
   ```

2. **Build and start (includes PostgreSQL):**
   ```bash
   docker-compose up --build
   ```
   
   This will automatically:
   - Start PostgreSQL database
   - Run database migrations
   - Start the API server

3. **Access the API:**
   - Health: http://localhost:8080/health
   - Swagger UI: http://localhost:8080/swagger-ui/
   - PostgreSQL: localhost:5432 (user: veridion, password: veridion_password)

4. **Stop services:**
   ```bash
   docker-compose down
   ```
   
   **Note:** To remove database data, use:
   ```bash
   docker-compose down -v
   ```

## Project Structure

```
veridion-nexus/
├── src/
│   ├── main.rs                    # Application entry point
│   ├── lib.rs                     # Library exports
│   ├── api_state.rs               # Application state
│   ├── routes.rs                  # API endpoints
│   ├── compliance_models.rs      # Data models
│   ├── crypto_shredder.rs         # GDPR Article 17
│   ├── privacy_bridge.rs          # eIDAS integration
│   └── annex_iv_compiler.rs       # PDF generation
├── tests/
│   └── integration_test.rs       # Integration tests
├── Cargo.toml                     # Rust dependencies
├── Dockerfile                     # Docker configuration
└── docker-compose.yml             # Docker Compose setup
```

## API Endpoints

### Core Endpoints
- `GET /health` - Health check
- `POST /api/v1/log_action` - Log AI action
- `GET /api/v1/logs` - Get compliance logs
- `GET /api/v1/download_report` - Download Annex IV PDF

### GDPR Endpoints (Priority 1)
- `GET /api/v1/data_subject/{user_id}/access` - Right to access
- `GET /api/v1/data_subject/{user_id}/export` - Data portability
- `PUT /api/v1/data_subject/{user_id}/rectify` - Right to rectification
- `POST /api/v1/shred_data` - Right to be forgotten

### EU AI Act Endpoints (Priority 1)
- `POST /api/v1/action/{seal_id}/require_approval` - Require human oversight
- `POST /api/v1/action/{seal_id}/approve` - Approve action
- `POST /api/v1/action/{seal_id}/reject` - Reject action
- `GET /api/v1/risk_assessment/{seal_id}` - Get risk assessment
- `GET /api/v1/risks` - Get all risks

### Data Breach Endpoints (Priority 1)
- `POST /api/v1/breach_report` - Report breach
- `GET /api/v1/breaches` - List breaches

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `VERIDION_MASTER_KEY` | Yes | - | Master key for crypto-shredding (32 bytes recommended) |
| `DATABASE_URL` | Yes* | - | PostgreSQL connection string (*auto-set in Docker) |
| `USE_REAL_API` | No | `false` | Enable live Signicat API |
| `SIGNICAT_CLIENT_ID` | No | `placeholder_id` | Signicat OAuth2 Client ID |
| `SIGNICAT_CLIENT_SECRET` | No | `placeholder_secret` | Signicat OAuth2 Client Secret |
| `RUST_LOG` | No | `info` | Log level (trace, debug, info, warn, error) |
| `TEST_DATABASE_URL` | No | Same as `DATABASE_URL` | Database URL for integration tests |

## Database Schema

The application uses PostgreSQL with the following main tables:
- `compliance_records` - All AI actions and compliance logs
- `risk_assessments` - Risk assessments for each action
- `human_oversight` - Human oversight requests and decisions
- `data_breaches` - Data breach reports
- `user_data_index` - User ID to seal ID mapping (for GDPR)
- `encrypted_log_keys` - Encrypted keys for crypto-shredding
- `system_config` - System configuration (lockdown status, etc.)

Migrations are automatically run on application startup.

## Testing

### Run Unit Tests
```bash
cargo test
```

### Run Integration Tests
```bash
# Make sure PostgreSQL is running
# Set TEST_DATABASE_URL if using different test database
export TEST_DATABASE_URL=postgresql://veridion:veridion_password@localhost:5432/veridion_nexus_test

cargo test --test integration_test
```

### Run Integration Tests
```bash
cargo test --test integration_test
```

### Test with Python Agent
```bash
python test_agent.py
```

## Troubleshooting

### Port Already in Use
Change the port in `docker-compose.yml` or use a different port:
```bash
cargo run -- --port 8081
```

### Build Errors
- Ensure Rust 1.70+ is installed: `rustc --version`
- Clear build cache: `cargo clean && cargo build`

### API Connection Issues
- Check if server is running: `curl http://localhost:8080/health`
- Check logs: `RUST_LOG=debug cargo run`

### Test Failures
- Ensure `VERIDION_MASTER_KEY` is set for integration tests
- Run tests with: `cargo test --test integration_test -- --nocapture`

## Next Steps

1. **Configure Signicat** (Optional):
   - Get API credentials from Signicat
   - Set `USE_REAL_API=true`
   - Configure `SIGNICAT_CLIENT_ID` and `SIGNICAT_CLIENT_SECRET`

2. **Test Priority 1 Features**:
   - Use Swagger UI to test all endpoints
   - Run integration tests
   - Test with Python agent

3. **Deploy to Production**:
   - Set secure `VERIDION_MASTER_KEY`
   - Configure proper logging
   - Set up monitoring

## Support

For issues and questions, please refer to the main README.md or open an issue.
