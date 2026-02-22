use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    Json,
    body::Body,
    extract::{ConnectInfo, Query, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::sleep;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub hostname: String,
}

// Health check handlers
pub async fn liveness_handler() -> &'static str {
    "OK"
}

pub async fn readiness_handler() -> Json<Value> {
    Json(serde_json::json!({ "status": "ready" }))
}

#[derive(Debug, Deserialize)]
pub struct EchoQueryParams {
    #[serde(rename = "x-set-response-status-code")]
    pub status_code: Option<u16>,

    #[serde(rename = "x-set-response-delay-ms")]
    pub delay_ms: Option<u64>,

    #[serde(rename = "x-set-response-content-type")]
    pub content_type: Option<String>,

    #[serde(rename = "response_body_only")]
    pub body_only: Option<bool>,
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

#[allow(clippy::too_many_lines)]
pub async fn echo_handler(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(query_params): Query<EchoQueryParams>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let headers = &parts.headers;
    let method = parts.method.to_string();
    let uri = &parts.uri;
    let path = uri.path().to_string();
    let protocol = format!("{:?}", parts.version);

    // Override response with file content if configured
    if let Some(file_path) = &state.config.override_response_body_file_path {
        return serve_file(file_path).await;
    }

    // Status code: query param → header → default OK
    let status_code = query_params
        .status_code
        .or_else(|| {
            headers
                .get("x-set-response-status-code")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
        })
        .and_then(|code| StatusCode::from_u16(code).ok())
        .unwrap_or(StatusCode::OK);

    // Delay: query param → header
    let delay_ms = query_params.delay_ms.or_else(|| {
        headers
            .get("x-set-response-delay-ms")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
    });

    // Content-type override: query param → header
    let content_type_override = query_params.content_type.or_else(|| {
        headers.get("x-set-response-content-type").and_then(|v| v.to_str().ok()).map(str::to_string)
    });

    if let Some(delay) = delay_ms {
        sleep(Duration::from_millis(delay)).await;
    }

    // Parse query string into a map
    let query: HashMap<String, String> = uri
        .query()
        .map(|q| {
            url::form_urlencoded::parse(q.as_bytes())
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Collect headers into a map
    let mut headers_map: HashMap<String, String> = HashMap::new();
    for (key, value) in headers {
        if let Ok(v) = value.to_str() {
            headers_map.insert(key.to_string(), v.to_string());
        }
    }

    let ip = addr.ip().to_string();
    let ips = extract_ips(headers, &ip);

    let xhr = headers
        .get("x-requested-with")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.eq_ignore_ascii_case("XMLHttpRequest"));

    let cookies = extract_cookies(headers);
    let subdomains = extract_subdomains(headers);

    let body_bytes =
        axum::body::to_bytes(body, state.config.max_body_size).await.unwrap_or_default();

    // Decompress gzip body if indicated
    let body_bytes = if headers
        .get("content-encoding")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.eq_ignore_ascii_case("gzip"))
    {
        decompress_gzip(&body_bytes).unwrap_or(body_bytes)
    } else {
        body_bytes
    };

    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

    // Parse JSON body if content-type is application/json
    let request_content_type = headers.get("content-type").and_then(|v| v.to_str().ok());
    let json_body = if request_content_type.is_some_and(|v| v.contains("application/json")) {
        serde_json::from_slice(&body_bytes).ok()
    } else {
        None
    };

    let os_info = Some(OsInfo { hostname: state.hostname.clone() });
    let connection_info = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .map(|host| ConnectionInfo { servername: Some(host.to_string()) });

    let environment =
        if state.config.include_env_vars { Some(std::env::vars().collect()) } else { None };

    #[cfg(feature = "jwt")]
    let jwt = extract_jwt(headers, &state.config);
    #[cfg(not(feature = "jwt"))]
    let jwt = None;

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

    // Suppress echo if disabled
    if state.config.echo_back_to_client == Some(false) {
        let mut response = Response::new(Body::empty());
        *response.status_mut() = status_code;
        return response;
    }

    // Body-only mode: return raw request body
    if query_params.body_only.unwrap_or_default() {
        let mut response = Response::new(Body::from(body_str));
        *response.status_mut() = status_code;
        if let Some(ct) = content_type_override {
            if let Ok(header_value) = HeaderValue::from_str(&ct) {
                response.headers_mut().insert("content-type", header_value);
            }
        }
        return response;
    }

    // Default: JSON echo response
    let mut response = Json(echo_response).into_response();
    *response.status_mut() = status_code;

    if let Some(ct) = content_type_override {
        if let Ok(header_value) = HeaderValue::from_str(&ct) {
            response.headers_mut().insert("content-type", header_value);
        }
    }

    // Manual CORS headers (when cors_allow_origin is set directly on config)
    if let Some(origin) = &state.config.cors_allow_origin {
        if let Ok(v) = HeaderValue::from_str(origin) {
            response.headers_mut().insert("access-control-allow-origin", v);
        }
        if let Some(methods) = &state.config.cors_allow_methods {
            if let Ok(v) = HeaderValue::from_str(methods) {
                response.headers_mut().insert("access-control-allow-methods", v);
            }
        }
        if let Some(hdrs) = &state.config.cors_allow_headers {
            if let Ok(v) = HeaderValue::from_str(hdrs) {
                response.headers_mut().insert("access-control-allow-headers", v);
            }
        }
        if let Some(creds) = &state.config.cors_allow_credentials {
            if let Ok(v) = HeaderValue::from_str(creds) {
                response.headers_mut().insert("access-control-allow-credentials", v);
            }
        }
    }

    response
}

fn extract_ips(headers: &HeaderMap, default_ip: &str) -> Vec<String> {
    let mut ips = vec![default_ip.to_string()];
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            ips.extend(forwarded_str.split(',').map(|s| s.trim().to_string()));
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
    let payload = jsonwebtoken::dangerous::insecure_decode::<Value>(token).ok().map(|td| td.claims);

    Some(JwtInfo { header: serde_json::to_value(header).ok(), payload })
}

#[cfg(feature = "mtls")]
fn extract_client_cert(headers: &HeaderMap) -> Option<ClientCertInfo> {
    let subject = headers
        .get("x-client-cert-subject")
        .or_else(|| headers.get("ssl-client-subject-dn"))
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)?;

