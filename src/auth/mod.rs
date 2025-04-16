use anyhow::{Result, Context};
use ring::signature::{self, Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519};
use base64::{engine::general_purpose, Engine as _};
use rand::{rngs::OsRng, RngCore, TryRngCore};
use std::fs;
use crate::config::ClientConfig;

/// Manages authentication for the IoT client
pub struct AuthManager {
    key_pair: Ed25519KeyPair,
    device_id: String,
}

impl AuthManager {
    /// Create a new auth manager from the client configuration
    pub fn new(config: &ClientConfig) -> Result<Self> {
        let key_pair = if let Some(key_path) = &config.private_key_path {
            // Load key from file
            let key_bytes = fs::read(key_path)
                .context("Failed to read private key file")?;
            Ed25519KeyPair::from_pkcs8(&key_bytes)
                .map_err(|_| anyhow::anyhow!("Invalid key format"))?
        } else if let Some(key_base64) = &config.private_key_base64 {
            // Decode base64 key
            let key_bytes = general_purpose::STANDARD.decode(key_base64)
                .context("Invalid base64 encoding for private key")?;
            Ed25519KeyPair::from_pkcs8(&key_bytes)
                .map_err(|_| anyhow::anyhow!("Invalid key format"))?
        } else {
            // Generate new key
            let rng = ring::rand::SystemRandom::new();
            let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)
                .map_err(|_| anyhow::anyhow!("Failed to generate key pair"))?;
            Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
                .map_err(|_| anyhow::anyhow!("Failed to parse generated key"))?
        };

        let device_id = config.device_id.clone()
            .unwrap_or_else(|| generate_device_id());

        Ok(Self {
            key_pair,
            device_id,
        })
    }

    /// Get the device ID
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// Get the public key for this device
    pub fn public_key(&self) -> &[u8] {
        self.key_pair.public_key().as_ref()
    }

    /// Export the public key as base64
    pub fn public_key_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.public_key())
    }

    /// Sign a message with the device's private key
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.key_pair.sign(message).as_ref().to_vec()
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<()> {
        let public_key = UnparsedPublicKey::new(&ED25519, self.public_key());
        public_key.verify(message, signature)
            .map_err(|_| anyhow::anyhow!("Signature verification failed"))
    }

    /// Create an authentication token for API requests
    pub fn create_auth_token(&self, expiration_seconds: u64) -> Result<String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .context("Failed to get current time")?
            .as_secs();

        let expiration = now + expiration_seconds;

        let payload = serde_json::json!({
            "device_id": self.device_id(),
            "exp": expiration,
            "iat": now,
        });

        let payload_str = serde_json::to_string(&payload)?;
        let signature = self.sign(payload_str.as_bytes());
        let signature_base64 = general_purpose::STANDARD.encode(signature);

        let token = format!("{}.{}", payload_str, signature_base64);
        Ok(general_purpose::STANDARD.encode(token))
    }
}

/// Generate a random device ID
fn generate_device_id() -> String {
    let mut rng = OsRng{};
    let mut bytes = [0u8; 32];
    rng.try_fill_bytes(&mut bytes).expect("Failed to generate random bytes");
    hex::encode(bytes)
}