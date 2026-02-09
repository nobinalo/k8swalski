# syntax=docker/dockerfile:1

# Runtime: Ultra-minimal static base (no shell, no package manager)
# Runs as non-root user 65532 by default
FROM chainguard/static:latest

# OCI standard labels
LABEL org.opencontainers.image.title="k8swalski" \
      org.opencontainers.image.description="HTTP/HTTPS echo server for debugging and testing" \
      org.opencontainers.image.source="https://github.com/audacioustux/k8swalski" \
      org.opencontainers.image.licenses="MIT"

# Set working directory
WORKDIR /app

# Copy pre-built binary (from GHA artifact)
COPY artifact/k8swalski ./k8swalski

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
