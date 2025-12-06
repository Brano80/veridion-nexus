# Production Deployment Guide

## Prerequisites

- PostgreSQL 14+ database
- Rust 1.70+ (for building from source)
- Docker & Docker Compose (optional, for containerized deployment)
- SSL/TLS certificates (for HTTPS)

## Environment Variables

Create a `.env` file in the project root with the following variables:

```bash
# Database
DATABASE_URL=postgresql://user:password@host:5432/veridion_nexus

# Security
JWT_SECRET=your-secret-key-min-32-chars
ALLOWED_ORIGINS=https://yourdomain.com,https://api.yourdomain.com

# Server
PORT=8080
RUST_LOG=info

# Rate Limiting (optional)
RATE_LIMIT_REQUESTS_PER_MINUTE=100
RATE_LIMIT_WINDOW_SECONDS=60
```

See `.env.example` for a complete list of all environment variables.

## Deployment Options

### Option 1: Docker Compose (Recommended)

1. **Build the Docker image:**
   ```bash
   docker-compose build
   ```

2. **Start services:**
   ```bash
   docker-compose up -d
   ```

3. **Run migrations:**
   ```bash
   docker-compose exec api sqlx migrate run
   ```

4. **Check logs:**
   ```bash
   docker-compose logs -f api
   ```

### Option 2: Systemd Service (Linux)

1. **Build the application:**
   ```bash
   cargo build --release
   ```

2. **Create systemd service file** (`/etc/systemd/system/veridion-nexus.service`):
   ```ini
   [Unit]
   Description=Veridion Nexus API
   After=network.target postgresql.service

   [Service]
   Type=simple
   User=veridion
   WorkingDirectory=/opt/veridion-nexus
   Environment="DATABASE_URL=postgresql://user:password@localhost:5432/veridion_nexus"
   Environment="JWT_SECRET=your-secret-key"
   Environment="ALLOWED_ORIGINS=https://yourdomain.com"
   ExecStart=/opt/veridion-nexus/target/release/veridion-nexus
   Restart=always
   RestartSec=10

   [Install]
   WantedBy=multi-user.target
   ```

3. **Enable and start the service:**
   ```bash
   sudo systemctl enable veridion-nexus
   sudo systemctl start veridion-nexus
   sudo systemctl status veridion-nexus
   ```

### Option 3: Manual Deployment

1. **Build release binary:**
   ```bash
   cargo build --release
   ```

2. **Run migrations:**
   ```bash
   export DATABASE_URL="postgresql://user:password@host:5432/veridion_nexus"
   sqlx migrate run
   ```

3. **Start the server:**
   ```bash
   ./target/release/veridion-nexus
   ```

## SSL/TLS Configuration

### Using Nginx as Reverse Proxy

1. **Install Nginx:**
   ```bash
   sudo apt-get install nginx certbot python3-certbot-nginx
   ```

2. **Create Nginx configuration** (`/etc/nginx/sites-available/veridion-nexus`):
   ```nginx
   server {
       listen 80;
       server_name yourdomain.com;

       location / {
           return 301 https://$server_name$request_uri;
       }
   }

   server {
       listen 443 ssl http2;
       server_name yourdomain.com;

       ssl_certificate /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
       ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;
       ssl_protocols TLSv1.2 TLSv1.3;
       ssl_ciphers HIGH:!aNULL:!MD5;

       location / {
           proxy_pass http://localhost:8080;
           proxy_set_header Host $host;
           proxy_set_header X-Real-IP $remote_addr;
           proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
           proxy_set_header X-Forwarded-Proto $scheme;
       }
   }
   ```

3. **Enable the site:**
   ```bash
   sudo ln -s /etc/nginx/sites-available/veridion-nexus /etc/nginx/sites-enabled/
   sudo nginx -t
   sudo systemctl reload nginx
   ```

4. **Obtain SSL certificate:**
   ```bash
   sudo certbot --nginx -d yourdomain.com
   ```

## Database Setup

1. **Create database:**
   ```sql
   CREATE DATABASE veridion_nexus;
   CREATE USER veridion WITH PASSWORD 'secure_password';
   GRANT ALL PRIVILEGES ON DATABASE veridion_nexus TO veridion;
   ```

