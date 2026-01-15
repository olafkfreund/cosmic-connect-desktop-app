//! Discovery Event System
//!
//! This module defines events emitted by the discovery service.

use crate::DeviceInfo;
use std::net::SocketAddr;

/// Events emitted by the discovery service
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    /// A new device was discovered on the network
    DeviceDiscovered {
        /// Information about the discovered device
        info: DeviceInfo,
        /// Network address where the device was discovered
        address: SocketAddr,
    },

    /// An existing device sent an updated identity packet
    DeviceUpdated {
        /// Updated device information
        info: DeviceInfo,
        /// Network address of the device
        address: SocketAddr,
    },

    /// A device has timed out (not seen for configured duration)
    DeviceTimeout {
        /// ID of the device that timed out
        device_id: String,
    },

    /// Discovery service started successfully
    ServiceStarted {
        /// Port the discovery service is listening on
        port: u16,
    },

    /// Discovery service stopped
    ServiceStopped,

    /// An error occurred during discovery
    Error {
        /// Error message
        message: String,
    },
}

impl DiscoveryEvent {
    /// Check if this is a device discovered event
    pub fn is_device_discovered(&self) -> bool {
        matches!(self, DiscoveryEvent::DeviceDiscovered { .. })
    }

    /// Check if this is a device updated event
    pub fn is_device_updated(&self) -> bool {
        matches!(self, DiscoveryEvent::DeviceUpdated { .. })
    }

    /// Check if this is a device timeout event
    pub fn is_device_timeout(&self) -> bool {
        matches!(self, DiscoveryEvent::DeviceTimeout { .. })
    }

    /// Get device ID if this event is device-related
    pub fn device_id(&self) -> Option<&str> {
        match self {
            DiscoveryEvent::DeviceDiscovered { info, .. } => Some(&info.device_id),
            DiscoveryEvent::DeviceUpdated { info, .. } => Some(&info.device_id),
            DiscoveryEvent::DeviceTimeout { device_id } => Some(device_id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DeviceType;

    #[test]
    fn test_event_type_checking() {
        let info = DeviceInfo::new("Test", DeviceType::Desktop, 1716);
        let addr = "192.168.1.100:1716".parse().unwrap();

        let discovered = DiscoveryEvent::DeviceDiscovered {
            info: info.clone(),
            address: addr,
        };
        assert!(discovered.is_device_discovered());
        assert!(!discovered.is_device_timeout());

        let timeout = DiscoveryEvent::DeviceTimeout {
            device_id: "test_id".to_string(),
        };
        assert!(timeout.is_device_timeout());
        assert!(!timeout.is_device_discovered());
    }

    #[test]
    fn test_device_id_extraction() {
        let info = DeviceInfo::with_id("test_123", "Test", DeviceType::Desktop, 1716);
        let addr = "192.168.1.100:1716".parse().unwrap();

        let discovered = DiscoveryEvent::DeviceDiscovered {
            info: info.clone(),
            address: addr,
        };
        assert_eq!(discovered.device_id(), Some("test_123"));

        let timeout = DiscoveryEvent::DeviceTimeout {
            device_id: "timeout_id".to_string(),
        };
        assert_eq!(timeout.device_id(), Some("timeout_id"));

        let started = DiscoveryEvent::ServiceStarted { port: 1716 };
        assert_eq!(started.device_id(), None);
    }
}
