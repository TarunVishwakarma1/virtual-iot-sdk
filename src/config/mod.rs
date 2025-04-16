use std::path::PathBuf;

/// Configuration for the IoT client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Path to the private key file
    pub private_key_path: Option<PathBuf>,
    
    /// Base64 encoded private key
    pub private_key_base64: Option<String>,
    
    /// Device ID
    pub device_id: Option<String>,
}
