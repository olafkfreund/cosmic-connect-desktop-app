//! KDE Connect Transport Layer
//!
//! This module provides network transport for KDE Connect protocol.
//! Implements both basic TCP (for pairing) and TLS (for secure communication).

pub mod tcp;
pub mod tls;
pub mod tls_config;

pub use tcp::TcpConnection;
pub use tls::{TlsConnection, TlsServer};
