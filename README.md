# k8swalski

[![Build](https://img.shields.io/github/actions/workflow/status/audacioustux/k8swalski/build-push-ghcr.yml?style=flat-square)](https://github.com/audacioustux/k8swalski/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)

HTTP/HTTPS echo server for debugging and testing. Built with Rust (axum + tokio).

## Quick Start

```bash
# Docker
docker run -p 8080:8080 -p 8443:8443 ghcr.io/audacioustux/k8swalski:latest

# Test it
curl http://localhost:8080/test
```

## Configuration

<!-- BEGIN_CLI_HELP -->
```
/home/runner/work/k8swalski/k8swalski/.pre-commit-config.yaml
pre-commit installed at .git/hooks/pre-commit
k8swalski dev environment loaded
Run 'task --list' to see available tasks
k8swalski is an HTTP/HTTPS echo server designed for debugging and testing.

It echoes back request details including headers, body, query parameters, and more.
Perfect for testing webhooks, debugging API clients, or understanding HTTP requests.

Features:
  • Dual HTTP and HTTPS support with configurable ports
  • Request/response logging in human-readable or JSON format
  • Optional JWT token decoding
  • Prometheus metrics endpoint
  • CORS support with flexible configuration
  • Configurable request body and header size limits

Usage: k8swalski [OPTIONS]

Options:
      --http-port <HTTP_PORT>
          HTTP port to listen on
          
          [env: HTTP_PORT=]
          [default: 8080]

      --https-port <HTTPS_PORT>
          HTTPS port to listen on
          
          [env: HTTPS_PORT=]
          [default: 8443]

      --tls-cert-path <TLS_CERT_PATH>
          Path to TLS certificate file
          
          [env: TLS_CERT_PATH=]
          [default: /tmp/cert.pem]

      --tls-key-path <TLS_KEY_PATH>
          Path to the TLS key file
          
          [env: TLS_KEY_PATH=]
          [default: /tmp/key.pem]

      --max-body-size <MAX_BODY_SIZE>
          Maximum request body size in bytes
          
          [env: MAX_BODY_SIZE=]
          [default: 10485760]

      --log-format <LOG_FORMAT>
          Log format: "human" or "json"
          
          [env: LOG_FORMAT=]
          [default: human]

      --disable-request-logs
          Disable request logging
          
          [env: DISABLE_REQUEST_LOGS=]

      --log-ignore-path <LOG_IGNORE_PATH>
          Regex pattern for paths to ignore in logs
          
          [env: LOG_IGNORE_PATH=]

      --include-env-vars
          Include environment variables in response
          
          [env: INCLUDE_ENV_VARS=]

      --jwt-header <JWT_HEADER>
          Decode JWT tokens in Authorization header
          
          [env: JWT_HEADER=]

      --prometheus
          Enable Prometheus metrics endpoint
          
          [env: PROMETHEUS=]

      --enable-cors
          Enable CORS
          
          [env: ENABLE_CORS=]

      --cors-allow-origin <CORS_ALLOW_ORIGIN>
          CORS Allow-Origin header value
          
          [env: CORS_ALLOW_ORIGIN=]

      --cors-allow-methods <CORS_ALLOW_METHODS>
          CORS Allow-Methods header value
          
          [env: CORS_ALLOW_METHODS=]

      --cors-allow-headers <CORS_ALLOW_HEADERS>
          CORS Allow-Headers header value
          
          [env: CORS_ALLOW_HEADERS=]

      --cors-allow-credentials <CORS_ALLOW_CREDENTIALS>
          CORS Allow-Credentials header value
          
          [env: CORS_ALLOW_CREDENTIALS=]

      --max-header-size <MAX_HEADER_SIZE>
          Maximum header size in bytes
          
          [env: MAX_HEADER_SIZE=]
          [default: 16384]

      --echo-back-to-client <ECHO_BACK_TO_CLIENT>
          Disable echoing response back to client (send empty response)
          
          [env: ECHO_BACK_TO_CLIENT=]
          [possible values: true, false]

      --log-without-newline
          Log JSON output without newlines
          
          [env: LOG_WITHOUT_NEWLINE=]

      --override-response-body-file-path <OVERRIDE_RESPONSE_BODY_FILE_PATH>
          Override response body with file content (path relative to current directory)
          
          [env: OVERRIDE_RESPONSE_BODY_FILE_PATH=]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
<!-- END_CLI_HELP -->

## Development

See [Taskfile.yml](Taskfile.yml) for available development tasks.

## License

MIT
