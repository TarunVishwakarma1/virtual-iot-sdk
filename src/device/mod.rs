//! Device Management ad metadata handling
use anyhow::Result;
use serde::{Serialize, Deserialize};
use crate::models::{DeviceInfo, DeviceStatus};
use crate::communication::http::HttpClient;
use std::collections::HashMap;
use tracing::info;

/// Device manager for handling IoT Devices
pub struct DeviceManager {
    http_client: HttpClient,
}

/// Device registration response
#[derive(Debug, Serialize, Deserialize)]
pub struct  DeviceRegistrationResponse {
    pub device_id: String,
    pub status: String,
    pub api_key: Option<String>,
}

/// Device update request
#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceUpdateRequest {
    pub name: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub firmware_version: Option<String>,
}

impl DeviceManager {
    /// Create a new device manager
    pub fn new(http_client: HttpClient) -> Self {
        Self {http_client}
    }

    /// Register a new deivce
    pub async fn register_device(&self, device_id: &str, info: &DeviceInfo) -> Result<DeviceRegistrationResponse> {
        let path = "/devices";
        let payload = serde_json::json!({
            "device_id": device_id,
            "device_type": info.device_type,
            "name": info.name,
            "firmware_version": info.firmware_version,
            "metadata": info.metadata,
        });

        info!("Registering device {}", device_id);
        let response: DeviceRegistrationResponse = self.http_client.post(path, &payload).await?;
        info!("Device registered successfully: {}", device_id);

        Ok(response)
    }

    /// Update device information
    pub async fn update_device(&self, device_id: &str, update: DeviceUpdateRequest) -> Result<DeviceInfo> {
        let path = format!("/devices/{}", device_id);
        let device: DeviceInfo = self.http_client.put(&path, &update).await?;

        info!("Device {} updated successfully", device_id);

        Ok(device)
    }

    /// Send device data
    pub async fn send_data(&self, device_id: &str, status: &DeviceStatus) -> Result<()> {
        let path = format!("/devices/{}/status", device_id);
        let payload = serde_json::json!({
            "status": status,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });

        let _: serde_json::Value = self.http_client.put(&path, &payload).await?;
        info!("Status updated for device {}: {:?}",device_id, status);

        Ok(())
    }

    /// List devices
    pub async fn list_devices(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<DeviceInfo>> {
        let mut path = "/devices?".to_string();

        if let Some(limit) = limit {
            path.push_str(&format!("limit={}&", limit));
        }

        if let Some(offset) = offset {
            path.push_str(&format!("offset={}", offset));
        }

        let devices: Vec<DeviceInfo> = self.http_client.get(&path).await?;

        Ok(devices)
    }
}

