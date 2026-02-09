# syntax=docker/dockerfile:1

FROM cgr.dev/chainguard/static:latest AS runtime

LABEL org.opencontainers.image.title="k8swalski"
LABEL org.opencontainers.image.description="HTTP/HTTPS echo server for debugging and testing"
LABEL org.opencontainers.image.source="https://github.com/audacioustux/k8swalski"
LABEL org.opencontainers.image.licenses="MIT"

# Ensure we run as the non-root 'nonroot' user provided by the base
USER 65532:65532

WORKDIR /app

# Copy binary directly from your local build context or previous stage
# Assuming the binary is built for linux/amd64 or arm64
COPY --chmod=755 artifact/k8swalski ./k8swalski

EXPOSE 8080 8443

ENTRYPOINT ["./k8swalski"]