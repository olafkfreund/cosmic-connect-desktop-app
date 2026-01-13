//! Connection Manager
//!
//! Manages TLS connections to multiple devices, handles connection lifecycle,
//! and routes packets between devices and the application.

use super::events::ConnectionEvent;
use crate::transport::{TlsConnection, TlsServer};
use crate::{CertificateInfo, DeviceManager, Packet, ProtocolError, Result};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

/// Keep-alive interval (send ping every 30 seconds)
const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(30);

/// Connection timeout (consider disconnected after 60 seconds of no activity)
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(60);

/// Commands that can be sent to a connection task
enum ConnectionCommand {
    /// Send a packet
    SendPacket(Packet),
    /// Close the connection
    Close,
}

/// Active connection to a device
struct ActiveConnection {
    /// Channel to send commands to the connection task
    command_tx: mpsc::UnboundedSender<ConnectionCommand>,
    /// Task handling this connection
    task: JoinHandle<()>,
    /// Device ID
    device_id: String,
    /// Remote address
    remote_addr: SocketAddr,
}

/// Connection manager configuration
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Local address to bind TLS server to
    pub listen_addr: SocketAddr,
    /// Keep-alive interval
    pub keep_alive_interval: Duration,
    /// Connection timeout
    pub connection_timeout: Duration,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:1716".parse().unwrap(),
            keep_alive_interval: KEEP_ALIVE_INTERVAL,
            connection_timeout: CONNECTION_TIMEOUT,
        }
    }
}

/// Connection manager for handling multiple TLS connections
pub struct ConnectionManager {
    /// Our device certificate
    certificate: Arc<CertificateInfo>,

    /// Active connections (device_id -> connection)
    connections: Arc<RwLock<HashMap<String, ActiveConnection>>>,

    /// Device manager (for getting paired device certificates)
    device_manager: Arc<RwLock<DeviceManager>>,

    /// Event channel sender
    event_tx: mpsc::UnboundedSender<ConnectionEvent>,

    /// Event channel receiver
    event_rx: Arc<RwLock<mpsc::UnboundedReceiver<ConnectionEvent>>>,

    /// Configuration
    config: ConnectionConfig,

    /// TLS server task handle
    server_task: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(
        certificate: CertificateInfo,
        device_manager: Arc<RwLock<DeviceManager>>,
        config: ConnectionConfig,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Self {
            certificate: Arc::new(certificate),
            connections: Arc::new(RwLock::new(HashMap::new())),
            device_manager,
            event_tx,
            event_rx: Arc::new(RwLock::new(event_rx)),
            config,
            server_task: Arc::new(RwLock::new(None)),
        }
    }

    /// Get a receiver for connection events
    pub async fn subscribe(&self) -> mpsc::UnboundedReceiver<ConnectionEvent> {
        let (tx, rx) = mpsc::unbounded_channel();

        // Forward events
        let event_rx = self.event_rx.clone();
        tokio::spawn(async move {
            let mut rx_lock = event_rx.write().await;
            while let Some(event) = rx_lock.recv().await {
                if tx.send(event).is_err() {
                    break;
                }
            }
        });

        rx
    }

