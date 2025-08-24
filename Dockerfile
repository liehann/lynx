# Multi-stage build for Rust application  
FROM rust:1.87-slim as builder

# Install system dependencies needed for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY templates ./templates
COPY migrations ./migrations
COPY static ./static

# Build the application in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false -m -d /app lynx

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/lynx /app/lynx

# Copy templates, migrations, and static files
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/static /app/static

# Change ownership to app user
RUN chown -R lynx:lynx /app

# Switch to app user
USER lynx

# Expose port (default 3000, but configurable via PORT env var)
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${PORT:-3000}/api/links || exit 1

# Run the application
CMD ["./lynx"]
