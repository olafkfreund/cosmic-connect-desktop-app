use thiserror::Error;

pub type Result<T> = std::result::Result<T, ProtocolError>;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),

    #[error("Certificate error: {0}")]
    Certificate(#[from] rcgen::Error),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Not paired")]
    NotPaired,

    #[error("Invalid packet: {0}")]
    InvalidPacket(String),

    #[error("Plugin error: {0}")]
    Plugin(String),
}
