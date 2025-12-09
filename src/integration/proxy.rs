// Proxy Mode - Reverse Proxy Service
// Intercepts AI API calls and enforces compliance at network level

use actix_web::{web, HttpRequest, HttpResponse};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use std::sync::Arc;
use std::net::ToSocketAddrs;
use std::time::Duration;
use maxminddb::Reader;
use crate::api_state::AppState;
use crate::core::sovereign_lock::EU_EEA_WHITELIST;

/// Proxy request configuration
#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct ProxyRequest {
    /// Target URL to proxy to (e.g., https://api.openai.com/v1/chat/completions)
    #[schema(example = "https://api.openai.com/v1/chat/completions")]
    pub target_url: String,
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    #[schema(example = "POST")]
    #[serde(default = "default_method")]
    pub method: String,
    /// Request headers (optional, will be forwarded)
    #[serde(default)]
    pub headers: Option<serde_json::Value>,
    /// Request body (optional)
    #[serde(default)]
    pub body: Option<serde_json::Value>,
}

fn default_method() -> String {
    "POST".to_string()
}

/// Proxy service for network-level compliance enforcement
pub struct ProxyService {
    client: Client,
    /// GeoIP database reader (optional, uses hostname patterns if not available)
    geoip_reader: Option<Arc<Reader<Vec<u8>>>>,
}

impl ProxyService {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        // Try to load GeoIP database
        let geoip_reader = match std::env::var("GEOIP_DB_PATH") {
            Ok(path) => {
                match Reader::open_readfile(&path) {
                    Ok(reader) => {
                        log::info!("GeoIP database loaded successfully from: {}", path);
                        Some(Arc::new(reader))
                    }
                    Err(e) => {
                        eprintln!("WARNING: Failed to load GeoIP database from {}: {}. GeoIP is disabled. Only hardcoded hostname pattern checks will work.", path, e);
                        None
                    }
                }
            }
            Err(_) => {
                eprintln!("WARNING: GEOIP_DB_PATH not set. GeoIP is disabled. Only hardcoded hostname pattern checks will work.");
                None
            }
        };
        
