use axum::{
    Json,
    body::Body,
    extract::{ConnectInfo, Query, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::time::sleep;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub hostname: String,
}

#[derive(Debug, Deserialize)]
pub struct EchoQueryParams {
    #[serde(rename = "x-set-response-status-code")]
    response_status_code: Option<u16>,

    #[serde(rename = "x-set-response-delay-ms")]
    response_delay_ms: Option<u64>,

    #[serde(rename = "x-set-response-content-type")]
    response_content_type: Option<String>,

    #[serde(rename = "response_body_only")]
    response_body_only: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct EchoResponse {
    pub path: String,
    pub headers: HashMap<String, String>,
    pub method: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<HashMap<String, String>>,
    pub fresh: bool,
    pub hostname: String,
    pub ip: String,
    pub ips: Vec<String>,
    pub protocol: String,
    pub query: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdomains: Option<Vec<String>>,
    pub xhr: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<OsInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection: Option<ConnectionInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, String>>,
    #[cfg(feature = "jwt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<JwtInfo>,
    #[cfg(feature = "mtls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_cert: Option<ClientCertInfo>,
}

#[derive(Debug, Serialize)]
pub struct OsInfo {
    pub hostname: String,
}

#[derive(Debug, Serialize)]
pub struct ConnectionInfo {
    pub servername: Option<String>,
}

#[cfg(feature = "jwt")]
#[derive(Debug, Serialize)]
pub struct JwtInfo {
    pub header: Option<Value>,
    pub payload: Option<Value>,
}

#[cfg(feature = "mtls")]
#[derive(Debug, Serialize)]
pub struct ClientCertInfo {
    pub subject: String,
    pub issuer: String,
    pub subjectaltname: Option<String>,
    pub info: Option<String>,
}

pub async fn echo_handler(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(query_params): Query<EchoQueryParams>,
    request: Request,
) -> Response {
    // Extract request parts before consuming the body
    let (parts, body) = request.into_parts();
    let headers = &parts.headers;
    let method = parts.method.to_string();
    let uri = &parts.uri;
    let path = uri.path().to_string();
    let protocol = format!("{:?}", parts.version);

    // Check if we should override response with file content
    if let Some(file_path) = &state.config.override_response_body_file_path {
        return serve_file(file_path).await;
    }

    // Extract status code override
    let status_code = query_params
        .response_status_code
        .or_else(|| {
            headers
                .get("x-set-response-status-code")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
        })
        .and_then(|code| StatusCode::from_u16(code).ok())
        .unwrap_or(StatusCode::OK);

    // Extract delay
    let delay_ms = query_params.response_delay_ms.or_else(|| {
        headers
            .get("x-set-response-delay-ms")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
    });

    // Extract custom content-type
    let content_type = query_params.response_content_type.or_else(|| {
        headers
            .get("x-set-response-content-type")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string())
    });

    // Apply delay if requested
    if let Some(delay) = delay_ms {
        sleep(Duration::from_millis(delay)).await;
    }

    // Parse query parameters
    let query: HashMap<String, String> = uri
        .query()
        .map(|q| {
            url::form_urlencoded::parse(q.as_bytes())
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Convert headers to HashMap
    let mut headers_map: HashMap<String, String> = HashMap::new();
    for (key, value) in headers.iter() {
        if let Ok(v) = value.to_str() {
            headers_map.insert(key.to_string(), v.to_string());
        }
    }

    // Extract IP information
    let ip = addr.ip().to_string();
    let ips = extract_ips(headers, &ip);

    // Check if XHR request
    let xhr = headers
        .get("x-requested-with")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("XMLHttpRequest"))
        .unwrap_or(false);

    // Extract cookies
    let cookies = extract_cookies(headers);

    // Extract subdomains
    let subdomains = extract_subdomains(headers);

    // Read body
    let body_bytes =
        axum::body::to_bytes(body, state.config.max_body_size).await.unwrap_or_default();

    // Decompress body if gzipped
    let body_bytes = if headers
        .get("content-encoding")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("gzip"))
        .unwrap_or(false)
    {
        decompress_gzip(&body_bytes).unwrap_or(body_bytes)
    } else {
        body_bytes
    };

    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

    // Parse JSON body if content-type is application/json
    let content_type_value = headers.get("content-type").and_then(|v| v.to_str().ok());

    let json_body = if content_type_value.map(|v| v.contains("application/json")).unwrap_or(false) {
        serde_json::from_slice(&body_bytes).ok()
    } else {
        None
    };

    // OS info
    let os_info = Some(OsInfo { hostname: state.hostname.clone() });

    // Connection info
    let connection_info = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .map(|host| ConnectionInfo { servername: Some(host.to_string()) });

    // Environment variables
    let environment =
        if state.config.include_env_vars { Some(std::env::vars().collect()) } else { None };

    // JWT decoding
    #[cfg(feature = "jwt")]
    let jwt = extract_jwt(headers, &state.config);

    #[cfg(not(feature = "jwt"))]
    let jwt = None;

    // mTLS client cert info
    #[cfg(feature = "mtls")]
    let client_cert = extract_client_cert(headers);

    #[cfg(not(feature = "mtls"))]
    let client_cert = None;

    let echo_response = EchoResponse {
        path,
        headers: headers_map,
        method,
        body: body_str.clone(),
        cookies,
        fresh: false,
        hostname: state.hostname.clone(),
        ip,
        ips,
        protocol,
        query,
        subdomains,
        xhr,
        os: os_info,
        connection: connection_info,
        json: json_body,
        environment,
        #[cfg(feature = "jwt")]
        jwt,
        #[cfg(feature = "mtls")]
        client_cert,
    };

    // Check if echo back to client is disabled
    if let Some(false) = state.config.echo_back_to_client {
        let mut response = Response::new(Body::empty());
        *response.status_mut() = status_code;
        return response;
    }

    // Check if response_body_only is requested
    if query_params.response_body_only.unwrap_or(false) {
        let mut response = Response::new(Body::from(body_str));
        *response.status_mut() = status_code;

        if let Some(ct) = content_type {
            if let Ok(header_value) = HeaderValue::from_str(&ct) {
                response.headers_mut().insert("content-type", header_value);
            }
        }

        return response;
    }

    // Build JSON response
    let json_response = Json(echo_response);
    let mut response = json_response.into_response();
    *response.status_mut() = status_code;

    // Apply custom content-type if specified
    if let Some(ct) = content_type {
        if let Ok(header_value) = HeaderValue::from_str(&ct) {
            response.headers_mut().insert("content-type", header_value);
        }
    }

    // Apply CORS headers if configured
    if let Some(origin) = &state.config.cors_allow_origin {
        if let Ok(header_value) = HeaderValue::from_str(origin) {
            response.headers_mut().insert("access-control-allow-origin", header_value);
        }

        if let Some(methods) = &state.config.cors_allow_methods {
            if let Ok(header_value) = HeaderValue::from_str(methods) {
                response.headers_mut().insert("access-control-allow-methods", header_value);
            }
        }

        if let Some(headers_val) = &state.config.cors_allow_headers {
            if let Ok(header_value) = HeaderValue::from_str(headers_val) {
                response.headers_mut().insert("access-control-allow-headers", header_value);
            }
        }

        if let Some(credentials) = &state.config.cors_allow_credentials {
            if let Ok(header_value) = HeaderValue::from_str(credentials) {
                response.headers_mut().insert("access-control-allow-credentials", header_value);
            }
        }
    }

    response
}

fn extract_ips(headers: &HeaderMap, default_ip: &str) -> Vec<String> {
    let mut ips = vec![default_ip.to_string()];

    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            let forwarded_ips: Vec<String> =
                forwarded_str.split(',').map(|s| s.trim().to_string()).collect();
            ips.extend(forwarded_ips);
        }
    }

    ips
}

