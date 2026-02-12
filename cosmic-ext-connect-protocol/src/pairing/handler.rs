//! CConnect Device Pairing
//!
//! This module implements TLS-based secure pairing between devices.
//! Devices must be paired before exchanging any functional packets.
//!
//! ## Pairing Protocol
//!
//! 1. **Certificate Generation**: Each device generates a self-signed certificate
//! 2. **Pairing Request**: Device A sends `cconnect.pair` with `pair: true`
//! 3. **User Verification**: Users verify SHA256 fingerprints on both devices
//! 4. **Pairing Response**: Device B responds with `pair: true` (accept) or `pair: false` (reject)
//! 5. **Certificate Storage**: Accepted certificates are stored for future connections
//!
//! ## Certificate Requirements
//!
//! - **Algorithm**: RSA 2048-bit
//! - **Organization (O)**: "KDE"
//! - **Organizational Unit (OU)**: "Kde connect"
//! - **Common Name (CN)**: Device UUID
//! - **Validity**: 10 years
//! - **Serial Number**: 10
//!
//! ## Security
//!
//! - Self-signed certificates exchanged on first pairing
//! - SHA256 fingerprint verification prevents MITM attacks
//! - Certificates stored and verified on subsequent connections
//! - Pairing timeout: 30 seconds
//!
//! ## References
//! - [Valent Protocol Reference](https://valent.andyholmes.ca/documentation/protocol.html)
//! - [CConnect TLS Implementation](https://invent.kde.org/network/cconnect-kde)

use crate::{Packet, ProtocolError, Result};
use cosmic_ext_connect_core::crypto::CertificateInfo;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Default pairing timeout (30 seconds per protocol specification)
pub const PAIRING_TIMEOUT: Duration = Duration::from_secs(30);

/// Pairing status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PairingStatus {
    /// Not paired
    Unpaired,
    /// Pairing request sent, awaiting response
    Requested,
    /// Pairing request received, awaiting user confirmation
    RequestedByPeer,
    /// Successfully paired
    Paired,
}

/// Pairing request/response packet
#[derive(Debug, Clone)]
pub struct PairingPacket {
    /// Whether pairing is requested (true) or rejected/unpaired (false)
    pub pair: bool,
}

impl PairingPacket {
    /// Create a pairing request packet
    pub fn request() -> Packet {
        // Include timestamp in pairing request (required by Android CConnect)
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Packet::new(
            "cconnect.pair",
            json!({
                "pair": true,
                "timestamp": timestamp
            }),
        )
    }

    /// Create a pairing accept response packet
    pub fn accept() -> Packet {
        // Include timestamp in pairing response (required by Android CConnect)
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Packet::new(
            "cconnect.pair",
            json!({
                "pair": true,
                "timestamp": timestamp
            }),
        )
    }

    /// Create a pairing reject response packet
    pub fn reject() -> Packet {
        Packet::new("cconnect.pair", json!({ "pair": false }))
    }

    /// Create an unpair packet
    pub fn unpair() -> Packet {
        Packet::new("cconnect.pair", json!({ "pair": false }))
    }

    /// Parse a pairing packet
    pub fn from_packet(packet: &Packet) -> Result<Self> {
        if !packet.is_type("cconnect.pair") {
            return Err(ProtocolError::InvalidPacket(
                "Not a pairing packet".to_string(),
            ));
        }

        let pair = packet
            .get_body_field::<bool>("pair")
            .ok_or_else(|| ProtocolError::InvalidPacket("Missing pair field".to_string()))?;

        Ok(Self { pair })
    }
}

/// Pairing handler for managing device pairing
pub struct PairingHandler {
    /// This device's certificate
    certificate: CertificateInfo,

    /// Pairing status
    status: PairingStatus,

    /// Paired device certificates (device_id -> certificate)
    paired_devices: std::collections::HashMap<String, Vec<u8>>,

    /// Certificate storage directory
    cert_dir: PathBuf,
}

impl PairingHandler {
    /// Create a new pairing handler
    ///
    /// # Arguments
    ///
    /// * `device_id` - This device's unique identifier
    /// * `cert_dir` - Directory to store certificates
    pub fn new(device_id: impl Into<String>, cert_dir: impl Into<PathBuf>) -> Result<Self> {
        let device_id = device_id.into();
        let cert_dir = cert_dir.into();

        // Ensure certificate directory exists
        fs::create_dir_all(&cert_dir)?;

        // Load or generate certificate
        let cert_path = cert_dir.join("device_cert.pem");
        let key_path = cert_dir.join("device_key.pem");

        let certificate = if cert_path.exists() && key_path.exists() {
            info!("Loading existing certificate for device {}", device_id);
            CertificateInfo::load_from_files(&cert_path, &key_path)?
        } else {
            info!("Generating new certificate for device {}", device_id);
            let cert = CertificateInfo::generate(&device_id)?;
            cert.save_to_files(&cert_path, &key_path)?;
            cert
        };

        Ok(Self {
            certificate,
            status: PairingStatus::Unpaired,
            paired_devices: std::collections::HashMap::new(),
            cert_dir,
        })
    }

