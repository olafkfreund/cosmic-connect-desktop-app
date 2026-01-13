//! TLS Transport for KDE Connect
//!
//! Provides encrypted TCP connections using TLS with mutual certificate authentication.
//! Used for secure communication between paired devices.

use crate::{CertificateInfo, Packet, ProtocolError, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{timeout, Duration};
use tokio_rustls::{TlsAcceptor, TlsConnector, TlsStream};
use tracing::{debug, error, info, warn};

use super::tls_config;

/// Default timeout for TLS operations
const TLS_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum packet size (10MB - larger than TCP to support file transfers)
const MAX_PACKET_SIZE: usize = 10 * 1024 * 1024;

/// TLS connection to a remote device
pub struct TlsConnection {
    /// TLS stream
    stream: TlsStream<TcpStream>,
    /// Remote address
    remote_addr: SocketAddr,
    /// Device ID of remote peer (if known)
    device_id: Option<String>,
}

impl TlsConnection {
    /// Connect to a remote device using TLS
    ///
    /// # Arguments
    ///
    /// * `addr` - Remote socket address
    /// * `our_cert` - Our device certificate
    /// * `peer_cert` - Expected peer certificate (from pairing)
    /// * `server_name` - SNI server name (usually IP address)
    pub async fn connect(
        addr: SocketAddr,
        our_cert: &CertificateInfo,
        peer_cert: Vec<u8>,
        server_name: &str,
    ) -> Result<Self> {
        info!("Connecting to {} via TLS", addr);

        // Create TLS client config
        let config = tls_config::create_client_config(our_cert, peer_cert)?;
        let connector = TlsConnector::from(Arc::new(config));

        // Connect TCP stream
        let tcp_stream = timeout(TLS_TIMEOUT, TcpStream::connect(addr))
            .await
            .map_err(|_| {
                ProtocolError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Connection timeout",
                ))
            })??;

        debug!("TCP connection established to {}", addr);

        // Perform TLS handshake
        let server_name = rustls::pki_types::ServerName::try_from(server_name.to_string())
            .map_err(|_| ProtocolError::CertificateValidation("Invalid server name".to_string()))?;

        let tls_stream = timeout(TLS_TIMEOUT, connector.connect(server_name, tcp_stream))
            .await
            .map_err(|_| {
                ProtocolError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "TLS handshake timeout",
                ))
            })?
            .map_err(|e| {
                error!("TLS handshake failed: {}", e);
                ProtocolError::Io(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    format!("TLS handshake failed: {}", e),
                ))
            })?;

        info!("TLS connection established to {}", addr);

        Ok(Self {
            stream: tokio_rustls::TlsStream::Client(tls_stream),
            remote_addr: addr,
            device_id: None,
        })
    }

    /// Create from an accepted TLS stream
    pub fn from_stream(stream: TlsStream<TcpStream>, remote_addr: SocketAddr) -> Self {
        Self {
            stream,
            remote_addr,
            device_id: None,
        }
    }

    /// Set the device ID for this connection
    pub fn set_device_id(&mut self, device_id: String) {
        self.device_id = Some(device_id);
    }

    /// Get the device ID if known
    pub fn device_id(&self) -> Option<&str> {
        self.device_id.as_deref()
    }

    /// Get remote address
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    /// Send a packet over the TLS connection
    pub async fn send_packet(&mut self, packet: &Packet) -> Result<()> {
        let bytes = packet.to_bytes()?;

        if bytes.len() > MAX_PACKET_SIZE {
            return Err(ProtocolError::InvalidPacket(format!(
                "Packet too large: {} bytes (max {})",
                bytes.len(),
                MAX_PACKET_SIZE
            )));
        }

        debug!(
            "Sending packet '{}' ({} bytes) to {}",
            packet.packet_type,
            bytes.len(),
            self.remote_addr
        );

        // Send packet length as 4-byte big-endian
        let len = bytes.len() as u32;
        self.stream.write_all(&len.to_be_bytes()).await?;

        // Send packet data
        self.stream.write_all(&bytes).await?;
        self.stream.flush().await?;

        debug!("Packet sent successfully to {}", self.remote_addr);
        Ok(())
    }

    /// Receive a packet from the TLS connection
    pub async fn receive_packet(&mut self) -> Result<Packet> {
        debug!("Waiting for packet from {}", self.remote_addr);

        // Read packet length (4 bytes, big-endian)
        let mut len_bytes = [0u8; 4];
        timeout(TLS_TIMEOUT, self.stream.read_exact(&mut len_bytes))
            .await
            .map_err(|_| {
                ProtocolError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Read timeout",
                ))
            })??;

        let len = u32::from_be_bytes(len_bytes) as usize;

        if len > MAX_PACKET_SIZE {
            error!("Packet too large: {} bytes", len);
            return Err(ProtocolError::InvalidPacket(format!(
                "Packet too large: {} bytes (max {})",
                len, MAX_PACKET_SIZE
            )));
        }

        debug!("Receiving packet ({} bytes) from {}", len, self.remote_addr);

        // Read packet data
        let mut data = vec![0u8; len];
        timeout(TLS_TIMEOUT, self.stream.read_exact(&mut data))
            .await
            .map_err(|_| {
                ProtocolError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Read timeout",
                ))
            })??;

        let packet = Packet::from_bytes(&data)?;
        debug!(
            "Received packet type '{}' from {}",
            packet.packet_type, self.remote_addr
        );

        Ok(packet)
    }

    /// Close the TLS connection
    pub async fn close(mut self) -> Result<()> {
        debug!("Closing TLS connection to {}", self.remote_addr);
        self.stream.shutdown().await?;
        Ok(())
    }
}