#[cfg(feature = "jwt")]
fn extract_jwt(headers: &HeaderMap, config: &Config) -> Option<JwtInfo> {
    let jwt_header = config.jwt_header.as_deref().unwrap_or("authorization");

    let token = headers
        .get(jwt_header)
        .and_then(|v| v.to_str().ok())
        .map(|v| if v.to_lowercase().starts_with("bearer ") { &v[7..] } else { v })?;

    let header = jsonwebtoken::decode_header(token).ok()?;

    // Decode without verification (just for inspection)
    let mut validation = jsonwebtoken::Validation::default();
    validation.insecure_disable_signature_validation();
    validation.validate_exp = false;

    let decoding_key = jsonwebtoken::DecodingKey::from_secret(&[]);

    // Try to decode payload as generic JSON
    let payload =
        if let Ok(token_data) = jsonwebtoken::decode::<Value>(token, &decoding_key, &validation) {
            Some(token_data.claims)
        } else {
            None
        };

    Some(JwtInfo { header: serde_json::to_value(header).ok(), payload })
}

#[cfg(feature = "mtls")]
fn extract_client_cert(headers: &HeaderMap) -> Option<ClientCertInfo> {
    // Check for common headers that contain client certificate info
    // These are typically set by reverse proxies (nginx, envoy, etc.)

    let subject = headers
        .get("x-client-cert-subject")
        .or_else(|| headers.get("ssl-client-subject-dn"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())?;

    let issuer = headers
        .get("x-client-cert-issuer")
        .or_else(|| headers.get("ssl-client-issuer-dn"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_default();

    let subjectaltname =
        headers.get("x-client-cert-san").and_then(|v| v.to_str().ok()).map(|s| s.to_string());

    let info =
        headers.get("x-client-cert-info").and_then(|v| v.to_str().ok()).map(|s| s.to_string());

    Some(ClientCertInfo { subject, issuer, subjectaltname, info })
}

#[cfg(feature = "prometheus")]
pub async fn metrics_handler() -> Response {
    use prometheus::{Encoder, TextEncoder};

    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];

    if encoder.encode(&metric_families, &mut buffer).is_ok() {
        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", encoder.format_type())
            .body(Body::from(buffer))
            .unwrap_or_else(|_| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .unwrap()
            })
    } else {
        Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty()).unwrap()
    }
}