    let issuer = headers
        .get("x-client-cert-issuer")
        .or_else(|| headers.get("ssl-client-issuer-dn"))
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .unwrap_or_default();

    let subjectaltname =
        headers.get("x-client-cert-san").and_then(|v| v.to_str().ok()).map(str::to_string);
    let info = headers.get("x-client-cert-info").and_then(|v| v.to_str().ok()).map(str::to_string);

    Some(ClientCertInfo { subject, issuer, subjectaltname, info })
}

#[cfg(feature = "prometheus")]
pub async fn metrics_handler() -> Response {
    use prometheus::{Encoder, TextEncoder};

    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];

    if encoder.encode(&metric_families, &mut buffer).is_ok() {
        let mut response = Response::new(Body::from(buffer));
        if let Ok(ct) = HeaderValue::from_str(encoder.format_type()) {
            response.headers_mut().insert("content-type", ct);
        }
        response
    } else {
        let mut response = Response::new(Body::empty());
        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        response
    }
}

fn extract_cookies(headers: &HeaderMap) -> Option<HashMap<String, String>> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    let mut cookies = HashMap::new();
    for raw in cookie_header.split(';') {
        let trimmed = raw.trim();
        if let Some((key, value)) = trimmed.split_once('=') {
            cookies.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    if cookies.is_empty() { None } else { Some(cookies) }
}

fn extract_subdomains(headers: &HeaderMap) -> Option<Vec<String>> {
    let host_header = headers.get("host")?.to_str().ok()?;
    let host = host_header.split(':').next()?;

    if host.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }

    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() > 2 {
        let subdomains: Vec<String> =
            parts[..parts.len() - 2].iter().copied().map(str::to_string).collect();
        if subdomains.is_empty() { None } else { Some(subdomains) }
    } else {
        None
    }
}

async fn serve_file(file_path: &std::path::Path) -> Response {
    match tokio::fs::read(file_path).await {
        Ok(contents) => {
            let mime = mime_guess::from_path(file_path).first_or_octet_stream().to_string();
            let mut response = Response::new(Body::from(contents));
            if let Ok(ct) = HeaderValue::from_str(&mime) {
                response.headers_mut().insert("content-type", ct);
            }
            response
        },
        Err(e) => {
            let mut response = Response::new(Body::from(format!("Failed to read file: {e}")));
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            response
        },
    }
}

fn decompress_gzip(data: &[u8]) -> Result<bytes::Bytes, std::io::Error> {
    use std::io::Read;

    use flate2::read::GzDecoder;

    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(bytes::Bytes::from(decompressed))
}
