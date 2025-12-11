# Stage 1: Builder
FROM rust:latest as builder

# Install PostgreSQL client and server for sqlx compile-time checking
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    postgresql-client \
    postgresql \
    && rm -rf /var/lib/apt/lists/*

# Install sqlx-cli for query preparation
RUN cargo install sqlx-cli --no-default-features --features postgres

WORKDIR /app

# Create a dummy project to cache dependencies
# Using --lib because Cargo.toml defines [lib] section
RUN cargo new --lib veridion-nexus
WORKDIR /app/veridion-nexus

# Create dummy main.rs for binary (since we have both lib and bin)
RUN touch src/main.rs

# Copy dependency files
COPY Cargo.toml Cargo.lock* ./

# Build dependencies (this layer will be cached if Cargo.toml doesn't change)
# Use || true to continue even if build fails (we just want to cache deps)
RUN cargo build --release || true

# Remove dummy source files
RUN rm -rf src/*.rs

# Copy source code
COPY src ./src

# Copy migrations directory (required for sqlx::migrate! macro at compile time)
COPY migrations ./migrations

# Set DATABASE_URL for sqlx compile-time checking
ENV DATABASE_URL=postgresql://veridion:veridion_secure_pass_2024@localhost:5432/veridion_nexus

# Start PostgreSQL service, set up database, apply migrations, and build
# All in one RUN to ensure PostgreSQL stays running throughout
# Note: postgresql is already installed above, but we install postgresql-contrib here
RUN apt-get update && apt-get install -y postgresql postgresql-contrib && \
    service postgresql start && \
    echo "Waiting for Postgres..." && sleep 10 && \
    su - postgres -c "psql -c \"CREATE USER veridion WITH PASSWORD 'veridion_secure_pass_2024';\"" || true && \
    su - postgres -c "psql -c \"ALTER USER veridion WITH PASSWORD 'veridion_secure_pass_2024';\"" || true && \
    su - postgres -c "psql -c \"CREATE DATABASE veridion_nexus OWNER veridion;\"" || true && \
    su - postgres -c "psql -c \"GRANT ALL PRIVILEGES ON DATABASE veridion_nexus TO veridion;\"" || true && \
    # Apply migrations in sorted order
    for f in $(ls migrations/*.sql | sort); do \
        echo "Applying $f..."; \
        PGPASSWORD=veridion_secure_pass_2024 psql -h localhost -U veridion -d veridion_nexus -f "/app/veridion-nexus/$f" || true; \
    done && \
    # Build with correct URL
    export DATABASE_URL=postgresql://veridion:veridion_secure_pass_2024@localhost:5432/veridion_nexus && \
    cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install required system dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for security
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/veridion-nexus/target/release/veridion-nexus /app/veridion-nexus

# Change ownership to non-root user
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose the API port
EXPOSE 8080

# Run the binary
CMD ["./veridion-nexus"]
