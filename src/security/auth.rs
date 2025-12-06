use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use actix_web::{HttpRequest, HttpResponse};
use std::env;

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub username: String,
    pub roles: Vec<String>,
    pub exp: i64, // expiration time
    pub iat: i64, // issued at
}

impl Claims {
    pub fn new(user_id: String, username: String, roles: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id,
            username,
            roles,
            exp: (now + Duration::hours(24)).timestamp(),
            iat: now.timestamp(),
        }
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|r| self.has_role(r))
    }
}

/// JWT Authentication service
pub struct AuthService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl AuthService {
    pub fn new() -> Result<Self, String> {
        let secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());

        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());

        Ok(Self {
            encoding_key,
            decoding_key,
        })
    }

    pub fn generate_token(&self, claims: &Claims) -> Result<String, String> {
        encode(&Header::default(), claims, &self.encoding_key)
            .map_err(|e| format!("Failed to generate token: {}", e))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, String> {
        let validation = Validation::default();
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| format!("Invalid token: {}", e))?;

        // Check expiration
        let now = Utc::now().timestamp();
        if token_data.claims.exp < now {
            return Err("Token expired".to_string());
        }

        Ok(token_data.claims)
    }

    pub fn extract_token_from_request(req: &HttpRequest) -> Option<String> {
        // Try Authorization header first
        if let Some(auth_header) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    return Some(auth_str[7..].to_string());
                }
            }
        }

        // Try query parameter
        if let Some(token) = req.uri().query() {
            if let Some(token_val) = token.strip_prefix("token=") {
                return Some(token_val.to_string());
            }
        }

        None
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new().expect("Failed to initialize AuthService")
    }
}

/// Extract claims from request
pub fn extract_claims(req: &HttpRequest, auth_service: &AuthService) -> Result<Claims, HttpResponse> {
    let token = AuthService::extract_token_from_request(req)
        .ok_or_else(|| {
            HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Unauthorized",
                "message": "Missing or invalid authentication token"
            }))
        })?;

    auth_service.validate_token(&token).map_err(|e| {
        HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Unauthorized",
            "message": e
        }))
    })
}

