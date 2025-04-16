//! Configuration for the IoT SDK client

use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::{Result, Context};

/// Configuration for the IoT client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Base URL for the API service
    pub api_url: String,
    
    /// Path to the private key file for authentication
    pub private_key_path: Option<String>,
    
    /// Private key as a base64 string (alternative to file)
    pub private_key_base64: Option<String>,
    
    /// Device identifier
    pub device_id: Option<String>,
    
    /// Timeout for API requests in seconds
    pub request_timeout: Option<u64>,
    
    /// WebSocket endpoint URL
    pub websocket_url: Option<String>,
}

impl ClientConfig {
    /// Create a new configuration with minimal required parameters
    pub fn new(api_url: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
            private_key_path: None,
            private_key_base64: None,
            device_id: None,
            request_timeout: Some(30),
            websocket_url: None,
        }
    }
    
    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_str = std::fs::read_to_string(path)
            .context("Failed to read config file")?;
            
        let config: ClientConfig = serde_json::from_str(&config_str)
            .context("Failed to parse config file")?;
            
        Ok(config)
    }
    
    /// Set the private key from a file path
    pub fn with_private_key_file<S: Into<String>>(mut self, path: S) -> Self {
        self.private_key_path = Some(path.into());
        self
    }
    
    /// Set the private key from a base64 encoded string
    pub fn with_private_key_base64<S: Into<String>>(mut self, key: S) -> Self {
        self.private_key_base64 = Some(key.into());
        self
    }
    
    /// Set the device ID
    pub fn with_device_id<S: Into<String>>(mut self, device_id: S) -> Self {
        self.device_id = Some(device_id.into());
        self
    }
    
    /// Set the websocket URL
    pub fn with_websocket_url<S: Into<String>>(mut self, url: S) -> Self {
        self.websocket_url = Some(url.into());
        self
    }
}