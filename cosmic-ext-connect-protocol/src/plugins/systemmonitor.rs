//! System Monitor Plugin
//!
//! Provides real-time system monitoring capabilities for remote desktop machines.
//! Allows viewing CPU, memory, disk, network statistics, and process lists.
//!
//! ## Protocol
//!
//! **Packet Types**:
//! - `cconnect.systemmonitor.request` - Request system statistics
//! - `cconnect.systemmonitor.stats` - System statistics response
//! - `cconnect.systemmonitor.processes` - Process list response
//!
//! **Capabilities**:
//! - Incoming: `cconnect.systemmonitor.request`
//! - Outgoing: `cconnect.systemmonitor.stats`, `cconnect.systemmonitor.processes`
//!
//! ## Packet Formats
//!
//! ### Request System Statistics
//!
//! ```json
//! {
//!     "id": 1234567890,
//!     "type": "cconnect.systemmonitor.request",
//!     "body": {
//!         "requestType": "stats"
//!     }
//! }
//! ```
//!
//! ### Statistics Response
//!
//! ```json
//! {
//!     "id": 1234567891,
//!     "type": "cconnect.systemmonitor.stats",
//!     "body": {
//!         "cpu": {
//!             "usage": 45.2,
//!             "cores": [12.3, 45.6, 78.9, 34.5]
//!         },
//!         "memory": {
//!             "total": 16777216000,
//!             "used": 8388608000,
//!             "available": 8388608000,
//!             "usagePercent": 50.0
//!         },
//!         "disk": [
//!             {
//!                 "mountPoint": "/",
//!                 "total": 500000000000,
//!                 "used": 250000000000,
//!                 "available": 250000000000,
//!                 "usagePercent": 50.0
//!             }
//!         ],
//!         "network": {
//!             "bytesReceived": 1234567890,
//!             "bytesSent": 987654321
//!         },
//!         "uptime": 86400
//!     }
//! }
//! ```
//!
//! ### Request Process List
//!
//! ```json
//! {
//!     "id": 1234567892,
//!     "type": "cconnect.systemmonitor.request",
//!     "body": {
//!         "requestType": "processes",
//!         "limit": 10
//!     }
//! }
//! ```
//!
//! ### Process List Response
//!
//! ```json
//! {
//!     "id": 1234567893,
//!     "type": "cconnect.systemmonitor.processes",
//!     "body": {
//!         "processes": [
//!             {
//!                 "pid": 1234,
//!                 "name": "firefox",
//!                 "cpu": 12.5,
//!                 "memory": 1073741824
//!             }
//!         ]
//!     }
//! }
//! ```
//!
//! ## Use Cases
//!
//! - Monitor remote desktop system resources
//! - View CPU and memory usage
//! - Check disk space availability
//! - Monitor network traffic
//! - Identify resource-intensive processes
//!
//! ## Platform Support
//!
//! - **Linux**: Full support via /proc filesystem
//! - **macOS**: Limited support (minimal stats)
//! - **Windows**: Limited support (minimal stats)

use crate::{Device, Packet, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::{Plugin, PluginFactory};

/// CPU statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuStats {
    /// Overall CPU usage percentage (0-100)
    pub usage: f64,
    /// Per-core CPU usage percentages
    pub cores: Vec<f64>,
}

/// Memory statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total memory in bytes
    pub total: u64,
    /// Used memory in bytes
    pub used: u64,
    /// Available memory in bytes
    pub available: u64,
    /// Memory usage percentage
    #[serde(rename = "usagePercent")]
    pub usage_percent: f64,
}

/// Disk statistics for a single mount point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskStats {
    /// Mount point path
    #[serde(rename = "mountPoint")]
    pub mount_point: String,
    /// Total space in bytes
    pub total: u64,
    /// Used space in bytes
    pub used: u64,
    /// Available space in bytes
    pub available: u64,
    /// Usage percentage
    #[serde(rename = "usagePercent")]
    pub usage_percent: f64,
}

/// Network statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Total bytes received across all interfaces
    #[serde(rename = "bytesReceived")]
    pub bytes_received: u64,
    /// Total bytes sent across all interfaces
    #[serde(rename = "bytesSent")]
    pub bytes_sent: u64,
}

