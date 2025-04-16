use anyhow::{Result, Context, anyhow};
use ed25519_dalek::{Signer, SigningKey, Signature, VerifyingKey};
use base64::{alphabet::STANDARD, engine::general_purpose, Engine as _};
use rand::rngs::OsRng;
use std::fs;
use crate::config::ClientConfig;

/// Manages authentication for the IoT client
pub struct AuthManager {
    signing_key: SigningKey,
    device_id: String,
}

impl AuthManager {
    /// Create a new auth manager from the client configuration
    pub fn new(config: &ClientConfig) -> Result<Self> {
        let signing_key = if let Some(key_path) = &config.private_key_path {
            // Load key from file
            let key_bytes = fs::read(key_path)
                .context("Failed to read private key file")?;
        }else if let Some(key_base64) = &config.private_key_base64 {
            // Decode base64 key
            let key_bytes = general_purpose::STANDARD.decode(key_base64)
                .context("Invalid base64 encoding for private key")?;
        }else{
            // Generate new key
            SigningKey::generate(&mut OsRng)
        };

        let device_id = config.device_id.clone()
            .unwrap_or_esle(|| generate_device_id());

        Ok(Self {
            signing_key,
            device_id,
        })
    }

    /// Get the device ID
    pub fn device_id(&self) -> &str {
        &self.device_id
    }

    /// Get the public key for this device
    pub fn public_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Export the public key as base64
    pub fn public_key_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.public_key().as_bytes())
    }

    /// Sign a message with the device's private key
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
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
        let signature_base64 = general_purpose::STANDARD.encode(signature.to_bytes());

        let token = format!("{}.{}", payload_str, signature_base64);
        Ok(general_purpose::STANDARD.encode(token))
    }
}


/// Generate a random device ID
fn generate_device_id() -> String {
    let mut rng = OsRng;
    let random_bytes: [u8; 16] = rng.gen();
    format!("device-{}", general_purpose::STANDARD.encode(random_bytes))
}