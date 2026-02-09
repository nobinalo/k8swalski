# syntax=docker/dockerfile:1

FROM cgr.dev/chainguard/static:latest AS runtime

LABEL org.opencontainers.image.title="k8swalski"
LABEL org.opencontainers.image.description="HTTP/HTTPS echo server for debugging and testing"
LABEL org.opencontainers.image.source="https://github.com/audacioustux/k8swalski"
LABEL org.opencontainers.image.licenses="MIT"

USER 65532:65532

WORKDIR /app

COPY --chmod=755 artifact/k8swalski ./k8swalski

EXPOSE 8080 8443

ENTRYPOINT ["./k8swalski"]