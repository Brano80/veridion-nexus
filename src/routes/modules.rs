use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::api_state::AppState;
use crate::module_service::ModuleService;
use crate::security::{AuthService, extract_claims, RbacService, require_permission, AuditService, Claims};
use uuid::Uuid;
use sqlx;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ModuleInfo {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub category: String,
    pub enabled: bool,
    pub requires_license: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ModulesListResponse {
    pub modules: Vec<ModuleInfo>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct EnableModuleRequest {
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ModuleResponse {
    pub status: String,
    pub message: String,
}

/// Helper function to authenticate and authorize
async fn authenticate_and_authorize(
    http_req: &HttpRequest,
    db_pool: &sqlx::PgPool,
    resource: &str,
    action: &str,
) -> Result<Claims, HttpResponse> {
    let auth_service = AuthService::new()
        .map_err(|e| HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to initialize auth service: {}", e)
        })))?;
    let claims = extract_claims(http_req, &auth_service)?;

    let rbac = RbacService::new(db_pool.clone());
    let audit_service = AuditService::new(db_pool.clone());
    
    if let Err(resp) = require_permission(http_req, &rbac, &claims, resource, action).await {
        let user_id = uuid::Uuid::parse_str(&claims.sub).ok();
        let ip_addr = http_req.connection_info().peer_addr().map(|s| s.to_string());
        audit_service.log_permission_denied(
            user_id,
            resource,
            action,
            ip_addr.as_deref(),
        ).await.ok();
        return Err(resp);
    }

    Ok(claims)
}

/// Get all modules with their activation status
#[utoipa::path(
    get,
    path = "/modules",
    responses((status = 200, body = ModulesListResponse))
)]
pub async fn list_modules(
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "module", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let module_service = ModuleService::new(data.db_pool.clone());
    
    match module_service.get_all_modules().await {
        Ok(modules) => {
            let module_infos: Vec<ModuleInfo> = modules.into_iter().map(|(module, enabled)| {
                ModuleInfo {
                    id: module.id,
                    name: module.name,
                    display_name: module.display_name,
                    description: module.description,
                    category: module.category,
                    enabled,
                    requires_license: module.requires_license,
                }
            }).collect();

            HttpResponse::Ok().json(ModulesListResponse {
                modules: module_infos,
            })
        }
        Err(e) => {
            eprintln!("Error listing modules: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to list modules"
            }))
        }
    }
}

/// Enable a module
#[utoipa::path(
    post,
    path = "/modules/{name}/enable",
    request_body = EnableModuleRequest,
    responses((status = 200, body = ModuleResponse))
)]
pub async fn enable_module(
    path: web::Path<String>,
    body: web::Json<EnableModuleRequest>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let claims = match authenticate_and_authorize(&http_req, &data.db_pool, "module", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let module_name = path.into_inner();
    let user_id = uuid::Uuid::parse_str(&claims.sub).ok();

    let mut module_service = ModuleService::new(data.db_pool.clone());
    
    match module_service.enable_module(&module_name, user_id, body.notes.clone()).await {
        Ok(_) => {
            // Log audit event
            let audit_service = AuditService::new(data.db_pool.clone());
            audit_service.log_event(
                user_id,
                None,
                "module.enabled",
                Some("module"),
                Some("enable"),
                http_req.connection_info().peer_addr().map(|s| s.to_string()).as_deref(),
                None,
                true,
                None,
                Some(serde_json::json!({ "module": module_name, "notes": body.notes })),
            ).await.ok();

            HttpResponse::Ok().json(ModuleResponse {
                status: "SUCCESS".to_string(),
                message: format!("Module {} enabled", module_name),
            })
        }
        Err(e) => {
            eprintln!("Error enabling module: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to enable module"
            }))
        }
    }
}

/// Disable a module
#[utoipa::path(
    post,
    path = "/modules/{name}/disable",
    responses((status = 200, body = ModuleResponse))
)]
pub async fn disable_module(
    path: web::Path<String>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let claims = match authenticate_and_authorize(&http_req, &data.db_pool, "module", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let module_name = path.into_inner();
    let user_id = uuid::Uuid::parse_str(&claims.sub).ok();

    let mut module_service = ModuleService::new(data.db_pool.clone());
    
    match module_service.disable_module(&module_name, user_id).await {
        Ok(_) => {
            // Log audit event
            let audit_service = AuditService::new(data.db_pool.clone());
            audit_service.log_event(
                user_id,
                None,
                "module.disabled",
                Some("module"),
                Some("disable"),
                http_req.connection_info().peer_addr().map(|s| s.to_string()).as_deref(),
                None,
                true,
                None,
                Some(serde_json::json!({ "module": module_name })),
            ).await.ok();

            HttpResponse::Ok().json(ModuleResponse {
                status: "SUCCESS".to_string(),
                message: format!("Module {} disabled", module_name),
            })
        }
        Err(e) => {
            eprintln!("Error disabling module: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to disable module"
            }))
        }
    }
}

