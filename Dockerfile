# Multi-stage build for smaller final image
FROM rust:1.75 as builder

WORKDIR /app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Now copy the real source code
COPY src ./src
COPY templates ./templates
COPY static ./static
COPY migrations ./migrations

# Build the actual application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/error-report-web /app/
COPY --from=builder /app/templates ./templates
COPY --from=builder /app/static ./static
COPY --from=builder /app/migrations ./migrations

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Environment variables
ENV RUST_LOG=info
ENV BIND_ADDRESS=0.0.0.0
ENV PORT=8000

EXPOSE 8000

CMD ["./error-report-web"]
