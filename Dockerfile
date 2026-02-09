# syntax=docker/dockerfile:1

# Build arguments for security
ARG UID=65532
ARG GID=65532

# Runtime: Minimal Chainguard Wolfi base
FROM chainguard/wolfi-base:latest

# Re-declare after FROM for use in later stages
ARG UID
ARG GID

# OCI standard labels
LABEL org.opencontainers.image.title="k8swalski" \
      org.opencontainers.image.description="HTTP/HTTPS echo server for debugging and testing" \
      org.opencontainers.image.source="https://github.com/audacioustux/k8swalski" \
      org.opencontainers.image.licenses="MIT"

# Set working directory
WORKDIR /app

# Copy pre-built binary (from GHA artifact)
COPY --chown=${UID}:${GID} artifact/k8swalski ./k8swalski

# Switch to non-root user for security
USER ${UID}:${GID}

# Expose HTTP and HTTPS ports
EXPOSE 8080 8443

# Health check using native binary capability
HEALTHCHECK --interval=10s \
            --timeout=5s \
            --start-period=10s \
            --retries=3 \
  CMD ["./k8swalski", "--check-health"]

# Run the application
ENTRYPOINT ["./k8swalski"]