/// Check if a module is enabled (public endpoint for SDKs)
#[utoipa::path(
    get,
    path = "/modules/{name}/status",
    responses((status = 200, body = serde_json::Value))
)]
pub async fn get_module_status(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let module_name = path.into_inner();
    let mut module_service = ModuleService::new(data.db_pool.clone());
    
    match module_service.is_module_enabled(&module_name).await {
        Ok(enabled) => {
            HttpResponse::Ok().json(serde_json::json!({
                "module": module_name,
                "enabled": enabled
            }))
        }
        Err(e) => {
            eprintln!("Error checking module status: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to check module status"
            }))
        }
    }
}

// ============================================================================
// NEW ENDPOINTS: Enhanced Module System
// ============================================================================

/// Get modules by regulation
#[utoipa::path(
    get,
    path = "/modules/by-regulation/{regulation}",
    responses((status = 200, body = ModulesListResponse))
)]
pub async fn get_modules_by_regulation(
    path: web::Path<String>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "module", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let regulation = path.into_inner();
    let module_service = ModuleService::new(data.db_pool.clone());
    
    match module_service.get_modules_by_regulation(&regulation).await {
        Ok(modules) => {
            let module_infos: Vec<ModuleInfo> = modules.into_iter().map(|module| {
                ModuleInfo {
                    id: module.id,
                    name: module.name,
                    display_name: module.display_name,
                    description: module.description,
                    category: module.category,
                    enabled: false, // Check separately if needed
                    requires_license: module.requires_license,
                }
            }).collect();

            HttpResponse::Ok().json(ModulesListResponse {
                modules: module_infos,
            })
        }
        Err(e) => {
            eprintln!("Error getting modules by regulation: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get modules by regulation"
            }))
        }
    }
}

/// Get module configuration schema
#[utoipa::path(
    get,
    path = "/modules/{name}/config-schema",
    responses((status = 200, body = serde_json::Value))
)]
pub async fn get_module_config_schema(
    path: web::Path<String>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "module", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let module_name = path.into_inner();
    
    // Get configuration schema from database
    let schema: Option<serde_json::Value> = sqlx::query_scalar(
        "SELECT configuration_schema FROM modules WHERE name = $1"
    )
    .bind(&module_name)
    .fetch_optional(&data.db_pool)
    .await
    .unwrap_or(None);

    if let Some(s) = schema {
        HttpResponse::Ok().json(s)
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": "Module not found or no configuration schema"
        }))
    }
}

/// Get company module configuration
#[utoipa::path(
    get,
    path = "/companies/{company_id}/modules/{module_name}/config",
    responses((status = 200, body = serde_json::Value))
)]
pub async fn get_company_module_config(
    path: web::Path<(String, String)>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "module", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let (company_id_str, module_name) = path.into_inner();
    let company_id = match Uuid::parse_str(&company_id_str) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid company ID"
            }));
        }
    };

    let module_service = ModuleService::new(data.db_pool.clone());
    
    match module_service.get_company_module_config(company_id, &module_name).await {
        Ok(Some(config)) => {
            HttpResponse::Ok().json(config)
        }
        Ok(None) => {
            HttpResponse::Ok().json(serde_json::json!({}))
        }
        Err(e) => {
            eprintln!("Error getting company module config: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get module configuration"
            }))
        }
    }
}

/// Set company module configuration
#[utoipa::path(
    post,
    path = "/companies/{company_id}/modules/{module_name}/config",
    request_body = serde_json::Value,
    responses((status = 200, body = ModuleResponse))
)]
pub async fn set_company_module_config(
    path: web::Path<(String, String)>,
    body: web::Json<serde_json::Value>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let claims = match authenticate_and_authorize(&http_req, &data.db_pool, "module", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let (company_id_str, module_name) = path.into_inner();
    let company_id = match Uuid::parse_str(&company_id_str) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid company ID"
            }));
        }
    };

    let user_id = uuid::Uuid::parse_str(&claims.sub).ok();
    let module_service = ModuleService::new(data.db_pool.clone());
    
    match module_service.set_company_module_config(company_id, &module_name, body.into_inner(), user_id).await {
        Ok(_) => {
            // Log audit event
            let audit_service = AuditService::new(data.db_pool.clone());
            audit_service.log_event(
                user_id,
                None,
                "module.config.updated",
                Some("module"),
                Some("update_config"),
                http_req.connection_info().peer_addr().map(|s| s.to_string()).as_deref(),
                None,
                true,
                None,
                Some(serde_json::json!({ 
                    "company_id": company_id_str,
                    "module": module_name 
                })),
            ).await.ok();

            HttpResponse::Ok().json(ModuleResponse {
                status: "SUCCESS".to_string(),
                message: format!("Module {} configuration updated", module_name),
            })
        }
        Err(e) => {
            eprintln!("Error setting company module config: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to set module configuration"
            }))
        }
    }
}

