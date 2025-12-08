use crate::core::crypto_shredder::VeridionKeyStore;
use crate::core::privacy_bridge::SignicatClient;
use crate::database::Database;
use crate::deployment::DeploymentConfig;
use crate::integration::notifications::NotificationService;
use sqlx::PgPool;
use std::sync::Arc;

/// Application state shared across all request handlers
pub struct AppState {
    /// Key store for crypto-shredding (GDPR compliance)
    pub key_store: Arc<VeridionKeyStore>,
    /// Signicat client for eIDAS sealing
    pub signicat: Arc<SignicatClient>,
    /// Database connection pool
    #[allow(dead_code)]
    pub db: Arc<Database>,
    /// Database pool (for direct access if needed)
    pub db_pool: PgPool,
    /// Deployment configuration
    #[allow(dead_code)]
    pub deployment: DeploymentConfig,
    /// Notification service for GDPR Article 33 and EU AI Act Article 13
    pub notification_service: Arc<NotificationService>,
}

impl AppState {
    /// Create a new AppState with database connection
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let db = Arc::new(Database::new(database_url).await?);
        let db_pool = db.pool().clone();

        Ok(Self {
            key_store: Arc::new(VeridionKeyStore::new()),
            signicat: Arc::new(SignicatClient::new()),
            db: db.clone(),
            db_pool,
            deployment: DeploymentConfig::default(),
            notification_service: Arc::new(NotificationService::new()),
        })
    }

    /// Check if system is locked down
    pub async fn is_locked_down(&self) -> Result<bool, sqlx::Error> {
        let result: Option<Option<String>> = sqlx::query_scalar(
            "SELECT value FROM system_config WHERE key = 'is_locked_down'"
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(match result {
            Some(Some(v)) => v.parse::<bool>().unwrap_or(false),
            _ => false,
        })
    }

    /// Set system lockdown status
    pub async fn set_locked_down(&self, locked: bool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO system_config (key, value) VALUES ('is_locked_down', $1)
             ON CONFLICT (key) DO UPDATE SET value = $1, updated_at = CURRENT_TIMESTAMP"
        )
        .bind(locked.to_string())
        .execute(&self.db_pool)
        .await?;
        Ok(())
    }
}

