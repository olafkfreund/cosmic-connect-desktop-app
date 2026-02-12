//! Connectivity Report Plugin
//!
//! Receives network connectivity status and signal strength from mobile devices.
//! Stores the latest connectivity information for display in the UI.
//!
//! ## Protocol
//!
//! **Packet Types**:
//! - Incoming: `cconnect.connectivity_report`, `kdeconnect.connectivity_report`
//!
//! **Body Fields**:
//! - `signalStrengths` (Object): Map of subscription ID to signal info
//!   - `networkType` (String): Network type (WiFi, 2G, 3G, 4G, 5G, etc.)
//!   - `signalStrength` (Number): Signal strength (0-4)
//!
//! ## Packet Format
//!
//! ```json
//! {
//!     "id": 1234567890,
//!     "type": "cconnect.connectivity_report",
//!     "body": {
//!         "signalStrengths": {
//!             "0": {
//!                 "networkType": "4G",
//!                 "signalStrength": 3
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ## Signal Strength Values
//!
//! - 0: No signal
//! - 1: Poor
//! - 2: Fair
//! - 3: Good
//! - 4: Excellent
//!
//! ## Network Types
//!
//! Common values: "WiFi", "2G", "3G", "LTE", "4G", "5G", "Unknown"
//!
//! ## Example
//!
//! ```rust,ignore
//! use cosmic_ext_connect_core::plugins::connectivity_report::ConnectivityReportPlugin;
//! use cosmic_ext_connect_core::Plugin;
//!
//! let plugin = ConnectivityReportPlugin::new();
//!
//! // After receiving packets, query the signal info
//! let signals = plugin.get_signal_strengths();
//! for (sub_id, info) in signals {
//!     println!("Sub {}: {} ({}/4)", sub_id, info.network_type, info.signal_strength);
//! }
//! ```
//!
//! ## References
//!
//! - [Valent Protocol Documentation](https://valent.andyholmes.ca/documentation/protocol.html)
//! - [KDE Connect Connectivity Report](https://github.com/KDE/kdeconnect-kde/tree/master/plugins/connectivity-report)

use crate::{Device, Packet, ProtocolError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::{Plugin, PluginFactory};

/// Packet type for connectivity reports
pub const PACKET_TYPE_CONNECTIVITY_REPORT: &str = "cconnect.connectivity_report";

/// KDE Connect compatible packet type
const PACKET_TYPE_KDECONNECT_CONNECTIVITY: &str = "kdeconnect.connectivity_report";

/// Signal strength info for a single subscription/SIM
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalInfo {
    /// Network type (WiFi, 2G, 3G, 4G, 5G, etc.)
    #[serde(rename = "networkType")]
    pub network_type: String,

    /// Signal strength (0-4)
    #[serde(rename = "signalStrength")]
    pub signal_strength: i32,
}

impl SignalInfo {
    /// Create new signal info
    pub fn new(network_type: impl Into<String>, signal_strength: i32) -> Self {
        Self {
            network_type: network_type.into(),
            signal_strength: signal_strength.clamp(0, 4),
        }
    }

    /// Get signal strength as percentage (0-100)
    pub fn strength_percent(&self) -> u8 {
        ((self.signal_strength.clamp(0, 4) as f32 / 4.0) * 100.0) as u8
    }

    /// Get human-readable signal description
    pub fn strength_description(&self) -> &'static str {
        match self.signal_strength {
            0 => "No signal",
            1 => "Poor",
            2 => "Fair",
            3 => "Good",
            _ => "Excellent",
        }
    }

    /// Check if this is WiFi connection
    pub fn is_wifi(&self) -> bool {
        self.network_type.eq_ignore_ascii_case("wifi")
    }

    /// Check if this is mobile data
    pub fn is_mobile(&self) -> bool {
        !self.is_wifi() && self.network_type != "Unknown"
    }
}

impl Default for SignalInfo {
    fn default() -> Self {
        Self {
            network_type: "Unknown".to_string(),
            signal_strength: 0,
        }
    }
}