    /// Start the connection manager and TLS server
    pub async fn start(&self) -> Result<u16> {
        info!("Starting connection manager on {}", self.config.listen_addr);

        // Get all paired device certificates
        let device_manager = self.device_manager.read().await;
        let paired_devices = device_manager.paired_devices().collect::<Vec<_>>();

        let mut trusted_certs = Vec::new();
        for device in paired_devices {
            if let Some(cert_data) = &device.certificate_data {
                trusted_certs.push(cert_data.clone());
            }
        }
        drop(device_manager);

        info!(
            "Starting TLS server with {} trusted device certificates",
            trusted_certs.len()
        );

        // Create TLS server
        let server = TlsServer::new(self.config.listen_addr, &self.certificate, trusted_certs).await?;
        let local_port = server.local_addr().port();

        // Emit started event
        let _ = self.event_tx.send(ConnectionEvent::ManagerStarted {
            port: local_port,
        });

        // Spawn server accept task
        let connections = self.connections.clone();
        let event_tx = self.event_tx.clone();
        let device_manager = self.device_manager.clone();

        let server_task = tokio::spawn(async move {
            loop {
                match server.accept().await {
                    Ok(connection) => {
                        let remote_addr = connection.remote_addr();
                        info!("Accepted TLS connection from {}", remote_addr);

                        // Spawn connection handler
                        Self::spawn_connection_handler(
                            connection,
                            remote_addr,
                            event_tx.clone(),
                            connections.clone(),
                            device_manager.clone(),
                        );
                    }
                    Err(e) => {
                        error!("Error accepting TLS connection: {}", e);
                    }
                }
            }
        });

        let mut server_task_lock = self.server_task.write().await;
        *server_task_lock = Some(server_task);
        drop(server_task_lock);

        info!("Connection manager started on port {}", local_port);

        Ok(local_port)
    }

    /// Connect to a remote device
    pub async fn connect(&self, device_id: &str, addr: SocketAddr) -> Result<()> {
        info!("Connecting to device {} at {}", device_id, addr);

        // Check if already connected
        let connections = self.connections.read().await;
        if connections.contains_key(device_id) {
            info!("Already connected to device {}", device_id);
            return Ok(());
        }
        drop(connections);

        // Get device certificate from device manager
        let device_manager = self.device_manager.read().await;
        let device = device_manager
            .get_device(device_id)
            .ok_or_else(|| ProtocolError::DeviceNotFound(device_id.to_string()))?;

        let peer_cert = device
            .certificate_data
            .clone()
            .ok_or_else(|| ProtocolError::CertificateValidation("Device has no certificate".to_string()))?;

        drop(device_manager);

        // Connect with TLS
        let mut connection = TlsConnection::connect(
            addr,
            &self.certificate,
            peer_cert,
            &addr.ip().to_string(),
        )
        .await?;

        connection.set_device_id(device_id.to_string());

        // Spawn connection handler
        Self::spawn_connection_handler(
            connection,
            addr,
            self.event_tx.clone(),
            self.connections.clone(),
            self.device_manager.clone(),
        );

        info!("Connected to device {} at {}", device_id, addr);

        Ok(())
    }

