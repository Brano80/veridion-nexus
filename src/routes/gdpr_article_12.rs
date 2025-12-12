// GDPR Article 12 - Transparent Information API Routes

use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::api_state::AppState;
use crate::modules::gdpr::article_12_transparent_information::{
    GDPRArticle12Module, CreatePrivacyNoticeRequest, UpdatePrivacyNoticeRequest,
};
use crate::security::{AuthService, extract_claims, RbacService, require_permission, AuditService, Claims};
use uuid::Uuid;
use sqlx;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PrivacyNoticeResponse {
    pub id: Uuid,
    pub company_id: Uuid,
    pub language_code: String,
    pub notice_type: String,
    pub content: String,
    pub version: i32,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct PrivacyNoticesListResponse {
    pub notices: Vec<PrivacyNoticeResponse>,
}

/// Helper function to authenticate and authorize
async fn authenticate_and_authorize(
    http_req: &HttpRequest,
    db_pool: &sqlx::PgPool,
    resource: &str,
    action: &str,
) -> Result<(Claims, Uuid), HttpResponse> {
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

    // Extract company_id from claims or query parameter
    let company_id = Uuid::parse_str(&claims.sub).unwrap_or_else(|_| Uuid::nil());

    Ok((claims, company_id))
}

/// List all privacy notices for a company
#[utoipa::path(
    get,
    path = "/modules/gdpr-article-12/notices",
    params(
        ("notice_type" = Option<String>, Query, description = "Filter by notice type"),
        ("language_code" = Option<String>, Query, description = "Filter by language code"),
        ("status" = Option<String>, Query, description = "Filter by status")
    ),
    responses((status = 200, body = PrivacyNoticesListResponse))
)]
pub async fn list_privacy_notices(
    http_req: HttpRequest,
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (_claims, company_id) = match authenticate_and_authorize(&http_req, &data.db_pool, "gdpr_article_12", "read").await {
        Ok((claims, cid)) => (claims, cid),
        Err(resp) => return resp,
    };

    let module = GDPRArticle12Module::new(data.db_pool.clone());
    
    let notice_type = query.get("notice_type").map(|s| s.as_str());
    let language_code = query.get("language_code").map(|s| s.as_str());
    let status = query.get("status").map(|s| s.as_str());

    match module.list_privacy_notices(company_id, notice_type, language_code, status).await {
        Ok(notices) => {
            let response: Vec<PrivacyNoticeResponse> = notices.into_iter().map(|n| {
                PrivacyNoticeResponse {
                    id: n.id,
                    company_id: n.company_id,
                    language_code: n.language_code,
                    notice_type: n.notice_type,
                    content: n.content,
                    version: n.version,
                    published_at: n.published_at,
                    status: n.status,
                    created_at: n.created_at,
                    updated_at: n.updated_at,
                }
            }).collect();

            HttpResponse::Ok().json(PrivacyNoticesListResponse {
                notices: response,
            })
        }
        Err(e) => {
            eprintln!("Error listing privacy notices: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to list privacy notices"
            }))
        }
    }
}

/// Create a privacy notice
#[utoipa::path(
    post,
    path = "/modules/gdpr-article-12/notices",
    request_body = CreatePrivacyNoticeRequest,
    responses((status = 201, body = PrivacyNoticeResponse))
)]
pub async fn create_privacy_notice(
    http_req: HttpRequest,
    request: web::Json<CreatePrivacyNoticeRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (_claims, company_id) = match authenticate_and_authorize(&http_req, &data.db_pool, "gdpr_article_12", "write").await {
        Ok((claims, cid)) => (claims, cid),
        Err(resp) => return resp,
    };

    let module = GDPRArticle12Module::new(data.db_pool.clone());

    match module.create_privacy_notice(company_id, request.into_inner()).await {
        Ok(notice) => {
            let response = PrivacyNoticeResponse {
                id: notice.id,
                company_id: notice.company_id,
                language_code: notice.language_code,
                notice_type: notice.notice_type,
                content: notice.content,
                version: notice.version,
                published_at: notice.published_at,
                status: notice.status,
                created_at: notice.created_at,
                updated_at: notice.updated_at,
            };

            // Log audit event
            let audit_service = AuditService::new(data.db_pool.clone());
            let user_id = http_req.headers().get("x-user-id")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| Uuid::parse_str(s).ok());
            let ip_addr = http_req.connection_info().peer_addr().map(|s| s.to_string());
            audit_service.log_event(
                user_id,
                None,
                "privacy_notice.created",
                Some("gdpr_article_12"),
                Some("create"),
                ip_addr.as_deref(),
                None,
                true,
                None,
                Some(serde_json::json!({ "notice_id": response.id })),
            ).await.ok();

            HttpResponse::Created().json(response)
        }
        Err(e) => {
            eprintln!("Error creating privacy notice: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create privacy notice"
            }))
        }
    }
}