/// Connectivity report body from packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityReport {
    /// Signal strengths keyed by subscription/SIM ID
    #[serde(rename = "signalStrengths")]
    pub signal_strengths: HashMap<String, SignalInfo>,
}

/// Connectivity Report plugin
///
/// Receives and stores network connectivity information from mobile devices.
pub struct ConnectivityReportPlugin {
    /// Whether the plugin is enabled
    enabled: bool,

    /// Current signal strengths keyed by subscription ID
    signal_strengths: Arc<RwLock<HashMap<String, SignalInfo>>>,
}

impl ConnectivityReportPlugin {
    /// Create a new Connectivity Report plugin
    pub fn new() -> Self {
        Self {
            enabled: false,
            signal_strengths: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get all current signal strengths
    pub async fn get_signal_strengths(&self) -> HashMap<String, SignalInfo> {
        self.signal_strengths.read().await.clone()
    }

    /// Get signal info for a specific subscription
    pub async fn get_signal_info(&self, subscription_id: &str) -> Option<SignalInfo> {
        self.signal_strengths
            .read()
            .await
            .get(subscription_id)
            .cloned()
    }

    /// Get the primary (first) signal info
    ///
    /// Returns the signal info for subscription "0" or the first available.
    pub async fn get_primary_signal(&self) -> Option<SignalInfo> {
        let signals = self.signal_strengths.read().await;

        // Try subscription 0 first
        if let Some(info) = signals.get("0") {
            return Some(info.clone());
        }

        // Fall back to first available
        signals.values().next().cloned()
    }

    /// Check if device has any connectivity
    pub async fn has_connectivity(&self) -> bool {
        let signals = self.signal_strengths.read().await;
        signals.values().any(|s| s.signal_strength > 0)
    }

    /// Handle connectivity report packet
    async fn handle_report(&self, packet: &Packet, device: &Device) -> Result<()> {
        let report: ConnectivityReport = serde_json::from_value(packet.body.clone())
            .map_err(|e| ProtocolError::InvalidPacket(format!("Failed to parse report: {}", e)))?;

        // Log the update
        for (id, info) in &report.signal_strengths {
            info!(
                "Connectivity update from {} - Sub {}: {} ({}/4, {})",
                device.name(),
                id,
                info.network_type,
                info.signal_strength,
                info.strength_description()
            );
        }

        // Update stored signal strengths
        let mut signals = self.signal_strengths.write().await;
        *signals = report.signal_strengths;

        Ok(())
    }

    /// Check if packet is a connectivity report
    fn is_connectivity_packet(packet: &Packet) -> bool {
        packet.is_type(PACKET_TYPE_CONNECTIVITY_REPORT)
            || packet.is_type(PACKET_TYPE_KDECONNECT_CONNECTIVITY)
    }
}

impl Default for ConnectivityReportPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for ConnectivityReportPlugin {
    fn name(&self) -> &str {
        "connectivity_report"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn incoming_capabilities(&self) -> Vec<String> {
        vec![
            PACKET_TYPE_CONNECTIVITY_REPORT.to_string(),
            PACKET_TYPE_KDECONNECT_CONNECTIVITY.to_string(),
        ]
    }

    fn outgoing_capabilities(&self) -> Vec<String> {
        // This plugin only receives reports
        vec![]
    }

    async fn init(
        &mut self,
        device: &Device,
        _packet_sender: tokio::sync::mpsc::Sender<(String, Packet)>,
    ) -> Result<()> {
        info!(
            "Connectivity Report plugin initialized for device {}",
            device.name()
        );
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        self.enabled = true;
        info!("Connectivity Report plugin started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.enabled = false;

        // Clear stored signals
        let mut signals = self.signal_strengths.write().await;
        signals.clear();

        info!("Connectivity Report plugin stopped");
        Ok(())
    }

    async fn handle_packet(&mut self, packet: &Packet, device: &mut Device) -> Result<()> {
        if !self.enabled {
            debug!("Connectivity Report plugin disabled, ignoring packet");
            return Ok(());
        }

        if Self::is_connectivity_packet(packet) {
            self.handle_report(packet, device).await?;
        }

        Ok(())
    }
}

/// Factory for creating ConnectivityReportPlugin instances
#[derive(Debug, Clone, Copy)]
pub struct ConnectivityReportPluginFactory;

impl PluginFactory for ConnectivityReportPluginFactory {
    fn name(&self) -> &str {
        "connectivity_report"
    }

    fn incoming_capabilities(&self) -> Vec<String> {
        vec![
            PACKET_TYPE_CONNECTIVITY_REPORT.to_string(),
            PACKET_TYPE_KDECONNECT_CONNECTIVITY.to_string(),
        ]
    }

    fn outgoing_capabilities(&self) -> Vec<String> {
        vec![]
    }

    fn create(&self) -> Box<dyn Plugin> {
        Box::new(ConnectivityReportPlugin::new())
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

    #[test]
    fn test_signal_info_creation() {
        let info = SignalInfo::new("4G", 3);
        assert_eq!(info.network_type, "4G");
        assert_eq!(info.signal_strength, 3);
    }

    #[test]
    fn test_signal_info_clamping() {
        let info = SignalInfo::new("5G", 10);
        assert_eq!(info.signal_strength, 4); // Clamped to max

        let info2 = SignalInfo::new("2G", -5);
        assert_eq!(info2.signal_strength, 0); // Clamped to min
    }

    #[test]
    fn test_signal_strength_percent() {
        assert_eq!(SignalInfo::new("4G", 0).strength_percent(), 0);
        assert_eq!(SignalInfo::new("4G", 1).strength_percent(), 25);
        assert_eq!(SignalInfo::new("4G", 2).strength_percent(), 50);
        assert_eq!(SignalInfo::new("4G", 3).strength_percent(), 75);
        assert_eq!(SignalInfo::new("4G", 4).strength_percent(), 100);
    }

    #[test]
    fn test_signal_strength_description() {
        assert_eq!(SignalInfo::new("4G", 0).strength_description(), "No signal");
        assert_eq!(SignalInfo::new("4G", 1).strength_description(), "Poor");
        assert_eq!(SignalInfo::new("4G", 2).strength_description(), "Fair");
        assert_eq!(SignalInfo::new("4G", 3).strength_description(), "Good");
        assert_eq!(SignalInfo::new("4G", 4).strength_description(), "Excellent");
    }

    #[test]
    fn test_signal_info_network_type() {
        let wifi = SignalInfo::new("WiFi", 4);
        assert!(wifi.is_wifi());
        assert!(!wifi.is_mobile());

        let lte = SignalInfo::new("LTE", 3);
        assert!(!lte.is_wifi());
        assert!(lte.is_mobile());

        let unknown = SignalInfo::new("Unknown", 0);
        assert!(!unknown.is_wifi());
        assert!(!unknown.is_mobile());
    }

    #[test]
    fn test_plugin_creation() {
        let plugin = ConnectivityReportPlugin::new();
        assert_eq!(plugin.name(), "connectivity_report");
        assert!(!plugin.enabled);
    }

    #[test]
    fn test_capabilities() {
        let plugin = ConnectivityReportPlugin::new();

        let incoming = plugin.incoming_capabilities();
        assert_eq!(incoming.len(), 2);
        assert!(incoming.contains(&PACKET_TYPE_CONNECTIVITY_REPORT.to_string()));
        assert!(incoming.contains(&PACKET_TYPE_KDECONNECT_CONNECTIVITY.to_string()));

        let outgoing = plugin.outgoing_capabilities();
        assert!(outgoing.is_empty());
    }

    #[tokio::test]
    async fn test_plugin_lifecycle() {
        let mut plugin = ConnectivityReportPlugin::new();
        let device = create_test_device();

        // Initialize
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        plugin.init(&device, tx).await.unwrap();

        // Start
        plugin.start().await.unwrap();
        assert!(plugin.enabled);

        // Stop
        plugin.stop().await.unwrap();
        assert!(!plugin.enabled);
    }

    #[tokio::test]
    async fn test_handle_connectivity_report() {
        let mut plugin = ConnectivityReportPlugin::new();
        let device = create_test_device();

        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        plugin.init(&device, tx).await.unwrap();
        plugin.start().await.unwrap();

        // Send connectivity report
        let mut device = create_test_device();
        let packet = Packet::new(
            PACKET_TYPE_CONNECTIVITY_REPORT,
            json!({
                "signalStrengths": {
                    "0": {
                        "networkType": "LTE",
                        "signalStrength": 3
                    }
                }
            }),
        );

        plugin.handle_packet(&packet, &mut device).await.unwrap();

        // Check stored signal
        let signals = plugin.get_signal_strengths().await;
        assert_eq!(signals.len(), 1);

        let info = signals.get("0").unwrap();
        assert_eq!(info.network_type, "LTE");
        assert_eq!(info.signal_strength, 3);
    }

    #[tokio::test]
    async fn test_handle_kdeconnect_packet() {
        let mut plugin = ConnectivityReportPlugin::new();
        let device = create_test_device();

        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        plugin.init(&device, tx).await.unwrap();
        plugin.start().await.unwrap();

        // Send KDE Connect format packet
        let mut device = create_test_device();
        let packet = Packet::new(
            PACKET_TYPE_KDECONNECT_CONNECTIVITY,
            json!({
                "signalStrengths": {
                    "0": {
                        "networkType": "5G",
                        "signalStrength": 4
                    }
                }
            }),
        );

        plugin.handle_packet(&packet, &mut device).await.unwrap();

        // Check stored signal
        let info = plugin.get_primary_signal().await.unwrap();
        assert_eq!(info.network_type, "5G");
        assert_eq!(info.signal_strength, 4);
    }

    #[tokio::test]
    async fn test_multiple_subscriptions() {
        let mut plugin = ConnectivityReportPlugin::new();
        let device = create_test_device();

        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        plugin.init(&device, tx).await.unwrap();
        plugin.start().await.unwrap();

        // Send report with multiple SIMs
        let mut device = create_test_device();
        let packet = Packet::new(
            PACKET_TYPE_CONNECTIVITY_REPORT,
            json!({
                "signalStrengths": {
                    "0": {
                        "networkType": "4G",
                        "signalStrength": 2
                    },
                    "1": {
                        "networkType": "3G",
                        "signalStrength": 1
                    }
                }
            }),
        );

        plugin.handle_packet(&packet, &mut device).await.unwrap();

        // Check both subscriptions
        let signals = plugin.get_signal_strengths().await;
        assert_eq!(signals.len(), 2);

        let sim0 = plugin.get_signal_info("0").await.unwrap();
        assert_eq!(sim0.network_type, "4G");

        let sim1 = plugin.get_signal_info("1").await.unwrap();
        assert_eq!(sim1.network_type, "3G");
    }

    #[tokio::test]
    async fn test_get_primary_signal() {
        let mut plugin = ConnectivityReportPlugin::new();
        let device = create_test_device();

        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        plugin.init(&device, tx).await.unwrap();
        plugin.start().await.unwrap();

        // Initially no signal
        assert!(plugin.get_primary_signal().await.is_none());

        // Add signal
        let mut device = create_test_device();
        let packet = Packet::new(
            PACKET_TYPE_CONNECTIVITY_REPORT,
            json!({
                "signalStrengths": {
                    "0": {
                        "networkType": "WiFi",
                        "signalStrength": 4
                    }
                }
            }),
        );

        plugin.handle_packet(&packet, &mut device).await.unwrap();

        // Check primary signal
        let primary = plugin.get_primary_signal().await.unwrap();
        assert!(primary.is_wifi());
        assert_eq!(primary.signal_strength, 4);
    }

    #[tokio::test]
    async fn test_has_connectivity() {
        let mut plugin = ConnectivityReportPlugin::new();
        let device = create_test_device();

        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        plugin.init(&device, tx).await.unwrap();
        plugin.start().await.unwrap();

        // No connectivity initially
        assert!(!plugin.has_connectivity().await);

        // Add signal with strength 0
        let mut device = create_test_device();
        let packet = Packet::new(
            PACKET_TYPE_CONNECTIVITY_REPORT,
            json!({
                "signalStrengths": {
                    "0": {
                        "networkType": "4G",
                        "signalStrength": 0
                    }
                }
            }),
        );

        plugin.handle_packet(&packet, &mut device).await.unwrap();
        assert!(!plugin.has_connectivity().await);

        // Update with actual signal
        let packet2 = Packet::new(
            PACKET_TYPE_CONNECTIVITY_REPORT,
            json!({
                "signalStrengths": {
                    "0": {
                        "networkType": "4G",
                        "signalStrength": 2
                    }
                }
            }),
        );

        plugin.handle_packet(&packet2, &mut device).await.unwrap();
        assert!(plugin.has_connectivity().await);
    }

    #[tokio::test]
    async fn test_disabled_plugin_ignores_packets() {
        let mut plugin = ConnectivityReportPlugin::new();
        let device = create_test_device();

        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        plugin.init(&device, tx).await.unwrap();
        // Don't start - plugin remains disabled

        // Send packet
        let mut device = create_test_device();
        let packet = Packet::new(
            PACKET_TYPE_CONNECTIVITY_REPORT,
            json!({
                "signalStrengths": {
                    "0": {
                        "networkType": "5G",
                        "signalStrength": 4
                    }
                }
            }),
        );

        plugin.handle_packet(&packet, &mut device).await.unwrap();

        // Should be empty (packet ignored)
        let signals = plugin.get_signal_strengths().await;
        assert!(signals.is_empty());
    }

    #[tokio::test]
    async fn test_stop_clears_signals() {
        let mut plugin = ConnectivityReportPlugin::new();
        let device = create_test_device();

        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        plugin.init(&device, tx).await.unwrap();
        plugin.start().await.unwrap();

        // Add signal
        let mut device = create_test_device();
        let packet = Packet::new(
            PACKET_TYPE_CONNECTIVITY_REPORT,
            json!({
                "signalStrengths": {
                    "0": {
                        "networkType": "4G",
                        "signalStrength": 3
                    }
                }
            }),
        );

        plugin.handle_packet(&packet, &mut device).await.unwrap();
        assert!(!plugin.get_signal_strengths().await.is_empty());

        // Stop plugin
        plugin.stop().await.unwrap();

        // Signals should be cleared
        assert!(plugin.get_signal_strengths().await.is_empty());
    }

    #[test]
    fn test_is_connectivity_packet() {
        let cconnect = Packet::new(PACKET_TYPE_CONNECTIVITY_REPORT, json!({}));
        let kdeconnect = Packet::new(PACKET_TYPE_KDECONNECT_CONNECTIVITY, json!({}));
        let other = Packet::new("other.packet", json!({}));

        assert!(ConnectivityReportPlugin::is_connectivity_packet(&cconnect));
        assert!(ConnectivityReportPlugin::is_connectivity_packet(
            &kdeconnect
        ));
        assert!(!ConnectivityReportPlugin::is_connectivity_packet(&other));
    }

    #[test]
    fn test_factory() {
        let factory = ConnectivityReportPluginFactory;
        assert_eq!(factory.name(), "connectivity_report");

        let incoming = factory.incoming_capabilities();
        assert_eq!(incoming.len(), 2);

        let outgoing = factory.outgoing_capabilities();
        assert!(outgoing.is_empty());

        let plugin = factory.create();
        assert_eq!(plugin.name(), "connectivity_report");
    }
}