/// Process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// CPU usage percentage
    pub cpu: f64,
    /// Memory usage in bytes
    pub memory: u64,
}

/// Complete system statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemStats {
    /// CPU statistics
    pub cpu: CpuStats,
    /// Memory statistics
    pub memory: MemoryStats,
    /// Disk statistics for all mount points
    pub disk: Vec<DiskStats>,
    /// Network statistics
    pub network: NetworkStats,
    /// System uptime in seconds
    pub uptime: u64,
}

/// System Monitor plugin for viewing remote system resources
///
/// Handles `cconnect.systemmonitor.*` packets for system monitoring.
///
/// ## Features
///
/// - Real-time CPU, memory, disk, and network statistics
/// - Process list with resource usage
/// - Thread-safe stats caching
/// - Public API for UI integration
#[derive(Debug)]
pub struct SystemMonitorPlugin {
    /// Device ID this plugin is attached to
    device_id: Option<String>,

    /// Whether the plugin is enabled
    enabled: bool,

    /// Cached system statistics
    stats: Arc<RwLock<SystemStats>>,

    /// Cached process list
    processes: Arc<RwLock<Vec<ProcessInfo>>>,

    /// Packet sender for response packets
    packet_sender: Option<tokio::sync::mpsc::Sender<(String, Packet)>>,
}

impl SystemMonitorPlugin {
    /// Create a new SystemMonitor plugin
    pub fn new() -> Self {
        Self {
            device_id: None,
            enabled: true,
            stats: Arc::new(RwLock::new(SystemStats::default())),
            processes: Arc::new(RwLock::new(Vec::new())),
            packet_sender: None,
        }
    }

