# syntax=docker/dockerfile:1

# Builder stage
FROM chainguard/rust:latest AS builder

USER 65532:65532

WORKDIR /app

# Copy source code
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY src ./src

# Build application
RUN --mount=type=cache,target=/home/nonroot/.cargo/registry,sharing=locked,uid=65532,gid=65532 \
    --mount=type=cache,target=/home/nonroot/.cargo/git,sharing=locked,uid=65532,gid=65532 \
    <<EOF
set -e

cargo build --release

# Prepare binary
cp target/release/k8swalski /tmp/k8swalski
strip /tmp/k8swalski
EOF

# Runtime stage
FROM chainguard/wolfi-base:latest

LABEL org.opencontainers.image.title="k8swalski"
LABEL org.opencontainers.image.description="HTTP/HTTPS echo server for debugging and testing"
LABEL org.opencontainers.image.source="https://github.com/audacioustux/k8swalski"
LABEL org.opencontainers.image.licenses="MIT"

# Install curl for healthchecks
RUN apk add --no-cache curl

WORKDIR /app

# Copy binary from builder
COPY --from=builder /tmp/k8swalski /app/k8swalski

# Run as nonroot user
USER 65532:65532

# Expose ports
EXPOSE 8080 8443

# Run the application
ENTRYPOINT ["/app/k8swalski"]