/// Get a privacy notice by ID
#[utoipa::path(
    get,
    path = "/modules/gdpr-article-12/notices/{id}",
    responses((status = 200, body = PrivacyNoticeResponse))
)]
pub async fn get_privacy_notice(
    http_req: HttpRequest,
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (_claims, company_id) = match authenticate_and_authorize(&http_req, &data.db_pool, "gdpr_article_12", "read").await {
        Ok((claims, cid)) => (claims, cid),
        Err(resp) => return resp,
    };

    let notice_id = path.into_inner();
    let module = GDPRArticle12Module::new(data.db_pool.clone());

    match module.get_privacy_notice(company_id, notice_id).await {
        Ok(Some(notice)) => {
            let response = PrivacyNoticeResponse {
                id: notice.id,
                company_id: notice.company_id,
                language_code: notice.language_code,
                notice_type: notice.notice_type,
                content: notice.content,
                version: notice.version,
                published_at: notice.published_at,
                status: notice.status,
                created_at: notice.created_at,
                updated_at: notice.updated_at,
            };

            HttpResponse::Ok().json(response)
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Privacy notice not found"
            }))
        }
        Err(e) => {
            eprintln!("Error getting privacy notice: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get privacy notice"
            }))
        }
    }
}

/// Update a privacy notice
#[utoipa::path(
    put,
    path = "/modules/gdpr-article-12/notices/{id}",
    request_body = UpdatePrivacyNoticeRequest,
    responses((status = 200, body = PrivacyNoticeResponse))
)]
pub async fn update_privacy_notice(
    http_req: HttpRequest,
    path: web::Path<Uuid>,
    request: web::Json<UpdatePrivacyNoticeRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (_claims, company_id) = match authenticate_and_authorize(&http_req, &data.db_pool, "gdpr_article_12", "write").await {
        Ok((claims, cid)) => (claims, cid),
        Err(resp) => return resp,
    };

    let notice_id = path.into_inner();
    let module = GDPRArticle12Module::new(data.db_pool.clone());

    match module.update_privacy_notice(company_id, notice_id, request.into_inner()).await {
        Ok(notice) => {
            let response = PrivacyNoticeResponse {
                id: notice.id,
                company_id: notice.company_id,
                language_code: notice.language_code,
                notice_type: notice.notice_type,
                content: notice.content,
                version: notice.version,
                published_at: notice.published_at,
                status: notice.status,
                created_at: notice.created_at,
                updated_at: notice.updated_at,
            };

            // Log audit event
            let audit_service = AuditService::new(data.db_pool.clone());
            let user_id = http_req.headers().get("x-user-id")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| Uuid::parse_str(s).ok());
            let ip_addr = http_req.connection_info().peer_addr().map(|s| s.to_string());
            audit_service.log_event(
                user_id,
                None,
                "privacy_notice.updated",
                Some("gdpr_article_12"),
                Some("update"),
                ip_addr.as_deref(),
                None,
                true,
                None,
                Some(serde_json::json!({ "notice_id": response.id })),
            ).await.ok();

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Error updating privacy notice: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update privacy notice"
            }))
        }
    }
}

/// Publish a privacy notice
#[utoipa::path(
    post,
    path = "/modules/gdpr-article-12/notices/{id}/publish",
    responses((status = 200, body = PrivacyNoticeResponse))
)]
pub async fn publish_privacy_notice(
    http_req: HttpRequest,
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
) -> impl Responder {
    let (_claims, company_id) = match authenticate_and_authorize(&http_req, &data.db_pool, "gdpr_article_12", "write").await {
        Ok((claims, cid)) => (claims, cid),
        Err(resp) => return resp,
    };

    let notice_id = path.into_inner();
    let module = GDPRArticle12Module::new(data.db_pool.clone());

    match module.publish_privacy_notice(company_id, notice_id).await {
        Ok(notice) => {
            let response = PrivacyNoticeResponse {
                id: notice.id,
                company_id: notice.company_id,
                language_code: notice.language_code,
                notice_type: notice.notice_type,
                content: notice.content,
                version: notice.version,
                published_at: notice.published_at,
                status: notice.status,
                created_at: notice.created_at,
                updated_at: notice.updated_at,
            };

            // Log audit event
            let audit_service = AuditService::new(data.db_pool.clone());
            let user_id = http_req.headers().get("x-user-id")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| Uuid::parse_str(s).ok());
            let ip_addr = http_req.connection_info().peer_addr().map(|s| s.to_string());
            audit_service.log_event(
                user_id,
                None,
                "privacy_notice.published",
                Some("gdpr_article_12"),
                Some("publish"),
                ip_addr.as_deref(),
                None,
                true,
                None,
                Some(serde_json::json!({ "notice_id": response.id })),
            ).await.ok();

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Error publishing privacy notice: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to publish privacy notice"
            }))
        }
    }
}

/// Get privacy notice templates
#[utoipa::path(
    get,
    path = "/modules/gdpr-article-12/templates",
    responses((status = 200, body = serde_json::Value))
)]
pub async fn get_privacy_notice_templates(
    _http_req: HttpRequest,
    _data: web::Data<AppState>,
) -> impl Responder {
    let templates = GDPRArticle12Module::get_templates();
    HttpResponse::Ok().json(templates)
}