    /// Get cached system statistics
    ///
    /// Returns the most recently collected system stats.
    pub fn get_stats(&self) -> SystemStats {
        self.stats
            .try_read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    /// Get CPU statistics
    pub fn get_cpu_stats(&self) -> CpuStats {
        self.get_stats().cpu
    }

    /// Get memory statistics
    pub fn get_memory_stats(&self) -> MemoryStats {
        self.get_stats().memory
    }

    /// Get disk statistics for all mount points
    pub fn get_disk_stats(&self) -> Vec<DiskStats> {
        self.get_stats().disk
    }

    /// Get network statistics
    pub fn get_network_stats(&self) -> NetworkStats {
        self.get_stats().network
    }

    /// Get system uptime in seconds
    pub fn get_uptime_secs(&self) -> u64 {
        self.get_stats().uptime
    }

    /// Get cached process list
    pub fn get_processes(&self) -> Vec<ProcessInfo> {
        self.processes
            .try_read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    /// Get number of cached processes
    pub fn process_count(&self) -> usize {
        self.processes
            .try_read()
            .map(|guard| guard.len())
            .unwrap_or(0)
    }

    /// Create a stats request packet
    pub fn create_stats_request(&self) -> Packet {
        Packet::new(
            "cconnect.systemmonitor.request",
            json!({ "requestType": "stats" }),
        )
    }

    /// Create a process list request packet
    pub fn create_processes_request(&self, limit: usize) -> Packet {
        Packet::new(
            "cconnect.systemmonitor.request",
            json!({
                "requestType": "processes",
                "limit": limit
            }),
        )
    }

    /// Update cached stats
    fn update_stats(&self, stats: SystemStats) {
        if let Ok(mut guard) = self.stats.try_write() {
            *guard = stats;
        }
    }

    /// Update cached processes
    fn update_processes(&self, processes: Vec<ProcessInfo>) {
        if let Ok(mut guard) = self.processes.try_write() {
            *guard = processes;
        }
    }

    /// Collect current system statistics
    fn collect_system_stats(&self) -> serde_json::Value {
        #[cfg(target_os = "linux")]
        {
            json!({
                "cpu": self.get_cpu_usage(),
                "memory": self.get_memory_info(),
                "disk": self.get_disk_info(),
                "network": self.get_network_info(),
                "uptime": self.get_uptime(),
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            json!({
                "cpu": { "usage": 0.0, "cores": [] },
                "memory": { "total": 0, "used": 0, "available": 0, "usagePercent": 0.0 },
                "disk": [],
                "network": { "bytesReceived": 0, "bytesSent": 0 },
                "uptime": 0,
            })
        }
    }

    #[cfg(target_os = "linux")]
    fn get_cpu_usage(&self) -> serde_json::Value {
        use std::fs;

        let Ok(stat_content) = fs::read_to_string("/proc/stat") else {
            return json!({ "usage": 0.0, "cores": [] });
        };

        let mut cores = Vec::new();
        let mut total_usage = 0.0;

        for line in stat_content.lines() {
            if line.starts_with("cpu") && !line.starts_with("cpu ") {
                if let Some(usage) = self.parse_cpu_line(line) {
                    cores.push(usage);
                    total_usage += usage;
                }
            }
        }

        let avg_usage = if cores.is_empty() {
            0.0
        } else {
            total_usage / cores.len() as f64
        };

        json!({
            "usage": (avg_usage * 100.0).round() / 100.0,
            "cores": cores.iter().map(|u| (u * 100.0).round() / 100.0).collect::<Vec<f64>>(),
        })
    }

    #[cfg(target_os = "linux")]
    fn parse_cpu_line(&self, line: &str) -> Option<f64> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return None;
        }

        let user: u64 = parts.get(1)?.parse().ok()?;
        let nice: u64 = parts.get(2)?.parse().ok()?;
        let system: u64 = parts.get(3)?.parse().ok()?;
        let idle: u64 = parts.get(4)?.parse().ok()?;

        let total = user + nice + system + idle;
        let active = user + nice + system;

        if total == 0 {
            return Some(0.0);
        }

        Some(active as f64 / total as f64)
    }

    #[cfg(target_os = "linux")]
    fn get_memory_info(&self) -> serde_json::Value {
        use std::fs;

        let Ok(meminfo_content) = fs::read_to_string("/proc/meminfo") else {
            return json!({ "total": 0, "used": 0, "available": 0, "usagePercent": 0.0 });
        };

        let mut mem_total = 0u64;
        let mut mem_available = 0u64;

        for line in meminfo_content.lines() {
            if let Some(value) = line.strip_prefix("MemTotal:") {
                mem_total = value
                    .split_whitespace()
                    .next()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0)
                    * 1024;
            } else if let Some(value) = line.strip_prefix("MemAvailable:") {
                mem_available = value
                    .split_whitespace()
                    .next()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0)
                    * 1024;
            }
        }

        let mem_used = mem_total.saturating_sub(mem_available);
        let usage_percent = if mem_total > 0 {
            (mem_used as f64 / mem_total as f64) * 100.0
        } else {
            0.0
        };

        json!({
            "total": mem_total,
            "used": mem_used,
            "available": mem_available,
            "usagePercent": (usage_percent * 100.0).round() / 100.0,
        })
    }

    #[cfg(target_os = "linux")]
    fn get_disk_info(&self) -> serde_json::Value {
        use std::collections::HashSet;
        use std::fs;

        let Ok(mounts_content) = fs::read_to_string("/proc/mounts") else {
            return json!([]);
        };

        let mut disks = Vec::new();
        let mut seen_devices = HashSet::new();

        for line in mounts_content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }

            let device = parts[0];
            let mount_point = parts[1];
            let fs_type = parts[2];

            // Skip non-physical filesystems and duplicates
            if !device.starts_with("/dev/") || fs_type == "squashfs" || fs_type == "tmpfs" {
                continue;
            }
            if !seen_devices.insert(device.to_string()) {
                continue;
            }

            let Ok(stat) = nix::sys::statvfs::statvfs(mount_point) else {
                continue;
            };

            let block_size = stat.block_size();
            let total = stat.blocks() * block_size;
            let available = stat.blocks_available() * block_size;
            let used = total - available;

