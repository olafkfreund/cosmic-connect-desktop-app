//! Network Share Plugin (SFTP)
//!
//! Allows mounting remote device filesystems via SFTP/SSHFS.
//!
//! ## Protocol
//!
//! **Packet Types**:
//! - `kdeconnect.sftp` - SFTP connection details (incoming)
//! - `cconnect.sftp` - COSMIC Connect SFTP details (incoming)
//!
//! **Capabilities**:
//! - Incoming: `kdeconnect.sftp`, `cconnect.sftp` - Receive SFTP connection info
//!
//! ## Packet Format
//!
//! ```json
//! {
//!     "id": 1234567890,
//!     "type": "kdeconnect.sftp",
//!     "body": {
//!         "ip": "192.168.1.10",
//!         "port": 1739,
//!         "user": "kdeconnect",
//!         "password": "generated_password",
//!         "path": "/storage/emulated/0"
//!     }
//! }
//! ```
//!
//! ## Behavior
//!
//! When this packet is received, the desktop client should mount the remote filesystem
//! using sshfs.
//!
//! `sshfs -p <port> <user>@<ip>:/ <mountpoint> -o password_stdin`
//!
//! ## Public API
//!
//! ```rust,ignore
//! use cosmic_ext_connect_core::plugins::networkshare::NetworkSharePlugin;
//!
//! // Get all active shares
//! let shares = plugin.get_shares().await;
//!
//! // Get share for a specific device
//! if let Some(info) = plugin.get_share("device-id").await {
//!     println!("SFTP: {}@{}:{}", info.user, info.ip, info.port.unwrap_or(22));
//! }
//!
//! // Check if any shares are available
//! if plugin.has_shares().await {
//!     println!("SFTP shares available");
//! }
//! ```
//!
//! ## References
//!
//! - [KDE Connect SFTP Plugin](https://invent.kde.org/network/kdeconnect-kde/-/tree/master/plugins/sftp)

use crate::{Device, Packet, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::{Plugin, PluginFactory};

/// Packet type for SFTP connection info
pub const PACKET_TYPE_SFTP: &str = "kdeconnect.sftp";
pub const PACKET_TYPE_CCONNECT_SFTP: &str = "cconnect.sftp";

/// SFTP connection details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SftpInfo {
    /// IP address of the SFTP server
    pub ip: String,
    /// Port number (optional, defaults to 22)
    pub port: Option<u16>,
    /// Username
    pub user: String,
    /// Password
    pub password: String,
    /// Path to mount (optional)
    pub path: Option<String>,
    /// Timestamp when this info was received
    #[serde(skip)]
    pub received_at: Option<std::time::Instant>,
}

impl SftpInfo {
    /// Get the effective port (defaults to 22 if not specified)
    pub fn effective_port(&self) -> u16 {
        self.port.unwrap_or(22)
    }

    /// Get the effective path (defaults to "/" if not specified)
    pub fn effective_path(&self) -> &str {
        self.path.as_deref().unwrap_or("/")
    }

    /// Generate the sshfs mount command
    pub fn sshfs_command(&self, mountpoint: &str) -> String {
        format!(
            "sshfs -p {} {}@{}:{} {} -o password_stdin",
            self.effective_port(),
            self.user,
            self.ip,
            self.effective_path(),
            mountpoint
        )
    }

    /// Generate a connection string for display
    pub fn connection_string(&self) -> String {
        format!(
            "{}@{}:{}{}",
            self.user,
            self.ip,
            self.effective_port(),
            self.effective_path()
        )
    }

    /// Check if this connection info is still fresh (within 5 minutes)
    pub fn is_fresh(&self) -> bool {
        self.received_at
            .map(|t| t.elapsed().as_secs() < 300)
            .unwrap_or(false)
    }
}

/// Network Share plugin for SFTP mounting
///
/// Stores SFTP connection details received from connected devices
/// and provides an API for accessing them.
pub struct NetworkSharePlugin {
    /// SFTP connection info keyed by device ID
    shares: Arc<RwLock<HashMap<String, SftpInfo>>>,
}