2. **Run migrations:**
   ```bash
   export DATABASE_URL="postgresql://veridion:secure_password@localhost:5432/veridion_nexus"
   sqlx migrate run
   ```

3. **Create initial admin user:**
   ```sql
   -- Default admin user (username: admin, password: admin123)
   -- CHANGE THE PASSWORD IMMEDIATELY IN PRODUCTION!
   -- Use the /api/v1/auth/login endpoint to get a token, then update password via API
   ```

## Security Hardening

### 1. Change Default Admin Password

After first login, immediately change the default admin password:

```bash
curl -X POST https://yourdomain.com/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "admin123"}'

# Use the token to update password (implement password change endpoint if needed)
```

### 2. Configure CORS

Set `ALLOWED_ORIGINS` environment variable to your production domains:

```bash
ALLOWED_ORIGINS=https://yourdomain.com,https://app.yourdomain.com
```

### 3. Enable Rate Limiting

Adjust rate limits based on your needs:

```bash
RATE_LIMIT_REQUESTS_PER_MINUTE=100
RATE_LIMIT_WINDOW_SECONDS=60
```

### 4. Firewall Configuration

Only expose necessary ports:

```bash
# Allow HTTPS
sudo ufw allow 443/tcp

# Allow SSH (if needed)
sudo ufw allow 22/tcp

# Enable firewall
sudo ufw enable
```

## Monitoring & Health Checks

### Health Check Endpoint

The API provides a health check endpoint:

```bash
curl http://localhost:8080/health
```

### Logging

Logs are written to stdout/stderr. For production, consider:

1. **Systemd journal:**
   ```bash
   sudo journalctl -u veridion-nexus -f
   ```

2. **Log aggregation:**
   - Use a log aggregation service (e.g., ELK, Loki, Datadog)
   - Configure log forwarding from systemd or Docker

### Database Backup

Set up regular database backups:

```bash
# Daily backup script
#!/bin/bash
pg_dump -U veridion veridion_nexus > /backups/veridion_nexus_$(date +%Y%m%d).sql
```

## Performance Tuning

### Database Connection Pooling

The application uses connection pooling. Adjust pool size in `src/database.rs` if needed:

```rust
.max_connections(20)  // Adjust based on your load
```

### Background Workers

Background workers handle:
- Webhook deliveries
- Retention policy deletions
- Materialized view refreshes

Monitor their performance and adjust intervals if needed.

## Troubleshooting

### Application won't start

1. Check database connectivity:
   ```bash
   psql $DATABASE_URL -c "SELECT 1;"
   ```

2. Check environment variables:
   ```bash
   env | grep -E "(DATABASE_URL|JWT_SECRET|ALLOWED_ORIGINS)"
   ```

3. Check logs:
   ```bash
   docker-compose logs api
   # or
   sudo journalctl -u veridion-nexus -n 50
   ```

### Database migration errors

1. Check migration status:
   ```bash
   sqlx migrate info
   ```

2. Rollback if needed:
   ```bash
   sqlx migrate revert
   ```

### High memory usage

1. Check connection pool size
2. Review background worker intervals
3. Monitor database query performance

## Updates & Maintenance

### Updating the Application

1. **Pull latest code:**
   ```bash
   git pull origin main
   ```

2. **Build new version:**
   ```bash
   cargo build --release
   ```

3. **Run new migrations:**
   ```bash
   sqlx migrate run
   ```

4. **Restart service:**
   ```bash
   sudo systemctl restart veridion-nexus
   # or
   docker-compose restart api
   ```

### Dependency Updates

Before updating dependencies:

1. **Check for vulnerabilities:**
   ```bash
   ./scripts/check-vulnerabilities.sh
   # or on Windows:
   .\scripts\check-vulnerabilities.ps1
   ```

2. **Update dependencies:**
   ```bash
   cargo update
   ```

3. **Test thoroughly before deploying**

## Support

For issues or questions:
- Check logs first
- Review this guide
- Check GitHub issues
- Contact support team

