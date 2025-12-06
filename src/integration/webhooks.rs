use crate::compliance_models::WebhookEvent;
use reqwest::Client;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::engine::general_purpose;
use base64::Engine;
use std::time::Duration;
use tokio::time::sleep;
use serde_json;

/// Webhook service for async delivery with retry logic
pub struct WebhookService {
    client: Client,
}

impl WebhookService {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }

    /// Generate HMAC-SHA256 signature for webhook payload
    pub fn generate_signature(payload: &str, secret: &str) -> String {
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        general_purpose::STANDARD.encode(&code_bytes)
    }

    /// Deliver webhook event asynchronously
    pub async fn deliver_webhook(
        &self,
        endpoint_url: &str,
        event: &WebhookEvent,
        secret_key: &str,
        timeout_seconds: u64,
    ) -> Result<(u16, String), String> {
        // Generate signature
        let payload_json = serde_json::to_string(event)
            .map_err(|e| format!("Failed to serialize event: {}", e))?;
        
        let signature = Self::generate_signature(&payload_json, secret_key);
        
        // Create signed event
        let mut signed_event = event.clone();
        signed_event.signature = Some(format!("sha256={}", signature));

        let signed_payload = serde_json::to_string(&signed_event)
            .map_err(|e| format!("Failed to serialize signed event: {}", e))?;

        // Send HTTP POST request
        let response = self.client
            .post(endpoint_url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", format!("sha256={}", signature))
            .header("X-Webhook-Event-Type", &event.event_type)
            .timeout(Duration::from_secs(timeout_seconds))
            .body(signed_payload)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();

        if status >= 200 && status < 300 {
            Ok((status, body))
        } else {
            Err(format!("HTTP {}: {}", status, body))
        }
    }

    /// Deliver webhook with retry logic
    pub async fn deliver_with_retry(
        &self,
        endpoint_url: &str,
        event: &WebhookEvent,
        secret_key: &str,
        timeout_seconds: u64,
        max_retries: i32,
    ) -> (bool, i32, Option<String>) {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < max_retries {
            attempts += 1;
            
            match self.deliver_webhook(endpoint_url, event, secret_key, timeout_seconds).await {
                Ok((status, body)) => {
                    return (true, attempts, Some(format!("HTTP {}: {}", status, body)));
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempts < max_retries {
                        // Exponential backoff: 1s, 2s, 4s, 8s...
                        let delay = Duration::from_secs(2_u64.pow((attempts - 1) as u32));
                        sleep(delay).await;
                    }
                }
            }
        }

        (false, attempts, last_error)
    }
}

impl Default for WebhookService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance_models::WebhookEvent;
    use serde_json::json;

    #[test]
    fn test_signature_generation() {
        let payload = r#"{"event_type":"test","timestamp":"2024-01-01T00:00:00Z","data":{}}"#;
        let secret = "test_secret_key";
        let signature = WebhookService::generate_signature(payload, secret);
        
        assert!(!signature.is_empty());
        // Signature should be base64 encoded
        assert!(signature.len() > 0);
    }

    #[tokio::test]
    async fn test_webhook_service_creation() {
        let service = WebhookService::new();
        // Should not panic
        assert!(true);
    }
}