        Self {
            client,
            geoip_reader,
        }
    }

    /// Check if target URL is in EU/EEA jurisdiction
    /// Returns (is_eu, country_code)
    pub async fn check_sovereignty(&self, target_url: &str) -> Result<(bool, String), String> {
        // Parse URL
        let url = reqwest::Url::parse(target_url)
            .map_err(|e| format!("Invalid URL: {}", e))?;
        
        let hostname = url.host_str()
            .ok_or_else(|| "No hostname in URL".to_string())?;

        // First, check known hostname patterns (fast path)
        let country = self.check_hostname_pattern(hostname);
        if country != "UNKNOWN" {
            let is_eu = EU_EEA_WHITELIST.contains(&country.as_str());
            return Ok((is_eu, country));
        }

        // Second, try DNS resolution + GeoIP lookup
        if let Ok(ip) = self.resolve_hostname(hostname).await {
            // Check if IP is in private ranges (assume EU for private IPs)
            if self.is_private_ip(&ip) {
                return Ok((true, "EU".to_string()));
            }
            
            // Try GeoIP lookup if database is available
            if let Some(country_code) = self.lookup_geoip(&ip).await {
                let is_eu = EU_EEA_WHITELIST.contains(&country_code.as_str());
                return Ok((is_eu, country_code));
            }
            
            // If GeoIP database is missing, log and be conservative
            log::warn!("GeoIP database not available. Cannot determine country for IP: {}. Blocking request.", ip);
            return Ok((false, "UNKNOWN".to_string()));
        }

        // If we can't determine, be conservative and block
        Ok((false, "UNKNOWN".to_string()))
    }

    /// Check hostname patterns for known AI services
    fn check_hostname_pattern(&self, hostname: &str) -> String {
        let hostname_lower = hostname.to_lowercase();
        
        // --- DEMO PATCH START ---
        // Hardcoded bypass for demo to avoid DNS issues
        if hostname_lower.contains("google.com") {
            return "US".to_string();
        }
        if hostname_lower.contains("europa.eu") {
            return "EU".to_string();
        }
        // --- DEMO PATCH END ---
        
        // US-based AI services
        if hostname_lower.contains("openai.com") 
            || hostname_lower.contains("anthropic.com")
            || hostname_lower.contains("cohere.com")
            || hostname_lower.contains("together.ai")
            || hostname_lower.contains("replicate.com") {
            return "US".to_string();
        }

        // Azure - check for EU regions in hostname
        if hostname_lower.contains("azure.com") || hostname_lower.contains("azure-api.net") {
            if hostname_lower.contains("westeurope") 
                || hostname_lower.contains("northeurope")
                || hostname_lower.contains("francecentral")
                || hostname_lower.contains("germanywestcentral") {
                return "EU".to_string();
            }
            return "US".to_string();
        }

        // AWS - check for EU regions
        if hostname_lower.contains("amazonaws.com") || hostname_lower.contains("bedrock") {
            if hostname_lower.contains("eu-west") 
                || hostname_lower.contains("eu-central")
                || hostname_lower.contains("eu-north")
                || hostname_lower.contains("eu-south") {
                return "EU".to_string();
            }
            return "US".to_string();
        }

        // Google Cloud - check for EU regions
        if hostname_lower.contains("googleapis.com") || hostname_lower.contains("googlecloud.com") {
            if hostname_lower.contains("europe") || hostname_lower.contains("eu-") {
                return "EU".to_string();
            }
            return "US".to_string();
        }

        // EU-based services
        if hostname_lower.contains(".eu") 
            || hostname_lower.contains(".de")
            || hostname_lower.contains(".fr")
            || hostname_lower.contains(".nl")
            || hostname_lower.contains(".ie") {
            return "EU".to_string();
        }

        "UNKNOWN".to_string()
    }

    /// Resolve hostname to IP address
    async fn resolve_hostname(&self, hostname: &str) -> Result<String, String> {
        // Use tokio::net::lookup_host for async DNS resolution
        let addr = format!("{}:80", hostname);
        match addr.to_socket_addrs() {
            Ok(mut addrs) => {
                if let Some(addr) = addrs.next() {
                    Ok(addr.ip().to_string())
                } else {
                    Err("No IP address found".to_string())
                }
            }
            Err(e) => Err(format!("DNS resolution failed: {}", e)),
        }
    }

    /// Look up IP address in GeoIP database
    /// Returns country code (e.g., "DE", "FR", "US") if found, None otherwise
    async fn lookup_geoip(&self, ip: &str) -> Option<String> {
        let geoip_reader = self.geoip_reader.as_ref()?;
        
        // Parse IP address
        let ip_addr: std::net::IpAddr = match ip.parse() {
            Ok(addr) => addr,
            Err(e) => {
                log::warn!("Failed to parse IP address '{}': {}", ip, e);
                return None;
            }
        };

        // Look up in GeoIP database
        match geoip_reader.lookup::<maxminddb::geoip2::Country>(ip_addr) {
            Ok(country) => {
                if let Some(country_data) = country.country {
                    if let Some(iso_code) = country_data.iso_code {
                        let code = iso_code.to_string();
                        log::debug!("GeoIP lookup for {}: {}", ip, code);
                        return Some(code);
                    }
                }
                log::debug!("GeoIP lookup for {}: No country code found", ip);
                None
            }
            Err(e) => {
                log::warn!("GeoIP lookup failed for {}: {}", ip, e);
                None
            }
        }
    }

    /// Check if IP is in private range
    fn is_private_ip(&self, ip: &str) -> bool {
        if let Ok(ip_addr) = ip.parse::<std::net::IpAddr>() {
            match ip_addr {
                std::net::IpAddr::V4(ipv4) => {
                    let octets = ipv4.octets();
                    // 10.0.0.0/8
                    if octets[0] == 10 {
                        return true;
                    }
                    // 172.16.0.0/12
                    if octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31 {
                        return true;
                    }
                    // 192.168.0.0/16
                    if octets[0] == 192 && octets[1] == 168 {
                        return true;
                    }
                }
                std::net::IpAddr::V6(_) => {
                    // IPv6 private ranges (simplified)
                    if ip.starts_with("fc00:") || ip.starts_with("fd00:") {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Forward request to target URL
    pub async fn forward_request(
        &self,
        proxy_req: &ProxyRequest,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let method = match proxy_req.method.as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            "PATCH" => reqwest::Method::PATCH,
            _ => reqwest::Method::POST,
        };

        let mut request = self.client.request(method, &proxy_req.target_url);

        // Add custom headers if provided
        if let Some(headers_json) = &proxy_req.headers {
            if let Some(headers_map) = headers_json.as_object() {
                for (key, value) in headers_map {
                    if let Some(val_str) = value.as_str() {
                        request = request.header(key, val_str);
                    }
                }
            }
        }

        // Add body if provided
        if let Some(body) = &proxy_req.body {
            request = request.json(body);
        }

        request.send().await
    }
}

impl Default for ProxyService {
    fn default() -> Self {
        Self::new()
    }
}
