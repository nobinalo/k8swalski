use anyhow::{Context, Result};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use std::{net::SocketAddr, path::Path, sync::Arc};
use tokio::signal;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use k8swalski::{
    build_router,
    config::{Config, LogFormat},
    handlers::AppState,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse configuration
    let config = Config::parse();

    // Initialize logging
    init_logging(&config.log_format);

    info!("Starting k8swalski echo server");
    info!("HTTP port: {}", config.http_port);
    info!("HTTPS port: {}", config.https_port);

    // Get hostname
    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string());

    info!("Hostname: {}", hostname);

    // Create application state
    let state = AppState { config: Arc::new(config.clone()), hostname };

    // Spawn HTTP server
    let http_handle = {
        let state = state.clone();
        let port = config.http_port;
        tokio::spawn(async move { run_http_server(port, state).await })
    };

    // Generate certificates if they don't exist
    generate_certs_if_missing(&config.tls_cert_path, &config.tls_key_path).await?;

    // Spawn HTTPS server
    let https_handle = {
        let port = config.https_port;
        let cert_path = config.tls_cert_path.clone();
        let key_path = config.tls_key_path.clone();
        tokio::spawn(async move { run_https_server(port, &cert_path, &key_path, state).await })
    };

    // Wait for servers to complete (they handle shutdown internally)
    let (http_result, https_result) = tokio::join!(http_handle, https_handle);

    if let Err(e) = http_result {
        warn!("HTTP server task error: {}", e);
    }
    if let Err(e) = https_result {
        warn!("HTTPS server task error: {}", e);
    }

    info!("Servers stopped");
    Ok(())
}

async fn generate_certs_if_missing(cert_path: &Path, key_path: &Path) -> Result<()> {
    // Check if certificates already exist
    if cert_path.exists() && key_path.exists() {
        info!("Using existing TLS certificates");
        return Ok(());
    }

    info!("Generating self-signed TLS certificates...");

    // Generate certificate with rcgen
    let mut params = rcgen::CertificateParams::new(vec!["localhost".to_string()])
        .context("Failed to create certificate params")?;
    params.distinguished_name = rcgen::DistinguishedName::new();
    params.distinguished_name.push(rcgen::DnType::CommonName, "localhost");

    let key_pair = rcgen::KeyPair::generate()?;
    let cert =
        params.self_signed(&key_pair).context("Failed to generate self-signed certificate")?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    // Write certificate and key
    tokio::fs::write(cert_path, cert_pem.as_bytes())
        .await
        .context("Failed to write certificate file")?;
    tokio::fs::write(key_path, key_pem.as_bytes()).await.context("Failed to write key file")?;

    info!("Generated self-signed TLS certificates at {:?} and {:?}", cert_path, key_path);
    Ok(())
}

fn init_logging(format: &LogFormat) {
    match format {
        LogFormat::Json => {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "k8swalski=info,tower_http=info".into()),
                )
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        },
        LogFormat::Human => {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "k8swalski=info,tower_http=info".into()),
                )
                .with(tracing_subscriber::fmt::layer().pretty())
                .init();
        },
    }
}

async fn run_http_server(port: u16, state: AppState) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("HTTP server listening on {}", addr);

    let app = build_router(state);
    let listener =
        tokio::net::TcpListener::bind(addr).await.context("Failed to bind HTTP listener")?;

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("HTTP server error")?;

    info!("HTTP server stopped");
    Ok(())
}

async fn run_https_server(
    port: u16,
    cert_path: &Path,
    key_path: &Path,
    state: AppState,
) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("HTTPS server listening on {}", addr);

    let app = build_router(state);
    let tls_config = RustlsConfig::from_pem_file(cert_path, key_path)
        .await
        .context("Failed to load TLS configuration")?;

    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .context("HTTPS server error")?;

    info!("HTTPS server stopped");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