    /// Get this device's certificate fingerprint
    pub fn fingerprint(&self) -> &str {
        &self.certificate.fingerprint
    }

    /// Get this device's certificate
    pub fn certificate(&self) -> &CertificateInfo {
        &self.certificate
    }

    /// Get current pairing status
    pub fn status(&self) -> PairingStatus {
        self.status
    }

    /// Send pairing request
    pub fn request_pairing(&mut self) -> Packet {
        self.status = PairingStatus::Requested;
        info!("Sending pairing request");
        PairingPacket::request()
    }

    /// Handle incoming pairing packet
    ///
    /// Returns (should_respond, response_packet)
    pub fn handle_pairing_packet(
        &mut self,
        packet: &Packet,
        device_id: &str,
        device_cert: &[u8],
    ) -> Result<(bool, Option<Packet>)> {
        debug!(
            "Received pairing packet from {} - body: {}",
            device_id, packet.body
        );

        let pairing = PairingPacket::from_packet(packet)?;

        debug!(
            "Processing pairing packet from {} - pair: {}",
            device_id, pairing.pair
        );

        if pairing.pair {
            // Pairing request or accept
            match self.status {
                PairingStatus::Unpaired => {
                    // Received pairing request
                    self.status = PairingStatus::RequestedByPeer;
                    info!("Received pairing request from device {}", device_id);
                    // Don't auto-accept, wait for user confirmation
                    Ok((false, None))
                }
                PairingStatus::Requested => {
                    // Received pairing accept - send confirmation response
                    self.store_device_certificate(device_id, device_cert)?;
                    self.status = PairingStatus::Paired;
                    info!(
                        "Pairing accepted by device {} - sending confirmation",
                        device_id
                    );
                    Ok((true, Some(PairingPacket::accept())))
                }
                PairingStatus::RequestedByPeer => {
                    // Already have a pending request from this device
                    warn!("Received duplicate pairing request from {}", device_id);
                    Ok((false, None))
                }
                PairingStatus::Paired => {
                    // Already paired - just acknowledge, don't respond (avoid pairing loop)
                    debug!(
                        "Received pairing request from already paired device {} - ignoring",
                        device_id
                    );
                    Ok((true, None))
                }
            }
        } else {
            // Pairing rejection or unpair
            if self.status == PairingStatus::Paired {
                self.remove_device_certificate(device_id)?;
                info!("Unpaired from device {}", device_id);
            } else {
                info!("Pairing rejected by device {}", device_id);
            }
            self.status = PairingStatus::Unpaired;
            Ok((false, None))
        }
    }

    /// Accept pairing request (user confirmed)
    pub fn accept_pairing(&mut self, device_id: &str, device_cert: &[u8]) -> Result<Packet> {
        if self.status != PairingStatus::RequestedByPeer {
            return Err(ProtocolError::InvalidPacket(
                "No pairing request pending".to_string(),
            ));
        }

        self.store_device_certificate(device_id, device_cert)?;
        self.status = PairingStatus::Paired;
        info!("Accepted pairing with device {}", device_id);

        Ok(PairingPacket::accept())
    }

    /// Reject pairing request (user declined)
    pub fn reject_pairing(&mut self) -> Packet {
        self.status = PairingStatus::Unpaired;
        info!("Rejected pairing request");
        PairingPacket::reject()
    }

    /// Unpair from a device
    pub fn unpair(&mut self, device_id: &str) -> Result<Packet> {
        self.remove_device_certificate(device_id)?;
        self.status = PairingStatus::Unpaired;
        info!("Unpairing from device {}", device_id);
        Ok(PairingPacket::unpair())
    }

    /// Check if a device is paired
    pub fn is_paired(&self, device_id: &str) -> bool {
        self.paired_devices.contains_key(device_id) || self.status == PairingStatus::Paired
    }

    /// Store device certificate
    fn store_device_certificate(&mut self, device_id: &str, cert_der: &[u8]) -> Result<()> {
        let cert_path = self.cert_dir.join(format!("{}.pem", device_id));
        let cert_pem = pem::encode(&pem::Pem::new("CERTIFICATE", cert_der.to_vec()));
        fs::write(&cert_path, cert_pem)?;

        self.paired_devices
            .insert(device_id.to_string(), cert_der.to_vec());
        debug!(
            "Stored certificate for device {} at {:?}",
            device_id, cert_path
        );

        Ok(())
    }

    /// Remove device certificate
    fn remove_device_certificate(&mut self, device_id: &str) -> Result<()> {
        let cert_path = self.cert_dir.join(format!("{}.pem", device_id));
        if cert_path.exists() {
            fs::remove_file(&cert_path)?;
        }

        self.paired_devices.remove(device_id);
        debug!("Removed certificate for device {}", device_id);

        Ok(())
    }

