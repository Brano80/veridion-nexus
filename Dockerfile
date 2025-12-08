# Stage 1: Builder
FROM rust:latest as builder

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

# Build the actual application
RUN cargo build --release

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
