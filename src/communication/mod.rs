//! Webhook management for real-time data updates

use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use crate::auth::AuthManager;

pub mod http;
pub mod websocket;

pub use http::HttpClient;