    /// Load all paired device certificates
    pub fn load_paired_devices(&mut self) -> Result<()> {
        for entry in fs::read_dir(&self.cert_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("pem") {
                let filename = path.file_stem().and_then(|s| s.to_str());
                if let Some(device_id) = filename {
                    // Skip our own certificate
                    if device_id == "device_cert" || device_id == "device_key" {
                        continue;
                    }

                    // Load certificate (PEM format) and extract DER
                    // Paired device certificates are stored as cert only, no private key needed
                    let cert_data = match fs::read(&path) {
                        Ok(data) => data,
                        Err(e) => {
                            warn!("Failed to read certificate file for {}: {}", device_id, e);
                            continue;
                        }
                    };

                    let cert_pem = match pem::parse(&cert_data) {
                        Ok(pem) => pem,
                        Err(e) => {
                            warn!("Failed to parse certificate PEM for {}: {}", device_id, e);
                            continue;
                        }
                    };

                    if cert_pem.tag() == "CERTIFICATE" {
                        self.paired_devices
                            .insert(device_id.to_string(), cert_pem.contents().to_vec());
                        debug!("Loaded paired device certificate: {}", device_id);
                    } else {
                        warn!(
                            "Invalid certificate tag for {}: expected CERTIFICATE, got {}",
                            device_id,
                            cert_pem.tag()
                        );
                    }
                }
            }
        }

        info!(
            "Loaded {} paired device certificates",
            self.paired_devices.len()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_certificate_generation() {
        let cert = CertificateInfo::generate("test_device_123").unwrap();

        assert_eq!(cert.device_id, "test_device_123");
        assert!(!cert.certificate.is_empty());
        assert!(!cert.private_key.is_empty());
        assert!(!cert.fingerprint.is_empty());

        // Fingerprint should be in format XX:XX:XX:...
        assert!(cert.fingerprint.contains(':'));
        assert_eq!(cert.fingerprint.len(), 95); // SHA256 is 32 bytes = 64 hex chars + 31 colons
    }

    #[test]
    fn test_certificate_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let cert_path = temp_dir.path().join("cert.pem");
        let key_path = temp_dir.path().join("key.pem");

        // Generate and save
        let original = CertificateInfo::generate("test_device").unwrap();
        original.save_to_files(&cert_path, &key_path).unwrap();

        assert!(cert_path.exists());
        assert!(key_path.exists());

        // Load and verify
        let loaded = CertificateInfo::load_from_files(&cert_path, &key_path).unwrap();
        assert_eq!(original.fingerprint, loaded.fingerprint);
    }

    #[test]
    fn test_pairing_packet_creation() {
        let request = PairingPacket::request();
        assert!(request.is_type("cconnect.pair"));
        assert_eq!(request.get_body_field::<bool>("pair"), Some(true));

        let accept = PairingPacket::accept();
        assert_eq!(accept.get_body_field::<bool>("pair"), Some(true));

        let reject = PairingPacket::reject();
        assert_eq!(reject.get_body_field::<bool>("pair"), Some(false));
    }

    #[test]
    fn test_pairing_packet_parsing() {
        let packet = PairingPacket::request();
        let parsed = PairingPacket::from_packet(&packet).unwrap();
        assert!(parsed.pair);

        let reject_packet = PairingPacket::reject();
        let parsed_reject = PairingPacket::from_packet(&reject_packet).unwrap();
        assert!(!parsed_reject.pair);
    }

    #[test]
    fn test_pairing_handler_creation() {
        let temp_dir = TempDir::new().unwrap();
        let handler = PairingHandler::new("test_device", temp_dir.path()).unwrap();

        assert_eq!(handler.status(), PairingStatus::Unpaired);
        assert!(!handler.fingerprint().is_empty());
    }

    #[test]
    fn test_pairing_request_flow() {
        let temp_dir = TempDir::new().unwrap();
        let mut handler = PairingHandler::new("test_device", temp_dir.path()).unwrap();

        // Send pairing request
        let request = handler.request_pairing();
        assert_eq!(handler.status(), PairingStatus::Requested);
        assert!(request.is_type("cconnect.pair"));
    }

    #[test]
    fn test_certificate_fingerprint() {
        let cert1 = CertificateInfo::generate("device1").unwrap();
        let cert2 = CertificateInfo::generate("device2").unwrap();

        // Different devices should have different fingerprints
        assert_ne!(cert1.fingerprint, cert2.fingerprint);

        // Same certificate should have same fingerprint
        let fp1 = CertificateInfo::calculate_fingerprint(&cert1.certificate);
        let fp2 = CertificateInfo::calculate_fingerprint(&cert1.certificate);
        assert_eq!(fp1, fp2);
    }

    #[test]
    fn test_fingerprint_format() {
        let cert = CertificateInfo::generate("test").unwrap();
        let parts: Vec<&str> = cert.fingerprint.split(':').collect();

        // SHA256 produces 32 bytes = 32 parts when split by colons
        assert_eq!(parts.len(), 32);

        // Each part should be 2 hex digits
        for part in parts {
            assert_eq!(part.len(), 2);
            assert!(part.chars().all(|c| c.is_ascii_hexdigit()));
        }
    }
}
