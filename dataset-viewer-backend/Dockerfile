# Build stage
FROM rust:1.75 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libcurl4 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false dataset-viewer

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/server /app/dataset-viewer-backend

# Copy docs
COPY docs ./docs

# Change ownership
RUN chown -R dataset-viewer:dataset-viewer /app

USER dataset-viewer

EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["./dataset-viewer-backend"]