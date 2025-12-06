# Environment Variables Configuration

This document describes all environment variables used by Veridion Nexus.

## Quick Start

Create a `.env` file in the project root with the following variables:

```bash
# Database
DATABASE_URL=postgresql://veridion:veridion_password@localhost:5432/veridion_nexus

# Security
JWT_SECRET=your-secret-key-min-32-chars
ALLOWED_ORIGINS=https://yourdomain.com,https://api.yourdomain.com

# Server
PORT=8080
RUST_LOG=info

# Rate Limiting
RATE_LIMIT_REQUESTS_PER_MINUTE=100
RATE_LIMIT_WINDOW_SECONDS=60
```

## Complete Variable Reference

### Database Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | `postgresql://veridion:veridion_password@localhost:5432/veridion_nexus` | PostgreSQL connection string |

**Format:** `postgresql://[user]:[password]@[host]:[port]/[database]`

**Example:**
```bash
DATABASE_URL=postgresql://veridion:secure_password@db.example.com:5432/veridion_nexus
```

### Security Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `JWT_SECRET` | Yes | None | Secret key for JWT token signing (minimum 32 characters) |
| `ALLOWED_ORIGINS` | No | `*` | Comma-separated list of allowed CORS origins |

**JWT_SECRET:**
- Must be at least 32 characters long
- Use a strong, randomly generated string in production
- Generate with: `openssl rand -base64 32`

**ALLOWED_ORIGINS:**
- Development: Use `*` to allow all origins
- Production: Specify exact origins (e.g., `https://yourdomain.com,https://app.yourdomain.com`)

**Examples:**
```bash
# Development
ALLOWED_ORIGINS=*

# Production
ALLOWED_ORIGINS=https://yourdomain.com,https://api.yourdomain.com
```

### Server Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PORT` | No | `8080` | Port number for the HTTP server |
| `RUST_LOG` | No | `info` | Log level (trace, debug, info, warn, error) |

**Examples:**
```bash
PORT=8080
RUST_LOG=info
```

### Rate Limiting Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `RATE_LIMIT_REQUESTS_PER_MINUTE` | No | `100` | Maximum requests per minute per IP address |
| `RATE_LIMIT_WINDOW_SECONDS` | No | `60` | Time window in seconds for rate limiting |

**Examples:**
```bash
RATE_LIMIT_REQUESTS_PER_MINUTE=100
RATE_LIMIT_WINDOW_SECONDS=60
```

### Background Worker Configuration

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `WEBHOOK_RETRY_INTERVAL` | No | `60` | Webhook delivery retry interval in seconds |
| `RETENTION_CHECK_INTERVAL` | No | `3600` | Retention deletion check interval in seconds (1 hour) |
| `VIEW_REFRESH_INTERVAL` | No | `3600` | Materialized view refresh interval in seconds (1 hour) |

**Examples:**
```bash
WEBHOOK_RETRY_INTERVAL=60
RETENTION_CHECK_INTERVAL=3600
VIEW_REFRESH_INTERVAL=3600
```

## Environment-Specific Configuration

### Development

```bash
DATABASE_URL=postgresql://veridion:veridion_password@localhost:5432/veridion_nexus
JWT_SECRET=dev-secret-key-minimum-32-characters-long
ALLOWED_ORIGINS=*
PORT=8080
RUST_LOG=debug
```

### Staging

```bash
DATABASE_URL=postgresql://veridion:staging_password@staging-db.example.com:5432/veridion_nexus
JWT_SECRET=staging-secret-key-minimum-32-characters-long
ALLOWED_ORIGINS=https://staging.yourdomain.com
PORT=8080
RUST_LOG=info
```

### Production

```bash
DATABASE_URL=postgresql://veridion:production_password@prod-db.example.com:5432/veridion_nexus
JWT_SECRET=<strong-random-32-char-secret>
ALLOWED_ORIGINS=https://yourdomain.com,https://api.yourdomain.com
PORT=8080
RUST_LOG=warn
```

## Security Best Practices

1. **Never commit `.env` files to version control**
   - Add `.env` to `.gitignore`
   - Use `.env.example` as a template (without sensitive values)

2. **Use strong secrets in production**
   - Generate JWT_SECRET with: `openssl rand -base64 32`
   - Use different secrets for each environment
   - Rotate secrets regularly

3. **Restrict CORS origins in production**
   - Never use `*` in production
   - Specify exact domains that need access

4. **Use secrets management services**
   - AWS Secrets Manager
   - HashiCorp Vault
   - Azure Key Vault
   - Kubernetes Secrets

5. **Environment variable injection**
   - Use CI/CD pipelines to inject secrets
   - Never hardcode secrets in code
   - Use environment-specific configuration files

## Loading Environment Variables

### From .env file

The application automatically loads variables from a `.env` file in the project root (if using `dotenv` crate).

### From system environment

Set variables in your shell:

```bash
export DATABASE_URL="postgresql://..."
export JWT_SECRET="..."
```

### From Docker

```bash
docker run -e DATABASE_URL="..." -e JWT_SECRET="..." veridion-nexus
```

Or use a `.env` file with docker-compose:

```yaml
services:
  api:
    env_file:
      - .env
```

### From systemd

Add to service file:

```ini
[Service]
Environment="DATABASE_URL=postgresql://..."
Environment="JWT_SECRET=..."
```

## Validation

The application validates required environment variables on startup. If a required variable is missing, the application will:

1. Log an error message
2. Exit with a non-zero status code
3. Display which variable is missing

## Troubleshooting

### Variable not being read

1. Check if the variable is set:
   ```bash
   echo $DATABASE_URL
   ```

2. Check if `.env` file exists and is readable:
   ```bash
   ls -la .env
   cat .env
   ```

3. Restart the application after changing variables

### Invalid format

- `DATABASE_URL`: Must be a valid PostgreSQL connection string
- `ALLOWED_ORIGINS`: Must be comma-separated URLs or `*`
- `JWT_SECRET`: Must be at least 32 characters

### Security warnings

If you see warnings about insecure configuration:
- Change `ALLOWED_ORIGINS=*` in production
- Use strong `JWT_SECRET` values
- Review rate limiting settings