            let usage_percent = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            disks.push(json!({
                "mountPoint": mount_point,
                "total": total,
                "used": used,
                "available": available,
                "usagePercent": (usage_percent * 100.0).round() / 100.0,
            }));
        }

        json!(disks)
    }

    #[cfg(target_os = "linux")]
    fn get_network_info(&self) -> serde_json::Value {
        use std::fs;

        let Ok(netdev_content) = fs::read_to_string("/proc/net/dev") else {
            return json!({ "bytesReceived": 0, "bytesSent": 0 });
        };

        let mut total_received = 0u64;
        let mut total_sent = 0u64;

        for line in netdev_content.lines().skip(2) {
            let Some((iface, stats)) = line.split_once(':') else {
                continue;
            };

            if iface.trim() == "lo" {
                continue;
            }

            let parts: Vec<&str> = stats.split_whitespace().collect();
            if parts.len() >= 9 {
                total_received += parts[0].parse::<u64>().unwrap_or(0);
                total_sent += parts[8].parse::<u64>().unwrap_or(0);
            }
        }

        json!({
            "bytesReceived": total_received,
            "bytesSent": total_sent,
        })
    }

    #[cfg(target_os = "linux")]
    fn get_uptime(&self) -> u64 {
        use std::fs;

        if let Ok(uptime_content) = fs::read_to_string("/proc/uptime") {
            if let Some(uptime_str) = uptime_content.split_whitespace().next() {
                if let Ok(uptime_float) = uptime_str.parse::<f64>() {
                    return uptime_float as u64;
                }
            }
        }
        0
    }

    /// Collect top processes by resource usage
    fn collect_process_list(&self, limit: usize) -> serde_json::Value {
        #[cfg(target_os = "linux")]
        {
            use std::fs;

            let Ok(entries) = fs::read_dir("/proc") else {
                return json!({ "processes": [] });
            };

            let mut processes: Vec<_> = entries
                .flatten()
                .filter_map(|entry| {
                    let file_name = entry.file_name().into_string().ok()?;
                    let pid = file_name.parse::<u32>().ok()?;
                    self.get_process_info(pid)
                })
                .collect();

            // Sort by CPU usage (descending)
            processes.sort_by(|a, b| {
                let cpu_a = a["cpu"].as_f64().unwrap_or(0.0);
                let cpu_b = b["cpu"].as_f64().unwrap_or(0.0);
                cpu_b
                    .partial_cmp(&cpu_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            processes.truncate(limit);

            json!({ "processes": processes })
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = limit;
            json!({ "processes": [] })
        }
    }

    #[cfg(target_os = "linux")]
    fn get_process_info(&self, pid: u32) -> Option<serde_json::Value> {
        use std::fs;

        let stat_content = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;

        let start = stat_content.find('(')?;
        let end = stat_content.rfind(')')?;
        let name = &stat_content[start + 1..end];

        let stats_part = &stat_content[end + 2..];
        let parts: Vec<&str> = stats_part.split_whitespace().collect();

        let memory = fs::read_to_string(format!("/proc/{pid}/statm"))
            .ok()
            .and_then(|content| {
                content
                    .split_whitespace()
                    .nth(1)?
                    .parse::<u64>()
                    .ok()
                    .map(|rss| rss * 4096)
            })
            .unwrap_or(0);

        let utime: u64 = parts.get(11)?.parse().ok()?;
        let stime: u64 = parts.get(12)?.parse().ok()?;
        let cpu_percent = ((utime + stime) as f64 / 1000.0).min(100.0);

        Some(json!({
            "pid": pid,
            "name": name,
            "cpu": (cpu_percent * 100.0).round() / 100.0,
            "memory": memory,
        }))
    }

    /// Handle system monitor request
    async fn handle_request(&mut self, packet: &Packet, device: &Device) -> Result<()> {
        debug!("Handling system monitor request from {}", device.name());

        let request_type = packet
            .body
            .get("requestType")
            .and_then(|v| v.as_str())
            .unwrap_or("stats");

        match request_type {
            "stats" => {
                info!("Collecting system statistics for {}", device.name());
                let stats_json = self.collect_system_stats();

                if let Ok(stats) = serde_json::from_value::<SystemStats>(stats_json.clone()) {
                    self.update_stats(stats);
                }

                let response = Packet::new("cconnect.systemmonitor.stats", stats_json);
                debug!(
                    "System stats collected for {}: {:?}",
                    device.name(),
                    response.body
                );

                // Send response packet
                if let (Some(device_id), Some(sender)) = (&self.device_id, &self.packet_sender) {
                    if let Err(e) = sender.send((device_id.clone(), response)).await {
                        warn!("Failed to send system stats packet: {}", e);
                    }
                } else {
                    warn!("Cannot send system stats - plugin not properly initialized");
                }
            }
            "processes" => {
                let limit = packet
                    .body
                    .get("limit")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(10) as usize;

                info!("Collecting top {} processes for {}", limit, device.name());
                let process_list = self.collect_process_list(limit);

                if let Some(processes_array) = process_list.get("processes") {
                    if let Ok(processes) =
                        serde_json::from_value::<Vec<ProcessInfo>>(processes_array.clone())
                    {
                        self.update_processes(processes);
                    }
                }

                let response = Packet::new("cconnect.systemmonitor.processes", process_list);
                debug!(
                    "Process list collected for {}: {:?}",
                    device.name(),
                    response.body
                );

                // Send response packet
                if let (Some(device_id), Some(sender)) = (&self.device_id, &self.packet_sender) {
                    if let Err(e) = sender.send((device_id.clone(), response)).await {
                        warn!("Failed to send process list packet: {}", e);
                    }
                } else {
                    warn!("Cannot send process list - plugin not properly initialized");
                }
            }
            _ => {
                warn!("Unknown system monitor request type: {}", request_type);
            }
        }

        Ok(())
    }
}

impl Default for SystemMonitorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for SystemMonitorPlugin {
    fn name(&self) -> &str {
        "systemmonitor"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn incoming_capabilities(&self) -> Vec<String> {
        vec![
            "cconnect.systemmonitor.request".to_string(),
            "kdeconnect.systemmonitor.request".to_string(),
        ]
    }

    fn outgoing_capabilities(&self) -> Vec<String> {
        vec![
            "cconnect.systemmonitor.stats".to_string(),
            "cconnect.systemmonitor.processes".to_string(),
        ]
    }

    async fn init(
        &mut self,
        device: &Device,
        packet_sender: tokio::sync::mpsc::Sender<(String, Packet)>,
    ) -> Result<()> {
        self.device_id = Some(device.id().to_string());
        self.packet_sender = Some(packet_sender);
        info!(
            "SystemMonitor plugin initialized for device {}",
            device.name()
        );
        Ok(())
    }

    async fn start(&mut self) -> Result<()> {
        info!("SystemMonitor plugin started");
        self.enabled = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        info!("SystemMonitor plugin stopped");
        self.enabled = false;
        Ok(())
    }

    async fn handle_packet(&mut self, packet: &Packet, device: &mut Device) -> Result<()> {
        if !self.enabled {
            debug!("SystemMonitor plugin is disabled, ignoring packet");
            return Ok(());
        }

        if packet.is_type("cconnect.systemmonitor.request") {
            self.handle_request(packet, device).await
        } else {
            Ok(())
        }
    }
}

/// Factory for creating SystemMonitorPlugin instances
#[derive(Debug, Clone, Copy)]
pub struct SystemMonitorPluginFactory;

impl PluginFactory for SystemMonitorPluginFactory {
    fn name(&self) -> &str {
        "systemmonitor"
    }

    fn incoming_capabilities(&self) -> Vec<String> {
        vec![
            "cconnect.systemmonitor.request".to_string(),
            "kdeconnect.systemmonitor.request".to_string(),
        ]
    }

    fn outgoing_capabilities(&self) -> Vec<String> {
        vec![
            "cconnect.systemmonitor.stats".to_string(),
            "cconnect.systemmonitor.processes".to_string(),
        ]
    }

    fn create(&self) -> Box<dyn Plugin> {
        Box::new(SystemMonitorPlugin::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DeviceInfo, DeviceType};

    fn create_test_device() -> Device {
        let info = DeviceInfo::new("Test Device", DeviceType::Desktop, 1716);
        Device::from_discovery(info)
    }

    #[test]
    fn test_plugin_creation() {
        let plugin = SystemMonitorPlugin::new();
        assert_eq!(plugin.name(), "systemmonitor");
        assert!(plugin.enabled);
    }

    #[test]
    fn test_capabilities() {
        let plugin = SystemMonitorPlugin::new();

        let incoming = plugin.incoming_capabilities();
        assert_eq!(incoming.len(), 2);
        assert!(incoming.contains(&"cconnect.systemmonitor.request".to_string()));
        assert!(incoming.contains(&"kdeconnect.systemmonitor.request".to_string()));

        let outgoing = plugin.outgoing_capabilities();
        assert_eq!(outgoing.len(), 2);
        assert!(outgoing.contains(&"cconnect.systemmonitor.stats".to_string()));
        assert!(outgoing.contains(&"cconnect.systemmonitor.processes".to_string()));
    }

    #[tokio::test]
    async fn test_plugin_lifecycle() {
        let mut plugin = SystemMonitorPlugin::new();
        let device = create_test_device();

        plugin
            .init(&device, tokio::sync::mpsc::channel(100).0)
            .await
            .unwrap();
        assert!(plugin.device_id.is_some());

        plugin.start().await.unwrap();
        assert!(plugin.enabled);

        plugin.stop().await.unwrap();
        assert!(!plugin.enabled);
    }

    #[tokio::test]
    async fn test_handle_stats_request() {
        let mut plugin = SystemMonitorPlugin::new();
        let device = create_test_device();
        plugin
            .init(&device, tokio::sync::mpsc::channel(100).0)
            .await
            .unwrap();
        plugin.start().await.unwrap();

        let mut device = create_test_device();
        let packet = Packet::new(
            "cconnect.systemmonitor.request",
            json!({
                "requestType": "stats"
            }),
        );

        let result = plugin.handle_packet(&packet, &mut device).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_processes_request() {
        let mut plugin = SystemMonitorPlugin::new();
        let device = create_test_device();
        plugin
            .init(&device, tokio::sync::mpsc::channel(100).0)
            .await
            .unwrap();
        plugin.start().await.unwrap();

        let mut device = create_test_device();
        let packet = Packet::new(
            "cconnect.systemmonitor.request",
            json!({
                "requestType": "processes",
                "limit": 5
            }),
        );

        let result = plugin.handle_packet(&packet, &mut device).await;
        assert!(result.is_ok());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_collect_system_stats() {
        let plugin = SystemMonitorPlugin::new();
        let stats = plugin.collect_system_stats();

        assert!(stats.get("cpu").is_some());
        assert!(stats.get("memory").is_some());
        assert!(stats.get("disk").is_some());
        assert!(stats.get("network").is_some());
        assert!(stats.get("uptime").is_some());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_collect_process_list() {
        let plugin = SystemMonitorPlugin::new();
        let processes = plugin.collect_process_list(5);

        assert!(processes.get("processes").is_some());
        assert!(processes["processes"].is_array());
    }

    #[test]
    fn test_factory() {
        let factory = SystemMonitorPluginFactory;
        assert_eq!(factory.name(), "systemmonitor");

        let plugin = factory.create();
        assert_eq!(plugin.name(), "systemmonitor");
    }

    #[test]
    fn test_initial_stats_empty() {
        let plugin = SystemMonitorPlugin::new();

        let stats = plugin.get_stats();
        assert_eq!(stats.cpu.usage, 0.0);
        assert!(stats.cpu.cores.is_empty());
        assert_eq!(stats.memory.total, 0);
        assert!(stats.disk.is_empty());
        assert_eq!(stats.network.bytes_received, 0);
        assert_eq!(stats.uptime, 0);
    }

    #[test]
    fn test_get_cpu_stats() {
        let plugin = SystemMonitorPlugin::new();
        let cpu = plugin.get_cpu_stats();
        assert_eq!(cpu.usage, 0.0);
        assert!(cpu.cores.is_empty());
    }

    #[test]
    fn test_get_memory_stats() {
        let plugin = SystemMonitorPlugin::new();
        let memory = plugin.get_memory_stats();
        assert_eq!(memory.total, 0);
        assert_eq!(memory.used, 0);
        assert_eq!(memory.available, 0);
        assert_eq!(memory.usage_percent, 0.0);
    }

    #[test]
    fn test_get_disk_stats() {
        let plugin = SystemMonitorPlugin::new();
        let disks = plugin.get_disk_stats();
        assert!(disks.is_empty());
    }

    #[test]
    fn test_get_network_stats() {
        let plugin = SystemMonitorPlugin::new();
        let network = plugin.get_network_stats();
        assert_eq!(network.bytes_received, 0);
        assert_eq!(network.bytes_sent, 0);
    }

    #[test]
    fn test_get_uptime() {
        let plugin = SystemMonitorPlugin::new();
        assert_eq!(plugin.get_uptime_secs(), 0);
    }

    #[test]
    fn test_get_processes_empty() {
        let plugin = SystemMonitorPlugin::new();
        let processes = plugin.get_processes();
        assert!(processes.is_empty());
        assert_eq!(plugin.process_count(), 0);
    }

    #[test]
    fn test_update_stats() {
        let plugin = SystemMonitorPlugin::new();

        let stats = SystemStats {
            cpu: CpuStats {
                usage: 45.5,
                cores: vec![40.0, 50.0, 45.0, 47.0],
            },
            memory: MemoryStats {
                total: 16_000_000_000,
                used: 8_000_000_000,
                available: 8_000_000_000,
                usage_percent: 50.0,
            },
            disk: vec![DiskStats {
                mount_point: "/".to_string(),
                total: 500_000_000_000,
                used: 250_000_000_000,
                available: 250_000_000_000,
                usage_percent: 50.0,
            }],
            network: NetworkStats {
                bytes_received: 1_000_000,
                bytes_sent: 500_000,
            },
            uptime: 86400,
        };

        plugin.update_stats(stats);

        let cached = plugin.get_stats();
        assert_eq!(cached.cpu.usage, 45.5);
        assert_eq!(cached.cpu.cores.len(), 4);
        assert_eq!(cached.memory.total, 16_000_000_000);
        assert_eq!(cached.disk.len(), 1);
        assert_eq!(cached.disk[0].mount_point, "/");
        assert_eq!(cached.network.bytes_received, 1_000_000);
        assert_eq!(cached.uptime, 86400);
    }

    #[test]
    fn test_update_processes() {
        let plugin = SystemMonitorPlugin::new();

        let processes = vec![
            ProcessInfo {
                pid: 1234,
                name: "firefox".to_string(),
                cpu: 12.5,
                memory: 1_000_000_000,
            },
            ProcessInfo {
                pid: 5678,
                name: "code".to_string(),
                cpu: 8.3,
                memory: 500_000_000,
            },
        ];

        plugin.update_processes(processes);

        let cached = plugin.get_processes();
        assert_eq!(cached.len(), 2);
        assert_eq!(cached[0].pid, 1234);
        assert_eq!(cached[0].name, "firefox");
        assert_eq!(cached[1].pid, 5678);
        assert_eq!(plugin.process_count(), 2);
    }

    #[test]
    fn test_create_stats_request() {
        let plugin = SystemMonitorPlugin::new();
        let packet = plugin.create_stats_request();

        assert_eq!(packet.packet_type, "cconnect.systemmonitor.request");
        assert_eq!(
            packet.body.get("requestType").and_then(|v| v.as_str()),
            Some("stats")
        );
    }

    #[test]
    fn test_create_processes_request() {
        let plugin = SystemMonitorPlugin::new();
        let packet = plugin.create_processes_request(25);

        assert_eq!(packet.packet_type, "cconnect.systemmonitor.request");
        assert_eq!(
            packet.body.get("requestType").and_then(|v| v.as_str()),
            Some("processes")
        );
        assert_eq!(packet.body.get("limit").and_then(|v| v.as_u64()), Some(25));
    }

    #[test]
    fn test_cpu_stats_serialization() {
        let cpu = CpuStats {
            usage: 45.5,
            cores: vec![40.0, 50.0],
        };

        let json = serde_json::to_value(&cpu).unwrap();
        assert_eq!(json["usage"], 45.5);
        assert_eq!(json["cores"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_memory_stats_serialization() {
        let memory = MemoryStats {
            total: 16_000_000_000,
            used: 8_000_000_000,
            available: 8_000_000_000,
            usage_percent: 50.0,
        };

        let json = serde_json::to_value(&memory).unwrap();
        assert_eq!(json["total"], 16_000_000_000u64);
        assert_eq!(json["usagePercent"], 50.0);
    }

    #[test]
    fn test_process_info_serialization() {
        let process = ProcessInfo {
            pid: 1234,
            name: "test".to_string(),
            cpu: 10.5,
            memory: 1_000_000,
        };

        let json = serde_json::to_value(&process).unwrap();
        assert_eq!(json["pid"], 1234);
        assert_eq!(json["name"], "test");
        assert_eq!(json["cpu"], 10.5);
        assert_eq!(json["memory"], 1_000_000);
    }
}