impl NetworkSharePlugin {
    /// Create a new Network Share plugin
    pub fn new() -> Self {
        Self {
            shares: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Handle SFTP packet from a device
    async fn handle_sftp_packet(&self, device: &Device, packet: &Packet) -> Result<()> {
        let mut info: SftpInfo = serde_json::from_value(packet.body.clone()).map_err(|e| {
            crate::ProtocolError::InvalidPacket(format!("Failed to parse SFTP info: {}", e))
        })?;

        info.received_at = Some(std::time::Instant::now());

        info!(
            "Received SFTP connection info from {}: {}",
            device.name(),
            info.connection_string()
        );

        self.shares
            .write()
            .await
            .insert(device.id().to_string(), info);

        debug!("SFTP share stored and ready for mounting");

        Ok(())
    }

    // ========== Public API ==========

    /// Get all active SFTP shares
    ///
    /// Returns a map of device ID to SFTP connection info.
    pub async fn get_shares(&self) -> HashMap<String, SftpInfo> {
        self.shares.read().await.clone()
    }

    /// Get SFTP share info for a specific device
    pub async fn get_share(&self, device_id: &str) -> Option<SftpInfo> {
        self.shares.read().await.get(device_id).cloned()
    }

    /// Check if any SFTP shares are available
    pub async fn has_shares(&self) -> bool {
        !self.shares.read().await.is_empty()
    }

    /// Get the number of active shares
    pub async fn share_count(&self) -> usize {
        self.shares.read().await.len()
    }

    /// Get all fresh shares (received within last 5 minutes)
    pub async fn get_fresh_shares(&self) -> HashMap<String, SftpInfo> {
        self.shares
            .read()
            .await
            .iter()
            .filter(|(_, info)| info.is_fresh())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Remove a share for a specific device
    pub async fn remove_share(&self, device_id: &str) -> Option<SftpInfo> {
        self.shares.write().await.remove(device_id)
    }

    /// Clear all stored shares
    pub async fn clear_shares(&self) {
        self.shares.write().await.clear();
    }
}

impl Default for NetworkSharePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for NetworkSharePlugin {
    fn name(&self) -> &str {
        "networkshare"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn incoming_capabilities(&self) -> Vec<String> {
        vec![
            PACKET_TYPE_SFTP.to_string(),
            PACKET_TYPE_CCONNECT_SFTP.to_string(),
        ]
    }

    fn outgoing_capabilities(&self) -> Vec<String> {
        vec![]
    }

    async fn init(
        &mut self,
        device: &Device,
        _packet_sender: tokio::sync::mpsc::Sender<(String, Packet)>,
    ) -> Result<()> {
        info!(
            "NetworkShare plugin initialized for device {}",
            device.name()
        );
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        info!("NetworkShare plugin started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.clear_shares().await;
        info!("NetworkShare plugin stopped");
        Ok(())
    }

    async fn handle_packet(&mut self, packet: &Packet, device: &mut Device) -> Result<()> {
        if packet.is_type(PACKET_TYPE_SFTP) || packet.is_type(PACKET_TYPE_CCONNECT_SFTP) {
            self.handle_sftp_packet(device, packet).await
        } else {
            warn!(
                "NetworkShare plugin received unknown packet type: {}",
                packet.packet_type
            );
            Ok(())
        }
    }
}

/// Factory for creating NetworkSharePlugin instances
#[derive(Debug, Clone, Copy)]
pub struct NetworkSharePluginFactory;

impl PluginFactory for NetworkSharePluginFactory {
    fn name(&self) -> &str {
        "networkshare"
    }

    fn incoming_capabilities(&self) -> Vec<String> {
        vec![
            PACKET_TYPE_SFTP.to_string(),
            PACKET_TYPE_CCONNECT_SFTP.to_string(),
        ]
    }

    fn outgoing_capabilities(&self) -> Vec<String> {
        vec![]
    }

    fn create(&self) -> Box<dyn Plugin> {
        Box::new(NetworkSharePlugin::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DeviceInfo, DeviceType};
    use serde_json::json;

    fn create_test_device() -> Device {
        let info = DeviceInfo::new("Test Device", DeviceType::Phone, 1716);
        Device::from_discovery(info)
    }

    fn create_test_device_with_id(id: &str, name: &str) -> Device {
        let mut info = DeviceInfo::new(name, DeviceType::Phone, 1716);
        info.device_id = id.to_string();
        Device::from_discovery(info)
    }

    // ========== Basic Plugin Tests ==========

    #[test]
    fn test_plugin_creation() {
        let plugin = NetworkSharePlugin::new();
        assert_eq!(plugin.name(), "networkshare");
    }

    #[test]
    fn test_default() {
        let plugin = NetworkSharePlugin::default();
        assert_eq!(plugin.name(), "networkshare");
    }

    #[test]
    fn test_factory() {
        let factory = NetworkSharePluginFactory;
        assert_eq!(factory.name(), "networkshare");
        assert!(factory
            .incoming_capabilities()
            .contains(&PACKET_TYPE_SFTP.to_string()));
        assert!(factory
            .incoming_capabilities()
            .contains(&PACKET_TYPE_CCONNECT_SFTP.to_string()));
        assert!(factory.outgoing_capabilities().is_empty());
    }

    #[test]
    fn test_factory_create() {
        let factory = NetworkSharePluginFactory;
        let plugin = factory.create();
        assert_eq!(plugin.name(), "networkshare");
    }

    // ========== SftpInfo Tests ==========

    #[test]
    fn test_sftp_info_effective_port_default() {
        let info = SftpInfo {
            ip: "192.168.1.10".to_string(),
            port: None,
            user: "test".to_string(),
            password: "pass".to_string(),
            path: None,
            received_at: None,
        };
        assert_eq!(info.effective_port(), 22);
    }

    #[test]
    fn test_sftp_info_effective_port_custom() {
        let info = SftpInfo {
            ip: "192.168.1.10".to_string(),
            port: Some(1739),
            user: "test".to_string(),
            password: "pass".to_string(),
            path: None,
            received_at: None,
        };
        assert_eq!(info.effective_port(), 1739);
    }

    #[test]
    fn test_sftp_info_effective_path_default() {
        let info = SftpInfo {
            ip: "192.168.1.10".to_string(),
            port: None,
            user: "test".to_string(),
            password: "pass".to_string(),
            path: None,
            received_at: None,
        };
        assert_eq!(info.effective_path(), "/");
    }

    #[test]
    fn test_sftp_info_effective_path_custom() {
        let info = SftpInfo {
            ip: "192.168.1.10".to_string(),
            port: None,
            user: "test".to_string(),
            password: "pass".to_string(),
            path: Some("/storage/emulated/0".to_string()),
            received_at: None,
        };
        assert_eq!(info.effective_path(), "/storage/emulated/0");
    }

    #[test]
    fn test_sftp_info_connection_string() {
        let info = SftpInfo {
            ip: "192.168.1.10".to_string(),
            port: Some(1739),
            user: "kdeconnect".to_string(),
            password: "secret".to_string(),
            path: Some("/sdcard".to_string()),
            received_at: None,
        };
        assert_eq!(
            info.connection_string(),
            "kdeconnect@192.168.1.10:1739/sdcard"
        );
    }

    #[test]
    fn test_sftp_info_sshfs_command() {
        let info = SftpInfo {
            ip: "192.168.1.10".to_string(),
            port: Some(1739),
            user: "kdeconnect".to_string(),
            password: "secret".to_string(),
            path: Some("/sdcard".to_string()),
            received_at: None,
        };
        let cmd = info.sshfs_command("/mnt/phone");
        assert!(cmd.contains("sshfs -p 1739"));
        assert!(cmd.contains("kdeconnect@192.168.1.10:/sdcard"));
        assert!(cmd.contains("/mnt/phone"));
        assert!(cmd.contains("-o password_stdin"));
    }

    #[test]
    fn test_sftp_info_is_fresh_none() {
        let info = SftpInfo {
            ip: "192.168.1.10".to_string(),
            port: None,
            user: "test".to_string(),
            password: "pass".to_string(),
            path: None,
            received_at: None,
        };
        assert!(!info.is_fresh());
    }

    #[test]
    fn test_sftp_info_is_fresh_recent() {
        let info = SftpInfo {
            ip: "192.168.1.10".to_string(),
            port: None,
            user: "test".to_string(),
            password: "pass".to_string(),
            path: None,
            received_at: Some(std::time::Instant::now()),
        };
        assert!(info.is_fresh());
    }

    // ========== Plugin Lifecycle Tests ==========

    #[tokio::test]
    async fn test_plugin_start_stop() {
        let mut plugin = NetworkSharePlugin::new();

        plugin.start().await.unwrap();
        plugin.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_plugin_init() {
        let mut plugin = NetworkSharePlugin::new();
        let device = create_test_device();
        let (tx, _rx) = tokio::sync::mpsc::channel(100);

        let result = plugin.init(&device, tx).await;
        assert!(result.is_ok());
    }

    // ========== Packet Handling Tests ==========

    #[tokio::test]
    async fn test_handle_sftp_packet_kdeconnect() {
        let mut plugin = NetworkSharePlugin::new();
        let mut device = create_test_device();

        let packet = Packet::new(
            PACKET_TYPE_SFTP,
            json!({
                "ip": "192.168.1.50",
                "port": 1739,
                "user": "testuser",
                "password": "secretpassword",
                "path": "/storage/emulated/0"
            }),
        );

        let result = plugin.handle_packet(&packet, &mut device).await;
        assert!(result.is_ok());
        assert!(plugin.has_shares().await);
        assert_eq!(plugin.share_count().await, 1);
    }

    #[tokio::test]
    async fn test_handle_sftp_packet_cconnect() {
        let mut plugin = NetworkSharePlugin::new();
        let mut device = create_test_device();

        let packet = Packet::new(
            PACKET_TYPE_CCONNECT_SFTP,
            json!({
                "ip": "10.0.0.5",
                "port": 2222,
                "user": "cosmicuser",
                "password": "cosmicpass"
            }),
        );

        let result = plugin.handle_packet(&packet, &mut device).await;
        assert!(result.is_ok());
        assert!(plugin.has_shares().await);
    }

    #[tokio::test]
    async fn test_handle_invalid_sftp_packet() {
        let mut plugin = NetworkSharePlugin::new();
        let mut device = create_test_device();

        let packet = Packet::new(
            PACKET_TYPE_SFTP,
            json!({
                "invalid": "data"
            }),
        );

        let result = plugin.handle_packet(&packet, &mut device).await;
        assert!(result.is_err());
    }

    // ========== Public API Tests ==========

    #[tokio::test]
    async fn test_get_share() {
        let mut plugin = NetworkSharePlugin::new();
        let mut device = create_test_device_with_id("device-123", "Test Phone");

        let packet = Packet::new(
            PACKET_TYPE_SFTP,
            json!({
                "ip": "192.168.1.100",
                "port": 1739,
                "user": "kdeconnect",
                "password": "pass123"
            }),
        );

        plugin.handle_packet(&packet, &mut device).await.unwrap();

        let share = plugin.get_share("device-123").await;
        assert!(share.is_some());
        let info = share.unwrap();
        assert_eq!(info.ip, "192.168.1.100");
        assert_eq!(info.effective_port(), 1739);
        assert_eq!(info.user, "kdeconnect");
    }

    #[tokio::test]
    async fn test_get_share_not_found() {
        let plugin = NetworkSharePlugin::new();
        let share = plugin.get_share("nonexistent").await;
        assert!(share.is_none());
    }

    #[tokio::test]
    async fn test_get_shares_multiple_devices() {
        let mut plugin = NetworkSharePlugin::new();

        // First device
        let mut device1 = create_test_device_with_id("phone-1", "Phone 1");
        let packet1 = Packet::new(
            PACKET_TYPE_SFTP,
            json!({
                "ip": "192.168.1.10",
                "user": "user1",
                "password": "pass1"
            }),
        );
        plugin.handle_packet(&packet1, &mut device1).await.unwrap();

        // Second device
        let mut device2 = create_test_device_with_id("phone-2", "Phone 2");
        let packet2 = Packet::new(
            PACKET_TYPE_SFTP,
            json!({
                "ip": "192.168.1.20",
                "user": "user2",
                "password": "pass2"
            }),
        );
        plugin.handle_packet(&packet2, &mut device2).await.unwrap();

        let shares = plugin.get_shares().await;
        assert_eq!(shares.len(), 2);
        assert!(shares.contains_key("phone-1"));
        assert!(shares.contains_key("phone-2"));
    }

    #[tokio::test]
    async fn test_remove_share() {
        let mut plugin = NetworkSharePlugin::new();
        let mut device = create_test_device_with_id("device-to-remove", "Test Device");

        let packet = Packet::new(
            PACKET_TYPE_SFTP,
            json!({
                "ip": "192.168.1.50",
                "user": "test",
                "password": "pass"
            }),
        );

        plugin.handle_packet(&packet, &mut device).await.unwrap();
        assert!(plugin.has_shares().await);

        let removed = plugin.remove_share("device-to-remove").await;
        assert!(removed.is_some());
        assert!(!plugin.has_shares().await);
    }

    #[tokio::test]
    async fn test_clear_shares() {
        let mut plugin = NetworkSharePlugin::new();
        let mut device = create_test_device_with_id("device-1", "Device 1");

        let packet = Packet::new(
            PACKET_TYPE_SFTP,
            json!({
                "ip": "192.168.1.10",
                "user": "test",
                "password": "pass"
            }),
        );

        plugin.handle_packet(&packet, &mut device).await.unwrap();
        assert_eq!(plugin.share_count().await, 1);

        plugin.clear_shares().await;
        assert_eq!(plugin.share_count().await, 0);
    }

    #[tokio::test]
    async fn test_stop_clears_shares() {
        let mut plugin = NetworkSharePlugin::new();
        let mut device = create_test_device();

        let packet = Packet::new(
            PACKET_TYPE_SFTP,
            json!({
                "ip": "192.168.1.50",
                "user": "test",
                "password": "pass"
            }),
        );

        plugin.handle_packet(&packet, &mut device).await.unwrap();
        assert!(plugin.has_shares().await);

        plugin.stop().await.unwrap();
        assert!(!plugin.has_shares().await);
    }

    #[tokio::test]
    async fn test_get_fresh_shares() {
        let mut plugin = NetworkSharePlugin::new();
        let mut device = create_test_device();

        let packet = Packet::new(
            PACKET_TYPE_SFTP,
            json!({
                "ip": "192.168.1.10",
                "user": "test",
                "password": "pass"
            }),
        );

        plugin.handle_packet(&packet, &mut device).await.unwrap();

        let fresh = plugin.get_fresh_shares().await;
        assert_eq!(fresh.len(), 1);
    }

    #[tokio::test]
    async fn test_share_update_replaces_old() {
        let mut plugin = NetworkSharePlugin::new();
        let mut device = create_test_device_with_id("same-device", "Same Device");

        // First share
        let packet1 = Packet::new(
            PACKET_TYPE_SFTP,
            json!({
                "ip": "192.168.1.10",
                "user": "user1",
                "password": "pass1"
            }),
        );
        plugin.handle_packet(&packet1, &mut device).await.unwrap();

        // Second share from same device should replace
        let packet2 = Packet::new(
            PACKET_TYPE_SFTP,
            json!({
                "ip": "192.168.1.20",
                "user": "user2",
                "password": "pass2"
            }),
        );
        plugin.handle_packet(&packet2, &mut device).await.unwrap();

        assert_eq!(plugin.share_count().await, 1);
        let share = plugin.get_share("same-device").await.unwrap();
        assert_eq!(share.ip, "192.168.1.20");
        assert_eq!(share.user, "user2");
    }

    // ========== Capability Tests ==========

    #[test]
    fn test_incoming_capabilities() {
        let plugin = NetworkSharePlugin::new();
        let caps = plugin.incoming_capabilities();
        assert!(caps.contains(&PACKET_TYPE_SFTP.to_string()));
        assert!(caps.contains(&PACKET_TYPE_CCONNECT_SFTP.to_string()));
    }

    #[test]
    fn test_outgoing_capabilities_empty() {
        let plugin = NetworkSharePlugin::new();
        let caps = plugin.outgoing_capabilities();
        assert!(caps.is_empty());
    }
}
