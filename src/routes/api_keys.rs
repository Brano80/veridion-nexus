use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::api_state::AppState;
use crate::security::{AuthService, AuditService, extract_claims, RbacService, require_permission};
use crate::security::api_keys::ApiKeyService;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use chrono::Utc;

#[derive(Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub expires_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct CreateApiKeyResponse {
    pub api_key: String,
    pub key_info: ApiKeyInfoResponse,
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct ApiKeyInfoResponse {
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

#[derive(Serialize, ToSchema)]
pub struct ApiKeysListResponse {
    pub api_keys: Vec<ApiKeyInfoResponse>,
    pub total_count: usize,
}

/// Create a new API key
#[utoipa::path(
    post,
    path = "/api/v1/api_keys",
    request_body = CreateApiKeyRequest,
    responses(
        (status = 201, description = "API key created", body = CreateApiKeyResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn create_api_key(
    req: web::Json<CreateApiKeyRequest>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // Authenticate and authorize
    let auth_service = AuthService::new().unwrap();
    let claims = match extract_claims(&http_req, &auth_service) {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let rbac = RbacService::new(data.db_pool.clone());
    let audit_service = AuditService::new(data.db_pool.clone());
    
    // Check permission - only admin can create API keys
    if let Err(resp) = require_permission(&http_req, &rbac, &claims, "api_key", "write").await {
        let user_id = uuid::Uuid::parse_str(&claims.sub).ok();
        let ip_addr = http_req.connection_info().peer_addr().map(|s| s.to_string());
        audit_service.log_permission_denied(
            user_id,
            "api_key",
            "write",
            ip_addr.as_deref(),
        ).await.ok();
        return resp;
    }

    let user_id = Uuid::parse_str(&claims.sub).ok();
    let api_key_service = ApiKeyService::new(data.db_pool.clone());

    match api_key_service.create_api_key(
        req.name.clone(),
        req.description.clone(),
        user_id,
        req.permissions.clone(),
        req.expires_at,
    ).await {
        Ok((key, info)) => {
            let ip_addr = http_req.connection_info().peer_addr().map(|s| s.to_string());
            audit_service.log_event(
                user_id,
                None,
                "api_key.created",
                Some("api_keys"),
                Some("write"),
                ip_addr.as_deref(),
                None,
                true,
                None,
                Some(serde_json::json!({ "key_name": req.name })),
            ).await.ok();

            HttpResponse::Created().json(CreateApiKeyResponse {
                api_key: key,
                key_info: ApiKeyInfoResponse {
                    id: info.id,
                    name: info.name,
                    description: info.description,
                    user_id: info.user_id,
                    permissions: info.permissions,
                    expires_at: info.expires_at,
                    last_used_at: info.last_used_at,
                    active: info.active,
                    created_at: info.created_at,
                },
                message: "API key created successfully. Store it securely - it will not be shown again.".to_string(),
            })
        }
        Err(e) => {
            eprintln!("Error creating API key: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create API key"
            }))
        }
    }
}

/// List API keys
#[utoipa::path(
    get,
    path = "/api/v1/api_keys",
    responses(
        (status = 200, description = "List of API keys", body = ApiKeysListResponse),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_api_keys(
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // Authenticate
    let auth_service = AuthService::new().unwrap();
    let claims = match extract_claims(&http_req, &auth_service) {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let user_id = Uuid::parse_str(&claims.sub).ok();
    let api_key_service = ApiKeyService::new(data.db_pool.clone());

    // Users can only see their own keys unless they're admin
    let is_admin = claims.roles.contains(&"admin".to_string());
    let filter_user_id = if is_admin { None } else { user_id };

    match api_key_service.list_api_keys(filter_user_id).await {
        Ok(keys) => {
            let api_keys: Vec<ApiKeyInfoResponse> = keys.into_iter().map(|info| {
                ApiKeyInfoResponse {
                    id: info.id,
                    name: info.name,
                    description: info.description,
                    user_id: info.user_id,
                    permissions: info.permissions,
                    expires_at: info.expires_at,
                    last_used_at: info.last_used_at,
                    active: info.active,
                    created_at: info.created_at,
                }
            }).collect();

            HttpResponse::Ok().json(ApiKeysListResponse {
                total_count: api_keys.len(),
                api_keys,
            })
        }
        Err(e) => {
            eprintln!("Error listing API keys: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to list API keys"
            }))
        }
    }
}

/// Get API key details
#[utoipa::path(
    get,
    path = "/api/v1/api_keys/{id}",
    responses(
        (status = 200, description = "API key details", body = ApiKeyInfoResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "API key not found")
    )
)]
pub async fn get_api_key(
    path: web::Path<String>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // Authenticate
    let auth_service = AuthService::new().unwrap();
    let claims = match extract_claims(&http_req, &auth_service) {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let key_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid API key ID"
            }));
        }
    };

    let user_id = Uuid::parse_str(&claims.sub).ok();
    let api_key_service = ApiKeyService::new(data.db_pool.clone());

    match api_key_service.list_api_keys(None).await {
        Ok(keys) => {
            let key = keys.into_iter().find(|k| k.id == key_id);
            
            match key {
                Some(info) => {
                    // Check if user owns the key or is admin
                    let is_admin = claims.roles.contains(&"admin".to_string());
                    if !is_admin && info.user_id != user_id {
                        return HttpResponse::Forbidden().json(serde_json::json!({
                            "error": "Access denied"
                        }));
                    }

                    HttpResponse::Ok().json(ApiKeyInfoResponse {
                        id: info.id,
                        name: info.name,
                        description: info.description,
                        user_id: info.user_id,
                        permissions: info.permissions,
                        expires_at: info.expires_at,
                        last_used_at: info.last_used_at,
                        active: info.active,
                        created_at: info.created_at,
                    })
                }
                None => HttpResponse::NotFound().json(serde_json::json!({
                    "error": "API key not found"
                }))
            }
        }
        Err(e) => {
            eprintln!("Error fetching API key: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch API key"
            }))
        }
    }
}