/// TLS server for accepting connections from paired devices
pub struct TlsServer {
    /// TCP listener
    listener: TcpListener,
    /// TLS acceptor
    acceptor: TlsAcceptor,
    /// Local address
    local_addr: SocketAddr,
}

impl TlsServer {
    /// Create a new TLS server
    ///
    /// # Arguments
    ///
    /// * `addr` - Local address to bind to
    /// * `our_cert` - Our device certificate
    /// * `trusted_device_certs` - Certificates of all paired devices
    pub async fn new(
        addr: SocketAddr,
        our_cert: &CertificateInfo,
        trusted_device_certs: Vec<Vec<u8>>,
    ) -> Result<Self> {
        info!("Starting TLS server on {}", addr);

        // Create TLS server config
        let config = tls_config::create_server_config(our_cert, trusted_device_certs)?;
        let acceptor = TlsAcceptor::from(Arc::new(config));

        // Bind TCP listener
        let listener = TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;

        info!("TLS server listening on {}", local_addr);

        Ok(Self {
            listener,
            acceptor,
            local_addr,
        })
    }

    /// Get the local address
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Accept an incoming TLS connection
    ///
    /// This will wait for a client to connect and complete the TLS handshake.
    /// Only accepts connections from paired devices (certificate validation).
    pub async fn accept(&self) -> Result<TlsConnection> {
        debug!("Waiting for incoming TLS connection");

        // Accept TCP connection
        let (tcp_stream, remote_addr) = self.listener.accept().await?;

        debug!("TCP connection accepted from {}", remote_addr);

        // Perform TLS handshake
        let tls_stream = timeout(TLS_TIMEOUT, self.acceptor.accept(tcp_stream))
            .await
            .map_err(|_| {
                warn!("TLS handshake timeout from {}", remote_addr);
                ProtocolError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "TLS handshake timeout",
                ))
            })?
            .map_err(|e| {
                warn!("TLS handshake failed from {}: {}", remote_addr, e);
                ProtocolError::Io(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    format!("TLS handshake failed: {}", e),
                ))
            })?;

        info!("TLS connection accepted from {}", remote_addr);

        Ok(TlsConnection::from_stream(
            tokio_rustls::TlsStream::Server(tls_stream),
            remote_addr,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_tls_connection_send_receive() {
        // Generate certificates for two devices
        let device1_cert = CertificateInfo::generate("device1").unwrap();
        let device2_cert = CertificateInfo::generate("device2").unwrap();

        // Start TLS server (device2)
        let server_addr = "127.0.0.1:0".parse().unwrap();
        let server = TlsServer::new(
            server_addr,
            &device2_cert,
            vec![device1_cert.certificate.clone()],
        )
        .await
        .unwrap();

        let server_port = server.local_addr().port();
        let server_addr = format!("127.0.0.1:{}", server_port).parse().unwrap();

        // Spawn server task
        let server_task = tokio::spawn(async move {
            // Accept connection
            let mut conn = server.accept().await.unwrap();

            // Receive packet
            let packet = conn.receive_packet().await.unwrap();
            assert_eq!(packet.packet_type, "test.packet");

            // Send response
            let response = Packet::new("test.response", json!({"status": "ok"}));
            conn.send_packet(&response).await.unwrap();

            conn.close().await.unwrap();
        });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Connect as client (device1)
        let mut client = TlsConnection::connect(
            server_addr,
            &device1_cert,
            device2_cert.certificate.clone(),
            "127.0.0.1",
        )
        .await
        .unwrap();

        // Send packet
        let test_packet = Packet::new("test.packet", json!({"data": "hello"}));
        client.send_packet(&test_packet).await.unwrap();

        // Receive response
        let response = client.receive_packet().await.unwrap();
        assert_eq!(response.packet_type, "test.response");

        client.close().await.unwrap();
        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn test_tls_connection_certificate_mismatch() {
        // Generate certificates
        let device1_cert = CertificateInfo::generate("device1").unwrap();
        let device2_cert = CertificateInfo::generate("device2").unwrap();
        let device3_cert = CertificateInfo::generate("device3").unwrap();

        // Start server that only trusts device3
        let server_addr = "127.0.0.1:0".parse().unwrap();
        let server = TlsServer::new(
            server_addr,
            &device2_cert,
            vec![device3_cert.certificate.clone()],
        )
        .await
        .unwrap();

        let server_port = server.local_addr().port();
        let server_addr = format!("127.0.0.1:{}", server_port).parse().unwrap();

        // Spawn server task (will reject device1)
        let _server_task = tokio::spawn(async move {
            let _ = server.accept().await; // Will fail
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        // Try to connect as device1 (should be rejected)
        let result = TlsConnection::connect(
            server_addr,
            &device1_cert,
            device2_cert.certificate.clone(),
            "127.0.0.1",
        )
        .await;

        // Connection should fail due to certificate mismatch
        assert!(result.is_err());
    }
}
