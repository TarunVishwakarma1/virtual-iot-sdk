//! Webhook management for real-time data updates

use anyhow::Result;
use serde::{Serialize, Deserialize};
use crate::communication::http::HttpClient;
use tracing::info;

/// A registered webhook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    // Unique ID for the webhook
    pub id: String,
    
    /// URL to call when events occur
    pub url: String,
    
    /// Device ID this webhook is for
    pub device_id: String,
    
    /// Secret used to sign webhook payloads
    pub secret: String,
    
    /// Events this webhook should receive
    pub events: Vec<WebhookEventType>,
}

/// Manager for handling webhooks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    /// Device data updates
    DataUpdate,
    
    /// Device status changes
    StatusChange,
    
    /// Device alerts
    Alert,
    
    /// Device configuration changes
    ConfigChange,
}

/// Manager for handling webhooks
pub struct WebhookManager {
    http_client: HttpClient,
}

impl WebhookManager {
    /// Create a new webhook manager
    pub fn new(http_client: HttpClient) -> Self {
        Self { 
            http_client, 
        }
    }

    /// Register a new webhook
    pub async fn register_webhook(&self, url: &str,  device_id: &str, events: Vec<WebhookEventType>) -> Result<Webhook> {
        let payload = serde_json::json!({
            "url": url,
            "device_id": device_id,
            "events": events,
        });
        
        info!("Registering webhook for device {} at URL {}", device_id, url);
        
        let webhook: Webhook = self.http_client.post("/webhooks", &payload).await?;
        
        info!("Successfully registered webhook with ID {}", webhook.id);
        
        Ok(webhook)
    }

    /// List all webhooks for a device
    pub async fn list_webhooks(&self, device_id: &str) -> Result<Vec<Webhook>> {
        let path = format!("/webhooks?device_id={}", device_id);
        let webhooks: Vec<Webhook> = self.http_client.get(&path).await?;
        
        Ok(webhooks)
    }

    /// Delete a webhook
    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<()> {
        let path = format!("/webhooks/{}", webhook_id);
        let _: serde_json::Value = self.http_client.delete(&path).await?;
        
        info!("Successfully deleted webhook {}", webhook_id);
        
        Ok(())
    }

    /// Test a webhook by sending a test event
    pub async fn test_webhook(&self, webhook_id: &str) -> Result<()> {
        let path = format!("/webhooks/{}/test", webhook_id);
        let _: serde_json::Value = self.http_client.post(&path, &serde_json::json!({})).await?;
        
        info!("Successfully sent test event to webhook {}", webhook_id);
        
        Ok(())
    }

    /// Generate a signature for webhook payload validation
    pub fn generate_signature(secret: &str, payload: &[u8]) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        type HmacSha256 = Hmac<Sha256>;
        
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
            
        mac.update(payload);
        
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        
        hex::encode(code_bytes)
    }
}