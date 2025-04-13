//! Data models for IoT device communication

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about an IoT device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo{
    /// Type of device
    pub device_type: String,

    /// Name of the device
    pub name: String,

    /// Version of the device firmware
    pub firmware_version: String,

    /// Additional metadata as key-value pairs
    pub metadata: HashMap<String, String>, 
}


/// Status of a device
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceStatus {
    #[serde(rename = "online")]
    Online,

    #[serde(rename = "offline")]
    Offline,

    #[serde(rename = "maintenance")]
    Maintenance,

    #[serde(rename = "erorr")]
    Error,
}


/// Data reported by the device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceData {
    /// Timestand when the data was collected
    pub timestamp: u64,

    /// Current status of the device
    pub status : DeviceStatus,

    /// Sensor readings as key-value pairs
    pub readings: HashMap<String, serde_json::Value>,

    /// Alert level if applicable
    pub alert_level: Option<AlertLevel>,
}

/// Alert levels for device notifications
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertLevel {
    #[serde(rename = "info")]
    Info,
    
    #[serde(rename = "warning")]
    Warning,
    
    #[serde(rename = "error")]
    Error,
    
    #[serde(rename = "critical")]
    Critical,
}

impl DeviceData {
    /// Create a new data point with the current timestamp
    pub fn new(status :DeviceStatus) -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status, 
            readings: HashMap::new(),
            alert_level: None,
        }
    }

    /// Add a sensor reading
    pub fn add_reading<T: Serialize>(mut self, name: &str, value: T) -> anyhow::Result<Self> {
        let value = serde_json::to_value(value)?;
        self.readings.insert(name.to_string(), value);
        Ok(self)
    }

    /// Set the alert level
    pub fn with_alert_level(mut self, level: AlertLevel) -> Self {
        self.alert_level = Some(level);
        self
    }
}