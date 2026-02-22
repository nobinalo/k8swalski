use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("TLS configuration error: {0}")]
    TlsConfig(String),

    #[error("Certificate loading error: {0}")]
    CertificateLoad(#[from] std::io::Error),

    #[error("Invalid certificate: {0}")]
    InvalidCertificate(String),

    #[error("Server error: {0}")]
    Server(String),

    #[cfg(feature = "jwt")]
    #[error("JWT decode error: {0}")]
    JwtDecode(#[from] jsonwebtoken::errors::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
