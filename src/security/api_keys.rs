use sha2::{Sha256, Digest};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use base64::engine::Engine as _;

/// API Key information (without the actual key)
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyInfo {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub user_id: Option<Uuid>,
    pub permissions: Vec<String>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
    pub last_used_at: Option<chrono::DateTime<Utc>>,
    pub active: bool,
    pub created_at: chrono::DateTime<Utc>,
}

/// API Key Service
pub struct ApiKeyService {
    db_pool: PgPool,
}

impl ApiKeyService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Generate a new API key
    pub fn generate_key() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        format!("vn_{}", base64::engine::general_purpose::STANDARD.encode(&bytes))
    }

    /// Hash API key for storage
    pub fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Create a new API key
    pub async fn create_api_key(
        &self,
        name: String,
        description: Option<String>,
        user_id: Option<Uuid>,
        permissions: Vec<String>,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<(String, ApiKeyInfo), sqlx::Error> {
        let key = Self::generate_key();
        let key_hash = Self::hash_key(&key);

        let id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO api_keys (id, key_hash, name, description, user_id, permissions, expires_at, active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
            "#,
        )
        .bind(&id)
        .bind(&key_hash)
        .bind(&name)
        .bind(&description)
        .bind(&user_id)
        .bind(&permissions)
        .bind(&expires_at)
        .bind(true)
        .bind(&now)
        .execute(&self.db_pool)
        .await?;

        let info = ApiKeyInfo {
            id,
            name,
            description,
            user_id,
            permissions,
            expires_at,
            last_used_at: None,
            active: true,
            created_at: now,
        };

        Ok((key, info))
    }

    /// Validate API key
    #[allow(dead_code)]
    pub async fn validate_api_key(
        &self,
        key: &str,
    ) -> Result<Option<ApiKeyInfo>, sqlx::Error> {
        let key_hash = Self::hash_key(key);

        let result: Option<(Uuid, String, Option<String>, Option<Uuid>, Vec<String>, Option<chrono::DateTime<Utc>>, Option<chrono::DateTime<Utc>>, bool, chrono::DateTime<Utc>)> = sqlx::query_as(
            r#"
            SELECT id, name, description, user_id, permissions, expires_at, last_used_at, active, created_at
            FROM api_keys
            WHERE key_hash = $1 AND active = true
            "#,
        )
        .bind(&key_hash)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some((id, name, description, user_id, permissions, expires_at, _last_used_at, active, created_at)) = result {
            // Check expiration
            if let Some(exp) = expires_at {
                if exp < Utc::now() {
                    return Ok(None);
                }
            }

            // Update last_used_at
            sqlx::query(
                "UPDATE api_keys SET last_used_at = $1 WHERE id = $2"
            )
            .bind(&Utc::now())
            .bind(&id)
            .execute(&self.db_pool)
            .await?;

            Ok(Some(ApiKeyInfo {
                id,
                name,
                description,
                user_id,
                permissions,
                expires_at,
                last_used_at: Some(Utc::now()),
                active,
                created_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Revoke API key
    pub async fn revoke_api_key(&self, key_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE api_keys SET active = false, updated_at = CURRENT_TIMESTAMP WHERE id = $1")
            .bind(&key_id)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    /// List API keys for a user
    pub async fn list_api_keys(
        &self,
        user_id: Option<Uuid>,
    ) -> Result<Vec<ApiKeyInfo>, sqlx::Error> {
        let keys: Vec<(Uuid, String, Option<String>, Option<Uuid>, Vec<String>, Option<chrono::DateTime<Utc>>, Option<chrono::DateTime<Utc>>, bool, chrono::DateTime<Utc>)> = if let Some(uid) = user_id {
            sqlx::query_as(
                r#"
                SELECT id, name, description, user_id, permissions, expires_at, last_used_at, active, created_at
                FROM api_keys
                WHERE user_id = $1
                ORDER BY created_at DESC
                "#,
            )
            .bind(&uid)
            .fetch_all(&self.db_pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                SELECT id, name, description, user_id, permissions, expires_at, last_used_at, active, created_at
                FROM api_keys
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&self.db_pool)
            .await?
        };

        Ok(keys.into_iter().map(|(id, name, description, user_id, permissions, expires_at, last_used_at, active, created_at)| {
            ApiKeyInfo {
                id,
                name,
                description,
                user_id,
                permissions,
                expires_at,
                last_used_at,
                active,
                created_at,
            }
        }).collect())
    }
}

