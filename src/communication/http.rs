//! HTTP client for communicating with the IoT service API

use anyhow::{Result, Context};
use reqwest::{Client, header};
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;
use crate::auth::AuthManager;
use tracing::{debug, error};

/// HTTP Client for the IoT service API
pub struct HttpClient{
    client: Client,
    base_url: String,
    auth_manager:AuthManager,
}

impl HttpClient {
    /// Create a new HTTP Client
    pub fn new(base_url: &str, auth_manager: AuthManager, timeout_seconds: u64) -> Result<Self>{
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .context("Failed to create HTTP Client")?;

        Ok(Self {
            client,
            base_url: base_url.to_string(),
            auth_manager,
        })
    }

    /// Make an authenticated GET request
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let token = self.auth_manager.create_auth_token(300)?; // 5 minutes

        debug!("Making GET request to {}", url);

        let response = self.client.get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to send GET request")?;

        self.handle_response(response).await
    }

    /// Make an authenticated PUT request
    pub async fn put<T: DeserializeOwned, B: Serialize>(&self, path: &str, body:&B) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let token = self.auth_manager.create_auth_token(300)?; // 5 minute token

        debug!("Making a PUT request to {}", url);

        let response = self.client.put(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await
            .context("Failed to senf PUT request")?;

        self.handle_response(response).await
    }

    /// Make an authenticated DELETE request
    pub async fn delete<T:DeserializeOwned>(&self, path: &str) -> Result<T>{
        let url = format!("{}{}", self.base_url, path);
        let token = self.auth_manager.create_auth_token(300)?; // 5 minute token
        
        debug!("Making DELETE request to {}", url);
        
        let response = self.client.delete(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to send DELETE request")?;
            
        self.handle_response(response).await
    }

    /// Make an authenticated POST request
    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let token = self.auth_manager.create_auth_token(300)?; // 5 minute token

        debug!("Making POST request to {}", url);

        let response = self.client.post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(body)
            .send()
            .await
            .context("Failed to send POST request")?;

        self.handle_response(response).await
    }

    /// Handle API response
    async fn handle_response<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status();
        let url = response.url().to_string();
        if status.is_success() {
            let body = response.json::<T>().await
                .context("Failed to parse response JSON")?;
            Ok(body)
        } else {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unable to read error response".to_string());

            error!("API error ({}): {} - {}", status.as_u16(), url, error_text);

            Err(anyhow::anyhow!("API error {}:{}", status.as_u16(), error_text))
        }
    }
}