# Build stage
FROM rust:1.92-alpine AS builder

RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static protobuf-dev

# Static linking for scratch image
ENV OPENSSL_STATIC=1

WORKDIR /app

# Copy all source files
COPY Cargo.toml Cargo.lock ./
COPY api/ api/
COPY core/ core/

# Build the application (statically linked with musl)
RUN cargo build --release --bin api

# Runtime stage
FROM scratch

# Copy CA certificates for HTTPS (Keycloak, MongoDB TLS)
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy binary from builder
COPY --from=builder /app/target/release/api /api

# Copy configuration files
COPY config/ /config/

# Set default environment variables
ENV ROUTING_CONFIG_PATH=/config/routing.yaml
ENV API_PORT=3001
ENV HEALTH_PORT=9090
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt

# Expose both server ports
EXPOSE 3001 9090

# Run the binary
ENTRYPOINT ["/api"]