    /// Send a packet to a device
    pub async fn send_packet(&self, device_id: &str, packet: &Packet) -> Result<()> {
        debug!("Sending packet '{}' to device {}", packet.packet_type, device_id);

        let connections = self.connections.read().await;
        let connection = connections
            .get(device_id)
            .ok_or_else(|| ProtocolError::DeviceNotFound(format!("Not connected to device {}", device_id)))?;

        connection
            .command_tx
            .send(ConnectionCommand::SendPacket(packet.clone()))
            .map_err(|_| ProtocolError::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Connection closed",
            )))?;

        debug!("Packet queued for device {}", device_id);
        Ok(())
    }

    /// Disconnect from a device
    pub async fn disconnect(&self, device_id: &str) -> Result<()> {
        info!("Disconnecting from device {}", device_id);

        let mut connections = self.connections.write().await;
        if let Some(active_conn) = connections.remove(device_id) {
            // Send close command
            let _ = active_conn.command_tx.send(ConnectionCommand::Close);

            // Abort task
            active_conn.task.abort();

            info!("Disconnected from device {}", device_id);
        }

        Ok(())
    }

    /// Stop the connection manager
    pub async fn stop(&self) {
        info!("Stopping connection manager");

        // Stop server task
        let mut server_task = self.server_task.write().await;
        if let Some(task) = server_task.take() {
            task.abort();
        }
        drop(server_task);

        // Disconnect all devices
        let device_ids: Vec<String> = {
            let connections = self.connections.read().await;
            connections.keys().cloned().collect()
        };

        for device_id in device_ids {
            let _ = self.disconnect(&device_id).await;
        }

        // Emit stopped event
        let _ = self.event_tx.send(ConnectionEvent::ManagerStopped);

        info!("Connection manager stopped");
    }

    /// Spawn a task to handle a connection (send/receive)
    fn spawn_connection_handler(
        mut connection: TlsConnection,
        remote_addr: SocketAddr,
        event_tx: mpsc::UnboundedSender<ConnectionEvent>,
        connections: Arc<RwLock<HashMap<String, ActiveConnection>>>,
        device_manager: Arc<RwLock<DeviceManager>>,
    ) {
        let (command_tx, mut command_rx) = mpsc::unbounded_channel();

        let _task = tokio::spawn(async move {
            let device_id: Option<String>;

            // First, try to receive identity packet
            match connection.receive_packet().await {
                Ok(packet) => {
                    // Extract device ID from packet
                    if let Some(id) = packet.body.get("deviceId").and_then(|v| v.as_str()) {
                        device_id = Some(id.to_string());
                        connection.set_device_id(id.to_string());

                        info!("Connection identified as device {}", id);

                        // Update device manager
                        let mut dm = device_manager.write().await;
                        if let Err(e) = dm.mark_connected(
                            id,
                            remote_addr.ip().to_string(),
                            remote_addr.port(),
                        ) {
                            warn!("Failed to mark device {} as connected: {}", id, e);
                        }
                        drop(dm);

                        // Emit connected event
                        let _ = event_tx.send(ConnectionEvent::Connected {
                            device_id: id.to_string(),
                            remote_addr,
                        });

                        // Emit packet received event
                        let _ = event_tx.send(ConnectionEvent::PacketReceived {
                            device_id: id.to_string(),
                            packet: packet.clone(),
                        });

                        // Store connection in active connections
                        let mut conns = connections.write().await;
                        conns.insert(
                            id.to_string(),
                            ActiveConnection {
                                command_tx: command_tx.clone(),
                                task: tokio::task::spawn(async {}), // Placeholder
                                device_id: id.to_string(),
                                remote_addr,
                            },
                        );
                        drop(conns);
                    } else {
                        warn!("First packet from {} did not contain deviceId", remote_addr);
                        return;
                    }
                }
                Err(e) => {
                    error!("Failed to receive identity packet from {}: {}", remote_addr, e);
                    return;
                }
            }

            let device_id = device_id.unwrap();

            // Main connection loop
            loop {
                tokio::select! {
                    // Handle commands
                    Some(cmd) = command_rx.recv() => {
                        match cmd {
                            ConnectionCommand::SendPacket(packet) => {
                                if let Err(e) = connection.send_packet(&packet).await {
                                    error!("Failed to send packet to {}: {}", device_id, e);
                                    break;
                                }
                            }
                            ConnectionCommand::Close => {
                                info!("Closing connection to {}", device_id);
                                break;
                            }
                        }
                    }

                    // Receive packets
                    result = connection.receive_packet() => {
                        match result {
                            Ok(packet) => {
                                debug!("Received packet '{}' from {}", packet.packet_type, device_id);
                                let _ = event_tx.send(ConnectionEvent::PacketReceived {
                                    device_id: device_id.clone(),
                                    packet,
                                });
                            }
                            Err(e) => {
                                warn!("Error receiving packet from {}: {}", device_id, e);
                                break;
                            }
                        }
                    }
                }
            }

            // Clean up
            info!("Connection handler for {} stopping", device_id);

            // Remove from active connections
            let mut conns = connections.write().await;
            conns.remove(&device_id);
            drop(conns);

            // Update device manager
            let mut dm = device_manager.write().await;
            let _ = dm.mark_disconnected(&device_id);
            drop(dm);

            // Emit disconnected event
            let _ = event_tx.send(ConnectionEvent::Disconnected {
                device_id: device_id.clone(),
                reason: Some("Connection closed".to_string()),
            });

            // Close connection
            let _ = connection.close().await;

            info!("Connection handler for {} stopped", device_id);
        });

        // Note: We can't update the task handle in ActiveConnection here
        // because we just moved it into the spawn. This is a limitation
        // of the current design. We could use Arc<Mutex<JoinHandle>> but
        // it's not necessary since we can abort via the command channel.
    }
}
