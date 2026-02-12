//! Authentication module for phone challenge-response protocol.
//!
//! This module implements Ed25519-based challenge-response authentication
//! for verifying phone devices connecting to the desktop.
//!
//! # Overview
//!
//! The authentication flow works as follows:
//!
//! 1. Desktop generates a challenge using [`ChallengeManager::generate_challenge`]
//! 2. Challenge is sent to the phone (via existing pairing channel)
//! 3. Phone signs the challenge with its Ed25519 private key
//! 4. Phone sends back a [`ChallengeResponse`] with the signature
//! 5. Desktop verifies the signature using [`Verifier::verify_response`]
//!
//! # Security Features
//!
//! - **32-byte challenges**: 256-bit cryptographic strength
//! - **16-byte nonces**: Unique identifier for replay prevention
//! - **30-second expiry**: Limits the window for attacks
//! - **Nonce tracking**: Prevents replay attacks
//! - **DoS protection**: Bounded active challenge storage
//! - **Constant-time verification**: Prevents timing attacks
//!
//! # Example
//!
//! ```rust,no_run
//! use cosmic_ext_connect_protocol::auth::{ChallengeManager, Verifier, ChallengeResponse};
//!
//! // Desktop: Generate a challenge
//! let manager = ChallengeManager::new("my-desktop-id".to_string());
//! let challenge = manager.generate_challenge().unwrap();
//!
//! // ... send challenge to phone, phone signs it ...
//!
//! // Desktop: Verify the response
//! # let phone_public_key = vec![0u8; 32];
//! # let signature = "".to_string();
//! let verifier = Verifier::new(phone_public_key).unwrap();
//!
//! // Consume the challenge (prevents replay)
//! let challenge = manager.get_and_consume_challenge(&challenge.nonce).unwrap();
//!
//! // Verify the signature
//! let response = ChallengeResponse {
//!     nonce: challenge.nonce.clone(),
//!     signature,
//!     phone_id: "phone-123".to_string(),
//! };
//!
//! verifier.verify_response(&challenge, &response).unwrap();
//! ```

mod challenge;
mod types;
mod verify;

pub use challenge::ChallengeManager;
pub use types::{
    AuthError, Challenge, ChallengeResponse, CHALLENGE_EXPIRY_SECS, CHALLENGE_SIZE,
    MAX_ACTIVE_CHALLENGES, NONCE_SIZE,
};
pub use verify::{Verifier, ED25519_PUBLIC_KEY_SIZE, ED25519_SIGNATURE_SIZE};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use base64::Engine;
    use ring::rand::SystemRandom;
    use ring::signature::{Ed25519KeyPair, KeyPair};

    /// Full integration test simulating the complete auth flow.
    #[test]
    fn test_full_authentication_flow() {
        // Setup: Phone has a keypair, desktop has the phone's public key
        let rng = SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let phone_keypair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap();
        let phone_public_key = phone_keypair.public_key().as_ref().to_vec();

        // Desktop: Create challenge manager and verifier
        let challenge_manager = ChallengeManager::new("test-desktop".to_string());
        let verifier = Verifier::new(phone_public_key).unwrap();

        // Desktop: Generate challenge
        let challenge = challenge_manager.generate_challenge().unwrap();
        assert_eq!(challenge.desktop_id, "test-desktop");

        // Phone: Sign the challenge
        let message = challenge.signing_message();
        let signature = phone_keypair.sign(&message);
        let signature_b64 = base64::engine::general_purpose::STANDARD.encode(signature.as_ref());

        let response = ChallengeResponse {
            nonce: challenge.nonce.clone(),
            signature: signature_b64,
            phone_id: "test-phone".to_string(),
        };

        // Desktop: Consume and verify
        let consumed_challenge = challenge_manager
            .get_and_consume_challenge(&response.nonce)
            .unwrap();

        let result = verifier.verify_response(&consumed_challenge, &response);
        assert!(result.is_ok(), "Authentication should succeed");
    }

    /// Test that replay attacks are prevented.
    #[test]
    fn test_replay_attack_prevention() {
        let rng = SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let phone_keypair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap();
        let phone_public_key = phone_keypair.public_key().as_ref().to_vec();

        let challenge_manager = ChallengeManager::new("test-desktop".to_string());
        let verifier = Verifier::new(phone_public_key).unwrap();

        // Generate and sign a challenge
        let challenge = challenge_manager.generate_challenge().unwrap();
        let message = challenge.signing_message();
        let signature = phone_keypair.sign(&message);
        let signature_b64 = base64::engine::general_purpose::STANDARD.encode(signature.as_ref());

        let response = ChallengeResponse {
            nonce: challenge.nonce.clone(),
            signature: signature_b64,
            phone_id: "test-phone".to_string(),
        };

        // First verification succeeds
        let consumed = challenge_manager
            .get_and_consume_challenge(&response.nonce)
            .unwrap();
        assert!(verifier.verify_response(&consumed, &response).is_ok());

        // Replay attack: try to use the same response again
        let replay_result = challenge_manager.get_and_consume_challenge(&response.nonce);
        assert!(
            matches!(replay_result, Err(AuthError::NonceReuse)),
            "Replay attack should be detected"
        );
    }

    /// Test that a malicious phone with wrong keys is rejected.
    #[test]
    fn test_impersonation_attack_prevention() {
        let rng = SystemRandom::new();

        // Legitimate phone keypair (public key is stored on desktop)
        let legit_pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let legit_keypair = Ed25519KeyPair::from_pkcs8(legit_pkcs8.as_ref()).unwrap();
        let legit_public_key = legit_keypair.public_key().as_ref().to_vec();

        // Attacker's keypair
        let attacker_pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let attacker_keypair = Ed25519KeyPair::from_pkcs8(attacker_pkcs8.as_ref()).unwrap();

        let challenge_manager = ChallengeManager::new("test-desktop".to_string());
        // Desktop only trusts the legitimate phone's public key
        let verifier = Verifier::new(legit_public_key).unwrap();

        let challenge = challenge_manager.generate_challenge().unwrap();

        // Attacker signs with their own key
        let message = challenge.signing_message();
        let attacker_signature = attacker_keypair.sign(&message);
        let signature_b64 =
            base64::engine::general_purpose::STANDARD.encode(attacker_signature.as_ref());

        let response = ChallengeResponse {
            nonce: challenge.nonce.clone(),
            signature: signature_b64,
            phone_id: "attacker-phone".to_string(),
        };

        let consumed = challenge_manager
            .get_and_consume_challenge(&response.nonce)
            .unwrap();

        // Verification should fail - attacker's signature doesn't match
        let result = verifier.verify_response(&consumed, &response);
        assert!(
            matches!(result, Err(AuthError::InvalidSignature)),
            "Impersonation attack should be detected"
        );
    }
}
