use crate::security::auth::Claims;
use actix_web::{HttpRequest, HttpResponse};
use sqlx::PgPool;
use std::collections::HashSet;

/// Permission check result
pub enum PermissionResult {
    Allowed,
    Denied(String),
}

/// RBAC Service
pub struct RbacService {
    db_pool: PgPool,
}

impl RbacService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Check if user has permission
    pub async fn check_permission(
        &self,
        user_id: &str,
        resource: &str,
        action: &str,
    ) -> Result<bool, sqlx::Error> {
        let permission_name = format!("{}.{}", resource, action);

        let result: Option<bool> = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM user_roles ur
                JOIN role_permissions rp ON ur.role_id = rp.role_id
                JOIN permissions p ON rp.permission_id = p.id
                WHERE ur.user_id = $1::uuid
                  AND p.name = $2
            )
            "#,
        )
        .bind(user_id)
        .bind(&permission_name)
        .fetch_optional(&self.db_pool)
        .await?
        .flatten();

        Ok(result.unwrap_or(false))
    }

    /// Get all permissions for a user
    pub async fn get_user_permissions(
        &self,
        user_id: &str,
    ) -> Result<HashSet<String>, sqlx::Error> {
        let permissions: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT DISTINCT p.name
            FROM user_roles ur
            JOIN role_permissions rp ON ur.role_id = rp.role_id
            JOIN permissions p ON rp.permission_id = p.id
            WHERE ur.user_id = $1::uuid
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(permissions.into_iter().collect())
    }

    /// Check permission from claims
    pub async fn check_permission_from_claims(
        &self,
        claims: &Claims,
        resource: &str,
        action: &str,
    ) -> PermissionResult {
        // Admin role has all permissions
        if claims.has_role("admin") {
            return PermissionResult::Allowed;
        }

        match self.check_permission(&claims.sub, resource, action).await {
            Ok(true) => PermissionResult::Allowed,
            Ok(false) => PermissionResult::Denied(format!(
                "User does not have permission: {}.{}",
                resource, action
            )),
            Err(e) => PermissionResult::Denied(format!("Database error: {}", e)),
        }
    }
}

/// Middleware helper to check permissions
pub async fn require_permission(
    req: &HttpRequest,
    rbac: &RbacService,
    claims: &Claims,
    resource: &str,
    action: &str,
) -> Result<(), HttpResponse> {
    match rbac.check_permission_from_claims(claims, resource, action).await {
        PermissionResult::Allowed => Ok(()),
        PermissionResult::Denied(msg) => Err(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Forbidden",
            "message": msg
        }))),
    }
}

/// Require specific role
pub fn require_role(claims: &Claims, required_role: &str) -> Result<(), HttpResponse> {
    if claims.has_role(required_role) || claims.has_role("admin") {
        Ok(())
    } else {
        Err(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Forbidden",
            "message": format!("Required role: {}", required_role)
        })))
    }
}

/// Require any of the specified roles
pub fn require_any_role(claims: &Claims, roles: &[&str]) -> Result<(), HttpResponse> {
    if claims.has_any_role(roles) || claims.has_role("admin") {
        Ok(())
    } else {
        Err(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Forbidden",
            "message": format!("Required one of roles: {:?}", roles)
        })))
    }
}

