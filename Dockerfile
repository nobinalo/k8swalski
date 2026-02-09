# syntax=docker/dockerfile:1

# Global args for nonroot user
ARG UID=65532
ARG GID=65532

# Builder stage
FROM chainguard/rust:latest AS builder

ARG UID
ARG GID

WORKDIR /app

# Install sccache (cached in /tmp to avoid permission issues)
RUN --mount=type=cache,target=/tmp/cargo-cache,uid=${UID},gid=${GID},sharing=locked \
    if [ ! -f /tmp/cargo-cache/sccache ]; then \
        cargo install sccache --version 0.13.0 --locked --root /tmp/cargo-cache; \
    fi && \
    cp /tmp/cargo-cache/bin/sccache /home/nonroot/.cargo/bin/sccache

# Configure sccache environment
ENV SCCACHE_DIR=/home/nonroot/.cache/sccache \
    SCCACHE_CACHE_SIZE=2G \
    RUSTC_WRAPPER=/home/nonroot/.cargo/bin/sccache

# Copy source code
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY src ./src

# Build application with sccache
RUN --mount=type=cache,target=/home/nonroot/.cache/sccache,uid=${UID},gid=${GID},sharing=locked \
    --mount=type=cache,target=/home/nonroot/.cargo/registry,uid=${UID},gid=${GID},sharing=locked \
    --mount=type=cache,target=/home/nonroot/.cargo/git,uid=${UID},gid=${GID},sharing=locked \
    cargo build --release && \
    sccache --show-stats && \
    cp target/release/k8swalski /tmp/k8swalski && \
    strip /tmp/k8swalski

# Runtime stage
FROM chainguard/wolfi-base:latest

ARG UID
ARG GID

LABEL org.opencontainers.image.title="k8swalski" \
      org.opencontainers.image.description="HTTP/HTTPS echo server for debugging and testing" \
      org.opencontainers.image.source="https://github.com/audacioustux/k8swalski" \
      org.opencontainers.image.licenses="MIT"

# Install curl for healthchecks
RUN apk add --no-cache curl

WORKDIR /app

# Copy binary from builder
COPY --from=builder /tmp/k8swalski /app/k8swalski

# Run as nonroot user
USER ${UID}:${GID}

# Expose ports
EXPOSE 8080 8443

# Run the application
ENTRYPOINT ["/app/k8swalski"]
