use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use serde_json::Value;

/// Security audit log service
pub struct AuditService {
    db_pool: PgPool,
}

impl AuditService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Log a security event
    pub async fn log_event(
        &self,
        user_id: Option<Uuid>,
        api_key_id: Option<Uuid>,
        event_type: &str,
        resource: Option<&str>,
        action: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        success: bool,
        error_message: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO security_audit_logs (
                user_id, api_key_id, event_type, resource, action,
                ip_address, user_agent, success, error_message, metadata, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(&user_id)
        .bind(&api_key_id)
        .bind(event_type)
        .bind(&resource)
        .bind(&action)
        .bind(&ip_address)
        .bind(&user_agent)
        .bind(success)
        .bind(&error_message)
        .bind(&metadata)
        .bind(&Utc::now())
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Log login attempt
    pub async fn log_login(
        &self,
        user_id: Option<Uuid>,
        username: &str,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        success: bool,
        error_message: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        self.log_event(
            user_id,
            None,
            "login",
            Some("auth"),
            Some("login"),
            ip_address,
            user_agent,
            success,
            error_message,
            Some(serde_json::json!({ "username": username })),
        )
        .await
    }

    /// Log permission denied
    pub async fn log_permission_denied(
        &self,
        user_id: Option<Uuid>,
        resource: &str,
        action: &str,
        ip_address: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        self.log_event(
            user_id,
            None,
            "permission_denied",
            Some(resource),
            Some(action),
            ip_address,
            None,
            false,
            Some("Permission denied"),
            None,
        )
        .await
    }

    /// Log rate limit exceeded
    pub async fn log_rate_limit(
        &self,
        identifier: &str,
        endpoint: &str,
        ip_address: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        self.log_event(
            None,
            None,
            "rate_limit_exceeded",
            Some(endpoint),
            None,
            ip_address,
            None,
            false,
            Some("Rate limit exceeded"),
            Some(serde_json::json!({
                "identifier": identifier,
                "endpoint": endpoint
            })),
        )
        .await
    }

    /// Get audit logs
    pub async fn get_audit_logs(
        &self,
        user_id: Option<Uuid>,
        event_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditLog>, sqlx::Error> {
        let logs: Vec<AuditLog> = if let Some(uid) = user_id {
            if let Some(et) = event_type {
                sqlx::query_as(
                    r#"
                    SELECT id, user_id, api_key_id, event_type, resource, action,
                           ip_address, user_agent, success, error_message, metadata, created_at
                    FROM security_audit_logs
                    WHERE user_id = $1 AND event_type = $2
                    ORDER BY created_at DESC
                    LIMIT $3 OFFSET $4
                    "#,
                )
                .bind(&uid)
                .bind(et)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.db_pool)
                .await?
            } else {
                sqlx::query_as(
                    r#"
                    SELECT id, user_id, api_key_id, event_type, resource, action,
                           ip_address, user_agent, success, error_message, metadata, created_at
                    FROM security_audit_logs
                    WHERE user_id = $1
                    ORDER BY created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(&uid)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.db_pool)
                .await?
            }
        } else if let Some(et) = event_type {
            sqlx::query_as(
                r#"
                SELECT id, user_id, api_key_id, event_type, resource, action,
                       ip_address, user_agent, success, error_message, metadata, created_at
                FROM security_audit_logs
                WHERE event_type = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(et)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db_pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                SELECT id, user_id, api_key_id, event_type, resource, action,
                       ip_address, user_agent, success, error_message, metadata, created_at
                FROM security_audit_logs
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db_pool)
            .await?
        };

        Ok(logs)
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub api_key_id: Option<Uuid>,
    pub event_type: String,
    pub resource: Option<String>,
    pub action: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
    pub metadata: Option<Value>,
    pub created_at: chrono::DateTime<Utc>,
}