fn extract_cookies(headers: &HeaderMap) -> Option<HashMap<String, String>> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;

    let mut cookies = HashMap::new();
    for cookie in cookie_header.split(';') {
        let cookie = cookie.trim();
        if let Some((key, value)) = cookie.split_once('=') {
            cookies.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    if cookies.is_empty() { None } else { Some(cookies) }
}

fn extract_subdomains(headers: &HeaderMap) -> Option<Vec<String>> {
    let host = headers.get("host")?.to_str().ok()?;

    // Remove port if present
    let host = host.split(':').next()?;

    // Split by dots and extract subdomains (excluding TLD and domain)
    let parts: Vec<&str> = host.split('.').collect();

    // Need at least 3 parts to have subdomains (e.g., sub.example.com)
    if parts.len() > 2 {
        let subdomains: Vec<String> =
            parts[..parts.len() - 2].iter().map(|s| s.to_string()).collect();

        if subdomains.is_empty() { None } else { Some(subdomains) }
    } else {
        None
    }
}

async fn serve_file(file_path: &std::path::Path) -> Response {
    use tokio::fs;

    match fs::read(file_path).await {
        Ok(contents) => {
            let mime_type = mime_guess::from_path(file_path).first_or_octet_stream().to_string();

            Response::builder()
                .status(StatusCode::OK)
                .header("content-type", mime_type)
                .body(Body::from(contents))
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Failed to build response"))
                        .unwrap()
                })
        },
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("Failed to read file: {}", e)))
            .unwrap(),
    }
}

fn decompress_gzip(data: &[u8]) -> Result<bytes::Bytes, std::io::Error> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(bytes::Bytes::from(decompressed))
}