/// Revoke API key
#[utoipa::path(
    delete,
    path = "/api/v1/api_keys/{id}",
    responses(
        (status = 200, description = "API key revoked"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "API key not found")
    )
)]
pub async fn revoke_api_key(
    path: web::Path<String>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // Authenticate and authorize
    let auth_service = AuthService::new().unwrap();
    let claims = match extract_claims(&http_req, &auth_service) {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let key_id = match Uuid::parse_str(&path.into_inner()) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid API key ID"
            }));
        }
    };

    let user_id = Uuid::parse_str(&claims.sub).ok();
    let api_key_service = ApiKeyService::new(data.db_pool.clone());
    let audit_service = AuditService::new(data.db_pool.clone());

    // Check if key exists and user has permission
    match api_key_service.list_api_keys(None).await {
        Ok(keys) => {
            let key = keys.into_iter().find(|k| k.id == key_id);
            
            match key {
                Some(info) => {
                    // Check if user owns the key or is admin
                    let is_admin = claims.roles.contains(&"admin".to_string());
                    if !is_admin && info.user_id != user_id {
                        return HttpResponse::Forbidden().json(serde_json::json!({
                            "error": "Access denied"
                        }));
                    }

                    // Revoke the key
                    match api_key_service.revoke_api_key(key_id).await {
                        Ok(_) => {
                            let ip_addr = http_req.connection_info().peer_addr().map(|s| s.to_string());
                            audit_service.log_event(
                                user_id,
                                None,
                                "api_key.revoked",
                                Some("api_keys"),
                                Some("delete"),
                                ip_addr.as_deref(),
                                None,
                                true,
                                None,
                                Some(serde_json::json!({ "key_name": info.name })),
                            ).await.ok();

                            HttpResponse::Ok().json(serde_json::json!({
                                "message": "API key revoked successfully"
                            }))
                        }
                        Err(e) => {
                            eprintln!("Error revoking API key: {}", e);
                            HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": "Failed to revoke API key"
                            }))
                        }
                    }
                }
                None => HttpResponse::NotFound().json(serde_json::json!({
                    "error": "API key not found"
                }))
            }
        }
        Err(e) => {
            eprintln!("Error fetching API key: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch API key"
            }))
        }
    }
}

