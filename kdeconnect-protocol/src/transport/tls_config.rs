//! TLS Configuration for KDE Connect
//!
//! This module provides TLS server and client configuration for secure
//! communication between paired devices using mutual TLS authentication.

use crate::{CertificateInfo, ProtocolError, Result};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::{ClientConfig, DigitallySignedStruct, DistinguishedName, ServerConfig, SignatureScheme};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, warn};

/// Custom server certificate verifier for KDE Connect
///
/// Validates client certificates against the list of paired devices.
/// Only accepts connections from devices we have paired with.
#[derive(Debug)]
struct KdeConnectServerCertVerifier {
    /// Set of trusted certificate DER bytes (from paired devices)
    trusted_certs: HashSet<Vec<u8>>,
}

impl KdeConnectServerCertVerifier {
    fn new(trusted_certs: Vec<Vec<u8>>) -> Self {
        Self {
            trusted_certs: trusted_certs.into_iter().collect(),
        }
    }
}

impl ClientCertVerifier for KdeConnectServerCertVerifier {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        // We don't use root certificates, each device has self-signed cert
        &[]
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _now: UnixTime,
    ) -> std::result::Result<ClientCertVerified, rustls::Error> {
        // Check if this certificate is in our trusted list
        if self.trusted_certs.contains(end_entity.as_ref()) {
            debug!("Client certificate verified successfully");
            Ok(ClientCertVerified::assertion())
        } else {
            warn!("Client certificate not in trusted list");
            Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::UnknownIssuer,
            ))
        }
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        // Accept TLS 1.2 signatures (KDE Connect protocol supports TLS 1.2+)
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        // Accept TLS 1.3 signatures
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        // Support common signature schemes
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
        ]
    }
}

/// Custom client certificate verifier for KDE Connect
///
/// Validates server certificates against the specific paired device's certificate.
#[derive(Debug)]
struct KdeConnectClientCertVerifier {
    /// Expected server certificate DER bytes (from pairing)
    expected_cert: Vec<u8>,
}

impl KdeConnectClientCertVerifier {
    fn new(expected_cert: Vec<u8>) -> Self {
        Self { expected_cert }
    }
}

impl ServerCertVerifier for KdeConnectClientCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, rustls::Error> {
        // Check if this is the expected certificate
        if end_entity.as_ref() == self.expected_cert.as_slice() {
            debug!("Server certificate verified successfully");
            Ok(ServerCertVerified::assertion())
        } else {
            warn!("Server certificate does not match expected certificate");
            Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::UnknownIssuer,
            ))
        }
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
        ]
    }
}

/// Create a TLS server configuration for accepting connections from paired devices
///
/// # Arguments
///
/// * `our_cert` - Our device certificate information
/// * `trusted_device_certs` - Certificates of all paired devices
///
/// # Returns
///
/// Configured ServerConfig that validates client certificates against paired devices
pub fn create_server_config(
    our_cert: &CertificateInfo,
    trusted_device_certs: Vec<Vec<u8>>,
) -> Result<ServerConfig> {
    debug!(
        "Creating TLS server config with {} trusted devices",
        trusted_device_certs.len()
    );

    // Convert our certificate to rustls format
    let cert_der = CertificateDer::from(our_cert.certificate.clone());
    let key_der = PrivateKeyDer::try_from(our_cert.private_key.clone())
        .map_err(|_| ProtocolError::CertificateValidation("Invalid private key format".to_string()))?;

    // Create custom client cert verifier
    let client_verifier = Arc::new(KdeConnectServerCertVerifier::new(trusted_device_certs));

    // Build server config
    let config = ServerConfig::builder()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(vec![cert_der], key_der)
        .map_err(|e| ProtocolError::CertificateValidation(format!("Failed to create server config: {}", e)))?;

    debug!("TLS server config created successfully");
    Ok(config)
}

/// Create a TLS client configuration for connecting to a specific paired device
///
/// # Arguments
///
/// * `our_cert` - Our device certificate information
/// * `peer_cert` - The paired device's certificate (for validation)
///
/// # Returns
///
/// Configured ClientConfig that validates the server certificate matches the paired device
pub fn create_client_config(our_cert: &CertificateInfo, peer_cert: Vec<u8>) -> Result<ClientConfig> {
    debug!("Creating TLS client config for specific peer");

    // Convert our certificate to rustls format
    let cert_der = CertificateDer::from(our_cert.certificate.clone());
    let key_der = PrivateKeyDer::try_from(our_cert.private_key.clone())
        .map_err(|_| ProtocolError::CertificateValidation("Invalid private key format".to_string()))?;

    // Create custom server cert verifier
    let server_verifier = Arc::new(KdeConnectClientCertVerifier::new(peer_cert));

    // Build client config with dangerous (custom) verifier
    let config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(server_verifier)
        .with_client_auth_cert(vec![cert_der], key_der)
        .map_err(|e| ProtocolError::CertificateValidation(format!("Failed to create client config: {}", e)))?;

    debug!("TLS client config created successfully");
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_server_config() {
        // Generate test certificate
        let cert = CertificateInfo::generate("test_device").unwrap();

        // Create server config with no trusted devices
        let config = create_server_config(&cert, vec![]);
        assert!(config.is_ok());

        // Create server config with one trusted device
        let trusted_certs = vec![cert.certificate.clone()];
        let config = create_server_config(&cert, trusted_certs);
        assert!(config.is_ok());
    }

    #[test]
    fn test_create_client_config() {
        // Generate test certificates
        let our_cert = CertificateInfo::generate("device1").unwrap();
        let peer_cert = CertificateInfo::generate("device2").unwrap();

        // Create client config
        let config = create_client_config(&our_cert, peer_cert.certificate.clone());
        assert!(config.is_ok());
    }
}
