# syntax=docker/dockerfile:1

ARG UID=65532
ARG GID=65532

# Runtime stage
FROM chainguard/wolfi-base:latest

ARG UID
ARG GID

LABEL org.opencontainers.image.title="k8swalski"
LABEL org.opencontainers.image.description="HTTP/HTTPS echo server for debugging and testing"
LABEL org.opencontainers.image.source="https://github.com/audacioustux/k8swalski"
LABEL org.opencontainers.image.licenses="MIT"

WORKDIR /app

# Copy pre-built binary from artifact
COPY artifact/k8swalski /app/k8swalski

# Run as nonroot user
USER ${UID}:${GID}

# Expose ports
EXPOSE 8080 8443

# Run the application
ENTRYPOINT ["/app/k8swalski"]
