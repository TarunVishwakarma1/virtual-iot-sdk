//! Webhook management for real-time data updates

use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use crate::auth::AuthManager;
use crate::communication::http::HttpClient;