//! Communication modules for the IoT SDK

pub mod http;
pub mod websocket;

// Re-export important types
pub use http::HttpClient;
pub use websocket::{WebSocketConnection, WebSocketMessage, WebSocketMessageType};