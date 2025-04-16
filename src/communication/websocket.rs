//! WebSocket communication for real-time updates

use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use anyhow::{Result, Context, anyhow};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{info, error, debug};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use tokio::time::sleep;
use base64::{engine::general_purpose, Engine as _};
use rand::{Rng, thread_rng};

/// A WebSocket connection to the IoT service
pub struct WebSocketConnection {
    send_tx: Option<mpsc::Sender<String>>,
    connected: Arc<Mutex<bool>>,
}

/// Message type for WebSocket communication
#[derive(Debug, Serialize, Deserialize)]
pub enum WebSocketMessageType {
    #[serde(rename = "data")]
    Data,
    
    #[serde(rename = "status")]
    Status,
    
    #[serde(rename = "command")]
    Command,
    
    #[serde(rename = "ack")]
    Acknowledgement,
}

/// Message structure for WebSocket communication
#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketMessage {
    #[serde(rename = "type")]
    pub message_type: WebSocketMessageType,
    
    pub device_id: String,
    
    pub payload: serde_json::Value,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    
    pub timestamp: u64,
}

impl WebSocketConnection {
    /// Create a new WebSocket connection (not connected yet)
    pub fn new() -> Self {
        Self {
            send_tx: None,
            connected: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Connect to the WebSocket server
    pub async fn connect(&mut self, url: &str, auth_token: &str, device_id: &str) -> Result<()> {
        let full_url = format!("{}?token={}&device_id={}", url, auth_token, device_id);
        let (ws_stream, _) = connect_async(&full_url)
            .await
            .context("Failed to connect to WebSocket server")?;
        
        info!("WebSocket connected to {}", url);
        
        let (mut write, mut read) = ws_stream.split();
        
        // Channel for sending messages to the WebSocket
        let (tx, mut rx) = mpsc::channel::<String>(100);
        self.send_tx = Some(tx.clone());
        
        // Set connected state
        let connected = self.connected.clone();
        *connected.lock().unwrap() = true;
        
        // Task for sending messages
        let connected_clone = connected.clone();
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = write.send(Message::Text(message.into())).await {
                    error!("Error sending WebSocket message: {}", e);
                    *connected_clone.lock().unwrap() = false;
                    break;
                }
            }
            debug!("WebSocket sender task ended");
        });
        
        // Task for receiving messages
        let device_id = device_id.to_string();
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(msg) => {
                        if let Message::Text(text) = msg {
                            debug!("Received message: {}", text);
                            
                            if let Ok(parsed) = serde_json::from_str::<WebSocketMessage>(&text) {
                                match parsed.message_type {
                                    WebSocketMessageType::Command => {
                                        info!("Received command: {}", text);
                                        
                                        if let Some(id) = parsed.id {
                                            let ack = WebSocketMessage {
                                                message_type: WebSocketMessageType::Acknowledgement,
                                                device_id: device_id.clone(),
                                                payload: serde_json::json!({ "status": "received" }),
                                                id: Some(id),
                                                timestamp: std::time::SystemTime::now()
                                                    .duration_since(std::time::UNIX_EPOCH)
                                                    .unwrap_or_default()
                                                    .as_secs(),
                                            };
                                            
                                            if let Ok(ack_json) = serde_json::to_string(&ack) {
                                                if let Err(e) = tx_clone.send(ack_json).await {
                                                    error!("Failed to send acknowledgement: {}", e);
                                                }
                                            }
                                        }
                                    },
                                    _ => {
                                        debug!("Received message of type: {:?}", parsed.message_type);
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => {
                        error!("Error receiving WebSocket message: {}", e);
                        *connected.lock().unwrap() = false;
                        break;
                    }
                }
            }
            debug!("WebSocket receiver task ended");
            *connected.lock().unwrap() = false;
        });
        
        Ok(())
    }
    
    /// Send a data message over the WebSocket
    pub async fn send_data<T: Serialize>(&self, device_id: &str, payload: T) -> Result<()> {
        let message = WebSocketMessage {
            message_type: WebSocketMessageType::Data,
            device_id: device_id.to_string(),
            payload: serde_json::to_value(payload)?,
            id: Some(generate_message_id()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        self.send_message(message).await
    }
    
    /// Send a status update over the WebSocket
    pub async fn send_status<T: Serialize>(&self, device_id: &str, status: T) -> Result<()> {
        let message = WebSocketMessage {
            message_type: WebSocketMessageType::Status,
            device_id: device_id.to_string(),
            payload: serde_json::to_value(status)?,
            id: Some(generate_message_id()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        
        self.send_message(message).await
    }
    
    /// Send a formatted message over the WebSocket
    async fn send_message(&self, message: WebSocketMessage) -> Result<()> {
        if !self.is_connected() {
            return Err(anyhow!("WebSocket is not connected"));
        }
        
        let json = serde_json::to_string(&message)?;
        
        if let Some(tx) = &self.send_tx {
            tx.send(json).await
                .context("Failed to send message to WebSocket task")?;
            Ok(())
        } else {
            Err(anyhow!("WebSocket sender channel not initialized"))
        }
    }
    
    /// Check if the WebSocket is connected
    pub fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }
    
    /// Reconnect with exponential backoff
    pub async fn reconnect_with_backoff(&mut self, url: &str, auth_token: &str, device_id: &str, 
                                        max_attempts: usize) -> Result<()> {
        let mut attempts = 0;
        let mut delay = Duration::from_secs(1);
        
        while attempts < max_attempts {
            match self.connect(url, auth_token, device_id).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    attempts += 1;
                    error!("Reconnection attempt {} failed: {}", attempts, e);
                    
                    if attempts >= max_attempts {
                        return Err(anyhow!("Failed to reconnect after {} attempts", max_attempts));
                    }
                    
                    sleep(delay).await;
                    delay = delay.saturating_mul(2); // Exponential backoff
                }
            }
        }
        
        Err(anyhow!("Failed to reconnect"))
    }
}

/// Generate a random message ID
fn generate_message_id() -> String {
    let mut rng = thread_rng();
    let random_bytes: [u8; 8] = rng.gen();
    base64::engine::general_purpose::STANDARD.encode(random_bytes)
}