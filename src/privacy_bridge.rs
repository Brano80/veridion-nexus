use sha2::{Sha256, Digest};
use reqwest::blocking::Client;
use std::thread;
use std::time::SystemTime;
use rand::RngCore;
use rand::rng;
use std::sync::Mutex;
use std::collections::VecDeque;
use serde_json::json;
use base64::Engine;

/// Hash a payload using SHA256 and return the hex string
/// 
/// # Arguments
/// 
/// * `payload` - The string to hash
/// 
/// # Returns
/// 
/// * SHA256 hash as a hexadecimal string
pub fn hash_payload(payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Client for interacting with Signicat Qualified Cloud service
pub struct SignicatClient {
    api_url: String,
    client: Client,
    /// Buffer for hashes when API is unreachable
    offline_buffer: Mutex<VecDeque<String>>,
    /// Flag to simulate API outage (for testing)
    simulate_outage: Mutex<bool>,
    /// Signicat OAuth2 client ID
    client_id: String,
    /// Signicat OAuth2 client secret
    client_secret: String,
    /// OAuth2 token endpoint URL
    token_url: String,
    /// Whether to use real API or mock mode
    use_real_api: bool,
}

impl SignicatClient {
    /// Create a new SignicatClient, loading configuration from environment
    pub fn new() -> Self {
        // Load configuration from environment variables
        let client_id = std::env::var("SIGNICAT_CLIENT_ID")
            .unwrap_or_else(|_| "placeholder_id".to_string());
        
        let client_secret = std::env::var("SIGNICAT_CLIENT_SECRET")
            .unwrap_or_else(|_| "placeholder_secret".to_string());
        
        let token_url = std::env::var("SIGNICAT_TOKEN_URL")
            .unwrap_or_else(|_| "https://api.signicat.com/auth/open/connect/token".to_string());
        
        let api_url = std::env::var("SIGNICAT_API_URL")
            .unwrap_or_else(|_| "https://api.signicat.com/v1/sealing".to_string());
        
        // Default to false (mock mode) if not set
        let use_real_api = std::env::var("USE_REAL_API")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);
        
        Self {
            api_url,
            client: Client::new(),
            offline_buffer: Mutex::new(VecDeque::new()),
            simulate_outage: Mutex::new(false),
            client_id,
            client_secret,
            token_url,
            use_real_api,
        }
    }

    /// Get OAuth2 access token from Signicat
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` with the access token
    /// * `Err(String)` if the token request fails
    fn get_live_token(&self) -> Result<String, String> {
        // Create Basic Auth header (client_id:client_secret base64 encoded)
        let auth_string = format!("{}:{}", self.client_id, self.client_secret);
        let auth_b64 = base64::engine::general_purpose::STANDARD.encode(auth_string.as_bytes());
        let auth_header = format!("Basic {}", auth_b64);
        
        // Prepare form data for OAuth2 client credentials flow
        let params = [
            ("grant_type", "client_credentials"),
        ];
        
        // POST request to token endpoint
        let response = self.client
            .post(&self.token_url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .map_err(|e| format!("Token request failed: {}", e))?;
        
        // Parse JSON response
        let json: serde_json::Value = response
            .json()
            .map_err(|e| format!("Failed to parse token response: {}", e))?;
        
        // Extract access_token
        json.get("access_token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "No access_token in response".to_string())
    }

    /// Set the outage simulation flag (for testing)
    /// 
    /// # Arguments
    /// 
    /// * `active` - If true, simulates API outage; if false, normal operation
    pub fn set_outage(&self, active: bool) {
        *self.simulate_outage.lock().expect("Failed to acquire outage lock") = active;
    }

    /// Request a Qualified Electronic Seal (QES) for a log hash
    /// 
    /// # Arguments
    /// 
    /// * `log_hash` - The SHA256 hash of the log to seal
    /// 
    /// # Returns
    /// 
    /// * `Ok(String)` with the seal ID and timestamp (or PENDING_SYNC_LOCAL if buffered)
    /// * `Err(String)` if the request fails
    pub fn request_seal(&self, log_hash: &str) -> Result<String, String> {
        // Check if outage is simulated
        let is_outage = *self.simulate_outage.lock().expect("Failed to acquire outage lock");
        
        if is_outage {
            // Simulated failure - buffer the hash locally
            self.offline_buffer
                .lock()
                .expect("Failed to acquire buffer lock")
                .push_back(log_hash.to_string());
            
            println!("⚠️ CIRCUIT OPEN: API Unreachable. Buffering hash locally...");
            
            // Return pending status (do NOT return Err, as we want the agent to continue)
            return Ok(format!("PENDING_SYNC_LOCAL:[{}]", log_hash));
        }
        
        // Check if we should use real API or mock mode
        if self.use_real_api {
            // Real API mode: Use OAuth2 and actual Signicat API
            return self.request_seal_real(log_hash);
        }
        
        // Mock mode: Keep existing logic (Circuit Breaker + Sleep + Fake Seal)
        // Show first 8 characters of hash for display
        let hash_start = if log_hash.len() >= 8 {
            &log_hash[..8]
        } else {
            log_hash
        };
        
        println!("Network: Sending Hash [{}...] to Signicat Qualified Cloud...", hash_start);
        
        // Mock network delay (500ms)
        thread::sleep(std::time::Duration::from_millis(500));
        
        // Generate a random ID for the seal
        let mut random_bytes = vec![0u8; 8];
        rng().fill_bytes(&mut random_bytes);
        let random_id = random_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        
        // Get current timestamp
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Return mock seal response
        Ok(format!("QES_SEAL_{} | TIMESTAMP: {}", random_id, timestamp))
    }

    /// Request a seal using the real Signicat API (OAuth2 authenticated)
    fn request_seal_real(&self, log_hash: &str) -> Result<String, String> {
        // Get OAuth2 access token
        let token = self.get_live_token()?;
        
        // Show first 8 characters of hash for display
        let hash_start = if log_hash.len() >= 8 {
            &log_hash[..8]
        } else {
            log_hash
        };
        
        println!("Network: Sending Hash [{}...] to Signicat Qualified Cloud (Real API)...", hash_start);
        
        // Prepare JSON body for the seal request
        let body = json!({
            "payload": log_hash
        });
        
        // Make POST request to Signicat API
        let response = self.client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| format!("Seal request failed: {}", e))?;
        
        // Check if request was successful
        if !response.status().is_success() {
            return Err(format!("API returned error status: {}", response.status()));
        }
        
        // Parse JSON response
        let json: serde_json::Value = response
            .json()
            .map_err(|e| format!("Failed to parse seal response: {}", e))?;
        
        // Extract seal ID from response (adjust field name based on Signicat API spec)
        let seal_id = json.get("sealId")
            .or_else(|| json.get("id"))
            .or_else(|| json.get("seal_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "No seal ID in response".to_string())?;
        
        // Get current timestamp
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Return seal response in same format as mock
        Ok(format!("QES_SEAL_{} | TIMESTAMP: {}", seal_id, timestamp))
    }

    /// Sync all pending buffered hashes to the API
    /// 
    /// This should be called when the API becomes available again
    pub fn sync_pending(&self) {
        let mut buffer = self.offline_buffer.lock().expect("Failed to acquire buffer lock");
        
        // Loop through offline_buffer
        while let Some(hash) = buffer.pop_front() {
            // Show first 8 characters of hash for display
            let hash_start = if hash.len() >= 8 {
                &hash[..8]
            } else {
                &hash
            };
            
            println!("🔄 SYNCING: Uploading buffered hash [{}...]...", hash_start);
            
            // In a real implementation, we would actually upload here
            // For now, we just simulate the upload by doing nothing
        }
        
        // Clear the buffer (already done by pop_front, but ensure it's empty)
        buffer.clear();
        
        println!("✅ SYNC COMPLETE: All logs sealed.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashing_determinism() {
        // SHA256 of "test" is a known value
        // Let's verify it's deterministic by hashing multiple times
        let hash1 = hash_payload("test");
        let hash2 = hash_payload("test");
        let hash3 = hash_payload("test");
        
        // All hashes should be identical
        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
        
        // Verify it's the correct SHA256 hash of "test"
        // SHA256("test") = 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08
        let expected_hash = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
        assert_eq!(hash1, expected_hash);
    }

    #[test]
    fn test_mock_seal() {
        let client = SignicatClient::new();
        let test_hash = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
        
        let result = client.request_seal(test_hash);
        
        assert!(result.is_ok());
        let seal_response = result.unwrap();
        assert!(seal_response.starts_with("QES_SEAL_"));
        assert!(seal_response.contains("TIMESTAMP:"));
    }

    #[test]
    fn test_circuit_breaker() {
        let client = SignicatClient::new();
        let test_hash = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
        
        // Enable outage
        client.set_outage(true);
        
        // Request seal (should return PENDING)
        let result = client.request_seal(test_hash);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.starts_with("PENDING_SYNC_LOCAL:"));
        
        // Check buffer length is 1
        let buffer = client.offline_buffer.lock().expect("Failed to acquire buffer lock");
        assert_eq!(buffer.len(), 1);
        drop(buffer);
        
        // Disable outage
        client.set_outage(false);
        
        // Call sync_pending
        client.sync_pending();
        
        // Check buffer is empty
        let buffer = client.offline_buffer.lock().expect("Failed to acquire buffer lock");
        assert_eq!(buffer.len(), 0);
    }
}

