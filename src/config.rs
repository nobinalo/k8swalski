use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "k8swalski")]
#[command(author, version, about)]
#[command(long_about = "k8swalski is an HTTP/HTTPS echo server designed for debugging and testing.

It echoes back request details including headers, body, query parameters, and more.
Perfect for testing webhooks, debugging API clients, or understanding HTTP requests.

Features:
  • Dual HTTP and HTTPS support with configurable ports
  • Request/response logging in human-readable or JSON format
  • Optional JWT token decoding
  • Prometheus metrics endpoint
  • CORS support with flexible configuration
  • Configurable request body and header size limits")]
pub struct Config {
    /// HTTP port to listen on
    #[arg(long, env = "HTTP_PORT", default_value = "8080")]
    pub http_port: u16,

    /// HTTPS port to listen on
    #[arg(long, env = "HTTPS_PORT", default_value = "8443")]
    pub https_port: u16,

    /// Path to TLS certificate file
    #[arg(long, env = "TLS_CERT_PATH", default_value = "/tmp/cert.pem")]
    pub tls_cert_path: PathBuf,

    /// Path to the TLS key file
    #[arg(long, env = "TLS_KEY_PATH", default_value = "/tmp/key.pem")]
    pub tls_key_path: PathBuf,

    /// Maximum request body size in bytes
    #[arg(long, env = "MAX_BODY_SIZE", default_value = "10485760")]
    pub max_body_size: usize,

    /// Log format: "human" or "json"
    #[arg(long, env = "LOG_FORMAT", default_value = "human")]
    pub log_format: LogFormat,

    /// Disable request logging
    #[arg(long, env = "DISABLE_REQUEST_LOGS")]
    pub disable_request_logs: bool,

    /// Regex pattern for paths to ignore in logs
    #[arg(long, env = "LOG_IGNORE_PATH")]
    pub log_ignore_path: Option<String>,

    /// Include environment variables in response
    #[arg(long, env = "INCLUDE_ENV_VARS")]
    pub include_env_vars: bool,

    /// Decode JWT tokens in Authorization header
    #[cfg(feature = "jwt")]
    #[arg(long, env = "JWT_HEADER")]
    pub jwt_header: Option<String>,

    /// Enable Prometheus metrics endpoint
    #[cfg(feature = "prometheus")]
    #[arg(long, env = "PROMETHEUS")]
    pub prometheus: bool,

    /// Enable CORS
    #[arg(long, env = "ENABLE_CORS")]
    pub enable_cors: bool,

    /// CORS Allow-Origin header value
    #[arg(long, env = "CORS_ALLOW_ORIGIN")]
    pub cors_allow_origin: Option<String>,

    /// CORS Allow-Methods header value
    #[arg(long, env = "CORS_ALLOW_METHODS")]
    pub cors_allow_methods: Option<String>,

    /// CORS Allow-Headers header value
    #[arg(long, env = "CORS_ALLOW_HEADERS")]
    pub cors_allow_headers: Option<String>,

    /// CORS Allow-Credentials header value
    #[arg(long, env = "CORS_ALLOW_CREDENTIALS")]
    pub cors_allow_credentials: Option<String>,

    /// Maximum header size in bytes
    #[arg(long, env = "MAX_HEADER_SIZE", default_value = "16384")]
    pub max_header_size: usize,

    /// Disable echoing response back to client (send empty response)
    #[arg(long, env = "ECHO_BACK_TO_CLIENT")]
    pub echo_back_to_client: Option<bool>,

    /// Log JSON output without newlines
    #[arg(long, env = "LOG_WITHOUT_NEWLINE")]
    pub log_without_newline: bool,

    /// Override response body with file content (path relative to current directory)
    #[arg(long, env = "OVERRIDE_RESPONSE_BODY_FILE_PATH")]
    pub override_response_body_file_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Human,
    Json,
}

impl std::str::FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "human" => Ok(LogFormat::Human),
            "json" => Ok(LogFormat::Json),
            _ => Err(format!("Invalid log format: {}. Use 'human' or 'json'", s)),
        }
    }
}

impl std::fmt::Display for LogFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogFormat::Human => write!(f, "human"),
            LogFormat::Json => write!(f, "json"),
        }
    }
}
