use super::events::DiscoveryEvent;
use crate::{DeviceInfo, Packet, ProtocolError, Result};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

pub const DISCOVERY_PORT: u16 = 1816;
pub const PORT_RANGE_START: u16 = 1814;
pub const PORT_RANGE_END: u16 = 1864;
pub const BROADCAST_ADDR: Ipv4Addr = Ipv4Addr::new(255, 255, 255, 255);
pub const DEFAULT_BROADCAST_INTERVAL: Duration = Duration::from_secs(5);
pub const DEFAULT_DEVICE_TIMEOUT: Duration = Duration::from_secs(30);

/// Additional broadcast addresses for cross-network discovery
/// Includes Waydroid subnet (192.168.240.255) by default
pub fn default_additional_broadcast_addrs() -> Vec<Ipv4Addr> {
    vec![
        Ipv4Addr::new(192, 168, 240, 255), // Waydroid default subnet
    ]
}

#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub broadcast_interval: Duration,
    pub device_timeout: Duration,
    pub enable_timeout_check: bool,
    /// Additional broadcast addresses for cross-network discovery (e.g., Waydroid, VMs)
    pub additional_broadcast_addrs: Vec<Ipv4Addr>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            broadcast_interval: DEFAULT_BROADCAST_INTERVAL,
            device_timeout: DEFAULT_DEVICE_TIMEOUT,
            enable_timeout_check: true,
            additional_broadcast_addrs: default_additional_broadcast_addrs(),
        }
    }
}

pub struct DiscoveryService {
    device_info: DeviceInfo,
    socket: Arc<UdpSocket>,
    event_tx: mpsc::UnboundedSender<DiscoveryEvent>,
    event_rx: Arc<RwLock<mpsc::UnboundedReceiver<DiscoveryEvent>>>,
    config: DiscoveryConfig,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    last_seen: Arc<RwLock<HashMap<String, u64>>>,
}

