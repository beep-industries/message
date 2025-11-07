# Stage 1: Build
FROM rust:alpine AS builder

# Install musl-dev for static linking
RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application with static linking
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Runtime (scratch)
FROM scratch

# Copy the binary from builder
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/message /message

EXPOSE 3000

# Set the entrypoint
ENTRYPOINT ["/message"]