impl DiscoveryService {
    pub fn new(device_info: DeviceInfo, config: DiscoveryConfig) -> Result<Self> {
        let socket = Self::bind_socket()?;
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Ok(Self {
            device_info,
            socket: Arc::new(socket),
            event_tx,
            event_rx: Arc::new(RwLock::new(event_rx)),
            config,
            shutdown_tx: None,
            last_seen: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn with_defaults(device_info: DeviceInfo) -> Result<Self> {
        Self::new(device_info, DiscoveryConfig::default())
    }

    fn bind_socket() -> Result<UdpSocket> {
        match UdpSocket::bind(("0.0.0.0", DISCOVERY_PORT)) {
            Ok(socket) => {
                info!("Bound to UDP port {}", DISCOVERY_PORT);
                socket.set_broadcast(true)?;
                socket.set_nonblocking(true)?;
                Ok(socket)
            }
            Err(e) => {
                warn!(
                    "Failed to bind to primary port {}: {}. Trying fallback range...",
                    DISCOVERY_PORT, e
                );
                for port in PORT_RANGE_START..=PORT_RANGE_END {
                    if port == DISCOVERY_PORT {
                        continue;
                    }
                    if let Ok(socket) = UdpSocket::bind(("0.0.0.0", port)) {
                        info!("Bound to fallback UDP port {}", port);
                        socket.set_broadcast(true)?;
                        socket.set_nonblocking(true)?;
                        return Ok(socket);
                    }
                }
                Err(ProtocolError::Io(std::io::Error::new(
                    std::io::ErrorKind::AddrInUse,
                    "Failed to bind to any port",
                )))
            }
        }
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
        Ok(())
    }

    pub async fn start(&mut self) -> Result<()> {
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);
        self.spawn_broadcaster(shutdown_rx);
        self.spawn_listener();
        if self.config.enable_timeout_check {
            self.spawn_timeout_checker();
        }
        Ok(())
    }

    pub async fn subscribe(&self) -> mpsc::UnboundedReceiver<DiscoveryEvent> {
        let mut rx = self.event_rx.write().await;
        let (_tx, new_rx) = mpsc::unbounded_channel();
        let old_rx = std::mem::replace(&mut *rx, new_rx);
        drop(rx);
        old_rx
    }

    fn spawn_broadcaster(&self, mut shutdown_rx: tokio::sync::oneshot::Receiver<()>) {
        let socket = self.socket.clone();
        let device_info = self.device_info.clone();
        let interval_duration = self.config.broadcast_interval;
        let additional_addrs = self.config.additional_broadcast_addrs.clone();
        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            let packet = device_info.to_identity_packet();
            let bytes = match packet.to_bytes() {
                Ok(b) => b,
                Err(e) => {
                    error!("Failed to serialize identity packet: {}", e);
                    return;
                }
            };

            // Build list of all broadcast addresses
            let mut broadcast_addrs =
                vec![SocketAddr::new(IpAddr::V4(BROADCAST_ADDR), DISCOVERY_PORT)];
            for addr in &additional_addrs {
                broadcast_addrs.push(SocketAddr::new(IpAddr::V4(*addr), DISCOVERY_PORT));
            }

            // Also broadcast to KDE Connect port for compatibility
            let kdeconnect_port = 1716u16;
            broadcast_addrs.push(SocketAddr::new(IpAddr::V4(BROADCAST_ADDR), kdeconnect_port));
            for addr in &additional_addrs {
                broadcast_addrs.push(SocketAddr::new(IpAddr::V4(*addr), kdeconnect_port));
            }

            info!(
                "Discovery broadcaster configured with {} addresses",
                broadcast_addrs.len()
            );

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let mut success_count = 0;
                        for broadcast_addr in &broadcast_addrs {
                            if let Err(e) = socket.send_to(&bytes, broadcast_addr) {
                                // Don't warn for "network unreachable" - common for virtual subnets
                                if e.kind() != std::io::ErrorKind::NetworkUnreachable {
                                    debug!("Failed to send broadcast to {}: {}", broadcast_addr, e);
                                }
                            } else {
                                success_count += 1;
                            }
                        }
                        debug!(
                            "Broadcasted identity packet ({} bytes) to {}/{} addresses for device: {}",
                            bytes.len(),
                            success_count,
                            broadcast_addrs.len(),
                            device_info.device_name
                        );
                    }
                    _ = &mut shutdown_rx => {
                        debug!("Broadcaster shutting down");
                        break;
                    }
                }
            }
        });
    }

    fn spawn_listener(&self) {
        let socket = self.socket.clone();
        let event_tx = self.event_tx.clone();
        let own_device_id = self.device_info.device_id.clone();
        let own_device_info = self.device_info.clone();
        let last_seen = self.last_seen.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 8192];
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((size, src_addr)) => {
                        if let Err(e) = Self::handle_packet(
                            &buf[..size],
                            src_addr,
                            &own_device_id,
                            &own_device_info,
                            &socket,
                            &event_tx,
                            &last_seen,
                        )
                        .await
                        {
                            debug!("Error handling packet from {}: {}", src_addr, e);
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                    Err(e) => {
                        error!("Error receiving packet: {}", e);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        });
    }

    async fn handle_packet(
        data: &[u8],
        src_addr: SocketAddr,
        own_device_id: &str,
        _own_device_info: &DeviceInfo,
        _socket: &UdpSocket,
        event_tx: &mpsc::UnboundedSender<DiscoveryEvent>,
        last_seen: &Arc<RwLock<HashMap<String, u64>>>,
    ) -> Result<()> {
        let packet = Packet::from_bytes(data)?;
        if !packet.is_type("cconnect.identity") {
            return Ok(());
        }
        let device_info = DeviceInfo::from_identity_packet(&packet)?;
        if device_info.device_id == own_device_id {
            return Ok(());
        }
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut last_seen_map = last_seen.write().await;
        let is_new = !last_seen_map.contains_key(&device_info.device_id);
        last_seen_map.insert(device_info.device_id.clone(), current_time);
        drop(last_seen_map);
        let mut tcp_addr = src_addr;
        tcp_addr.set_port(device_info.tcp_port);
        let event = if is_new {
            info!(
                "Discovered new device: {} ({}) at {}",
                device_info.device_name,
                device_info.device_type.as_str(),
                tcp_addr
            );
            DiscoveryEvent::tcp_discovered(device_info, tcp_addr)
        } else {
            DiscoveryEvent::tcp_updated(device_info, tcp_addr)
        };
        let _ = event_tx.send(event);
        Ok(())
    }

    fn spawn_timeout_checker(&self) {
        let event_tx = self.event_tx.clone();
        let last_seen = self.last_seen.clone();
        let timeout_duration = self.config.device_timeout;
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let mut last_seen_map = last_seen.write().await;
                let mut timed_out = Vec::new();
                for (id, &last_time) in last_seen_map.iter() {
                    if current_time - last_time > timeout_duration.as_secs() {
                        timed_out.push(id.clone());
                    }
                }
                for id in timed_out {
                    last_seen_map.remove(&id);
                    let _ = event_tx.send(DiscoveryEvent::DeviceTimeout { device_id: id });
                }
            }
        });
    }

    pub fn local_port(&self) -> Result<u16> {
        Ok(self.socket.local_addr()?.port())
    }
}
