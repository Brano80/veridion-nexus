use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::api_state::AppState;
use crate::security::{AuthService, extract_claims, RbacService, require_permission, AuditService, Claims};
pub mod auth;
pub mod api_keys;
pub mod modules;

/// Helper function to authenticate and authorize user
async fn authenticate_and_authorize(
    http_req: &HttpRequest,
    db_pool: &sqlx::PgPool,
    resource: &str,
    action: &str,
) -> Result<Claims, HttpResponse> {
    let auth_service = AuthService::new().unwrap();
    let claims = extract_claims(http_req, &auth_service)?;

    let rbac = RbacService::new(db_pool.clone());
    let audit_service = AuditService::new(db_pool.clone());
    
    // Check permission
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
use crate::core::annex_iv::ComplianceRecord;
use crate::compliance_models::*;
use crate::models::db_models::*;
use crate::models::db_models::{ConsentRecordDb, DpiaRecordDb, RetentionPolicyDb, RetentionAssignmentDb, MonitoringEventDb, SystemHealthStatusDb, WebhookEndpointDb, WebhookDeliveryDb};
use crate::integration::webhooks::WebhookService;
use utoipa::ToSchema;
use actix_web::http::header::ContentDisposition;
use std::fs;
use chrono::{Local, Utc, DateTime};
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LogRequest {
    pub agent_id: String,
    pub action: String,
    pub payload: String,
    pub target_region: Option<String>,
    /// Transparency flag (EU AI Act Article 13)
    #[schema(example = true)]
    pub user_notified: Option<bool>,
    /// Timestamp when user was notified
    #[schema(example = "2024-01-15 14:30:05")]
    pub notification_timestamp: Option<String>,
    /// User ID for data subject rights (GDPR)
    #[schema(example = "user-123")]
    pub user_id: Option<String>,
    /// Whether this action requires human oversight
    #[schema(example = false)]
    pub requires_human_oversight: Option<bool>,
    // ========== GREEN AI TELEMETRY (EU AI Act Article 40) ==========
    /// Inference time in milliseconds
    #[schema(example = 150)]
    pub inference_time_ms: Option<u64>,
    /// GPU power rating in watts
    #[schema(example = 250.0)]
    pub gpu_power_rating_watts: Option<f64>,
    /// CPU power rating in watts
    #[schema(example = 100.0)]
    pub cpu_power_rating_watts: Option<f64>,
    /// Pre-calculated energy estimate in kWh (optional, will be calculated if not provided)
    #[schema(example = 0.000104)]
    pub energy_estimate_kwh: Option<f64>,
    /// Pre-calculated carbon footprint in grams (optional, will be calculated if not provided)
    #[schema(example = 0.0494)]
    pub carbon_grams: Option<f64>,
    /// System ID for AI-BOM tracking
    #[schema(example = "AI-SYSTEM-001")]
    pub system_id: Option<String>,
    /// Model name
    #[schema(example = "gpt-4")]
    pub model_name: Option<String>,
    /// Model version
    #[schema(example = "v1.0")]
    pub model_version: Option<String>,
    /// Hardware type: GPU, CPU, TPU, EDGE
    #[schema(example = "GPU")]
    pub hardware_type: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LogResponse {
    pub status: String,
    pub seal_id: String,
    pub tx_id: String,
    /// Risk level assessed
    #[schema(example = "MEDIUM")]
    pub risk_level: Option<String>,
    /// Human oversight status
    #[schema(example = "PENDING")]
    pub human_oversight_status: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ShredRequest {
    pub seal_id: String,
}

// 1. LOG ACTION (Updated with Priority 1 features)
#[utoipa::path(
    post,
    path = "/log_action",
    request_body = LogRequest,
    responses((status = 200, body = LogResponse), (status = 403, body = LogResponse))
)]
pub async fn log_action(
    req: web::Json<LogRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "compliance", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    // A. KILL SWITCH CHECK
    if let Ok(true) = data.is_locked_down().await {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "status": "SYSTEM_LOCKDOWN",
            "reason": "Agent Identity Revoked"
        }));
    }

    // A.1. CONSENT CHECK (GDPR Article 6, 7) - if user_id is provided
    if let Some(ref user_id) = req.user_id {
        let has_consent = check_consent(&data.db_pool, user_id, "PROCESSING").await
            .unwrap_or(false);
        
        if !has_consent {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "status": "CONSENT_REQUIRED",
                "reason": "User has not granted consent for data processing",
                "user_id": user_id
            }));
        }
    }

    // B. SOVEREIGN LOCK
    let is_violation = req.target_region.as_deref() == Some("US") 
        || req.target_region.as_deref() == Some("CN")
        || req.target_region.as_deref() == Some("RU");
    
    let status = if is_violation { "BLOCKED (SOVEREIGNTY)" } else { "COMPLIANT" };

    // C. RISK ASSESSMENT (EU AI Act Article 9)
    let risk_level = assess_risk(&req.action, &req.payload, is_violation);
    let risk_factors = vec![
        if is_violation { "Sovereignty violation attempt" } else { "Standard operation" }.to_string(),
        if req.requires_human_oversight.unwrap_or(false) { "High-risk action" } else { "Low-risk action" }.to_string(),
    ];
    let mitigation_actions = if is_violation {
        vec!["Action blocked".to_string()]
    } else {
        vec!["Sovereign lock active".to_string(), "Encryption enabled".to_string()]
    };

    // D. PRIVACY BRIDGE (Signicat)
    let log_hash = crate::core::privacy_bridge::hash_payload(&req.payload);
    let seal_result = data.signicat.request_seal(&log_hash).await;
    let seal_id = seal_result.unwrap_or_else(|e| format!("ERROR: {}", e));

    // E. CRYPTO-SHREDDER
    let encrypted_log = data.key_store.log_event(&req.payload);
    
    // Store wrapped DEK in database for persistence
    if let Some(wrapped_dek) = data.key_store.get_wrapped_dek(&encrypted_log.log_id) {
        let _ = sqlx::query(
            "INSERT INTO encrypted_log_keys (log_id, wrapped_dek) VALUES ($1, $2)
             ON CONFLICT (log_id) DO NOTHING"
        )
        .bind(&encrypted_log.log_id)
        .bind(&wrapped_dek)
        .execute(&data.db_pool)
        .await;
    }

    // F. HUMAN OVERSIGHT (EU AI Act Article 14)
    let human_oversight_status = if req.requires_human_oversight.unwrap_or(false) {
        // Insert into database
        let _ = sqlx::query(
            "INSERT INTO human_oversight (seal_id, status) VALUES ($1, 'PENDING')
             ON CONFLICT (seal_id) DO UPDATE SET status = 'PENDING', updated_at = CURRENT_TIMESTAMP"
        )
        .bind(&seal_id)
        .execute(&data.db_pool)
        .await;
        Some("PENDING".to_string())
    } else {
        None
    };

    // G. STORE COMPLIANCE RECORD IN DATABASE
    let now = Utc::now();
    let notification_ts = req.notification_timestamp.as_ref()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
        .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));

    let record_id = Uuid::new_v4();
    if let Err(e) = sqlx::query(
        "INSERT INTO compliance_records (
            id, timestamp, agent_id, action_summary, seal_id, status,
            user_notified, notification_timestamp, human_oversight_status,
            risk_level, user_id, tx_id, payload_hash
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"
    )
    .bind(record_id)
    .bind(now)
    .bind(&req.agent_id)
    .bind(format!("{}: {}", req.agent_id, req.action))
    .bind(&seal_id)
    .bind(&status)
    .bind(req.user_notified)
    .bind(notification_ts)
    .bind(&human_oversight_status)
    .bind(&risk_level)
    .bind(&req.user_id)
    .bind(&encrypted_log.log_id)
    .bind(&log_hash)
    .execute(&data.db_pool)
    .await
    {
        eprintln!("Error storing compliance record: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to store compliance record"
        }));
    }

    // Store risk assessment in database
    let risk_assessment_id = Uuid::new_v4();
    let risk_factors_json = serde_json::to_value(&risk_factors).unwrap_or(serde_json::json!([]));
    let mitigation_actions_json = serde_json::to_value(&mitigation_actions).unwrap_or(serde_json::json!([]));
    
    let _ = sqlx::query(
        "INSERT INTO risk_assessments (id, seal_id, risk_level, risk_factors, mitigation_actions, assessed_at)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (seal_id) DO UPDATE SET
         risk_level = EXCLUDED.risk_level,
         risk_factors = EXCLUDED.risk_factors,
         mitigation_actions = EXCLUDED.mitigation_actions,
         assessed_at = EXCLUDED.assessed_at"
    )
    .bind(risk_assessment_id)
    .bind(&seal_id)
    .bind(&risk_level)
    .bind(risk_factors_json)
    .bind(mitigation_actions_json)
    .bind(now)
    .execute(&data.db_pool)
    .await;

    // Index user data for GDPR compliance
    if let Some(ref user_id) = req.user_id {
        let _ = sqlx::query(
            "INSERT INTO user_data_index (user_id, seal_id) VALUES ($1, $2)
             ON CONFLICT (user_id, seal_id) DO NOTHING"
        )
        .bind(user_id)
        .bind(&seal_id)
        .execute(&data.db_pool)
        .await;
    }

    // Store energy telemetry (EU AI Act Article 40)
    if req.inference_time_ms.is_some() || req.energy_estimate_kwh.is_some() {
        let energy_kwh = if let Some(energy) = req.energy_estimate_kwh {
            energy
        } else if let Some(inference_ms) = req.inference_time_ms {
            // Calculate energy: (GPU + CPU power) * time_in_hours / 1000
            let gpu_power = req.gpu_power_rating_watts.unwrap_or(250.0);
            let cpu_power = req.cpu_power_rating_watts.unwrap_or(0.0);
            let total_power = gpu_power + cpu_power;
            let time_hours = inference_ms as f64 / 1000.0 / 3600.0;
            (total_power * time_hours) / 1000.0
        } else {
            0.0
        };

        let carbon_grams = if let Some(carbon) = req.carbon_grams {
            carbon
        } else {
            // EU average grid carbon intensity: 475 g CO2/kWh
            energy_kwh * 475.0
        };

        let _ = sqlx::query(
            "INSERT INTO ai_energy_telemetry (
                seal_id, agent_id, system_id, inference_time_ms,
                gpu_power_rating_watts, cpu_power_rating_watts,
                energy_estimate_kwh, carbon_grams,
                model_name, model_version, hardware_type
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        )
        .bind(&seal_id)
        .bind(&req.agent_id)
        .bind(&req.system_id)
        .bind(req.inference_time_ms.map(|t| t as i64))
        .bind(req.gpu_power_rating_watts)
        .bind(req.cpu_power_rating_watts)
        .bind(energy_kwh)
        .bind(carbon_grams)
        .bind(&req.model_name)
        .bind(&req.model_version)
        .bind(&req.hardware_type)
        .execute(&data.db_pool)
        .await;
    }

    println!("Log: {} | Status: {} | Risk: {}", req.action, status, risk_level);

    // Trigger webhook event for compliance action
    let webhook_data = serde_json::json!({
        "seal_id": seal_id,
        "agent_id": req.agent_id,
        "action": req.action,
        "status": status,
        "risk_level": risk_level,
        "human_oversight_status": human_oversight_status,
        "is_violation": is_violation,
        "timestamp": now.to_rfc3339(),
    });
    trigger_webhook_event(&data.db_pool, "compliance.action", webhook_data).await;

    if is_violation {
        HttpResponse::Forbidden().json(LogResponse {
            status: status.to_string(),
            seal_id: "N/A (Connection Refused)".to_string(),
            tx_id: "0000".to_string(),
            risk_level: Some(risk_level),
            human_oversight_status,
        })
    } else {
        HttpResponse::Ok().json(LogResponse {
            status: status.to_string(),
            seal_id,
            tx_id: encrypted_log.log_id,
            risk_level: Some(risk_level),
            human_oversight_status,
        })
    }
}

/// Risk assessment function (EU AI Act Article 9)
fn assess_risk(action: &str, payload: &str, is_violation: bool) -> String {
    if is_violation {
        return "HIGH".to_string();
    }
    
    // Simple risk assessment logic
    let high_risk_keywords = ["credit", "loan", "medical", "diagnosis", "criminal"];
    let action_lower = action.to_lowercase();
    let payload_lower = payload.to_lowercase();
    
    if high_risk_keywords.iter().any(|kw| action_lower.contains(kw) || payload_lower.contains(kw)) {
        "HIGH".to_string()
    } else if action_lower.contains("transaction") || action_lower.contains("payment") {
        "MEDIUM".to_string()
    } else {
        "LOW".to_string()
    }
}

// 2. GET LOGS (with pagination)
#[utoipa::path(
    get,
    path = "/logs",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default: 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default: 100, max: 1000)")
    )
)]
pub async fn get_logs(
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "compliance", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let page = query
        .get("page")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1)
        .max(1);
    let limit = query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100)
        .min(1000)
        .max(1);
    let offset = (page - 1) * limit;

    // Get total count
    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM compliance_records")
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or(0);

    match sqlx::query_as::<_, ComplianceRecordDb>(
        "SELECT * FROM compliance_records ORDER BY timestamp DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&data.db_pool)
    .await
    {
        Ok(records) => {
            let compliance_records: Vec<ComplianceRecord> = records.into_iter().map(|r| r.into()).collect();
            HttpResponse::Ok().json(serde_json::json!({
                "data": compliance_records,
                "pagination": {
                    "page": page,
                    "limit": limit,
                    "total": total_count,
                    "total_pages": (total_count + limit - 1) / limit
                }
            }))
        }
        Err(e) => {
            eprintln!("Error fetching logs: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch logs"
            }))
        }
    }
}

// 3. CRYPTO-SHREDDER
#[utoipa::path(
    post, 
    path = "/shred_data", 
    request_body = ShredRequest
)]
pub async fn shred_data(
    req: web::Json<ShredRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "compliance", "delete").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    // Get tx_id from compliance record
    let tx_id_result: Result<Option<String>, _> = sqlx::query_scalar(
        "SELECT tx_id FROM compliance_records WHERE seal_id = $1"
    )
    .bind(&req.seal_id)
    .fetch_optional(&data.db_pool)
    .await;

    let tx_id = match tx_id_result {
        Ok(Some(id)) => id,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({"status": "NOT_FOUND"})),
        Err(e) => {
            eprintln!("Error fetching tx_id: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Database error"}));
        }
    };

    // Shred the key in key_store
    data.key_store.shred_key(&tx_id);

    // Mark key as shredded in database
    let _ = sqlx::query(
        "UPDATE encrypted_log_keys SET shredded_at = CURRENT_TIMESTAMP WHERE log_id = $1"
    )
    .bind(&tx_id)
    .execute(&data.db_pool)
    .await;

    // Update compliance record status
    let result = sqlx::query(
        "UPDATE compliance_records 
         SET action_summary = '[GDPR PURGED] Data Cryptographically Erased',
             status = 'ERASED (Art. 17)'
         WHERE seal_id = $1"
    )
    .bind(&req.seal_id)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(rows) if rows.rows_affected() > 0 => {
            println!("🗑️ Shredded record: {}", req.seal_id);
            
            // Trigger webhook event for GDPR erasure
            let webhook_data = serde_json::json!({
                "seal_id": req.seal_id,
                "tx_id": tx_id,
                "timestamp": Utc::now().to_rfc3339(),
            });
            trigger_webhook_event(&data.db_pool, "gdpr.erased", webhook_data).await;
            
            HttpResponse::Ok().json(serde_json::json!({"status": "SUCCESS"}))
        }
        _ => HttpResponse::NotFound().json(serde_json::json!({"status": "NOT_FOUND"}))
    }
}

// 4. DOWNLOAD REPORT
#[utoipa::path(get, path = "/download_report")]
pub async fn download_report(data: web::Data<AppState>) -> impl Responder {
    match sqlx::query_as::<_, ComplianceRecordDb>(
        "SELECT * FROM compliance_records ORDER BY timestamp DESC"
    )
    .fetch_all(&data.db_pool)
    .await
    {
        Ok(records) => {
            let compliance_records: Vec<ComplianceRecord> = records.into_iter().map(|r| r.into()).collect();
            let _ = crate::core::annex_iv::generate_report(&compliance_records, "server_report.pdf");
            if let Ok(bytes) = fs::read("server_report.pdf") {
                HttpResponse::Ok().content_type("application/pdf")
                    .append_header(ContentDisposition::attachment("Veridion_Annex_IV.pdf")).body(bytes)
            } else { 
                HttpResponse::InternalServerError().finish() 
            }
        }
        Err(e) => {
            eprintln!("Error fetching records for report: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// 5. REVOKE ACCESS
#[utoipa::path(post, path = "/revoke_access")]
pub async fn revoke_access(data: web::Data<AppState>) -> impl Responder {
    match data.set_locked_down(true).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "SUCCESS"})),
        Err(e) => {
            eprintln!("Error setting lockdown: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to set lockdown"
            }))
        }
    }
}

// ========== PRIORITY 1: DATA SUBJECT RIGHTS (GDPR Articles 15-22) ==========

// 6. DATA SUBJECT ACCESS (GDPR Article 15)
#[utoipa::path(
    get,
    path = "/data_subject/{user_id}/access",
    responses((status = 200, body = DataSubjectAccessResponse))
)]
pub async fn data_subject_access(
    path: web::Path<String>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION - Critical: data_subject.read
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "data_subject", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let user_id = path.into_inner();
    
    // Get all records for this user from database
    match sqlx::query_as::<_, ComplianceRecordDb>(
        "SELECT cr.* FROM compliance_records cr
         INNER JOIN user_data_index udi ON cr.seal_id = udi.seal_id
         WHERE udi.user_id = $1
         ORDER BY cr.timestamp DESC"
    )
    .bind(&user_id)
    .fetch_all(&data.db_pool)
    .await
    {
        Ok(records) => {
            let user_records: Vec<DataSubjectRecord> = records.into_iter().map(|r| {
                DataSubjectRecord {
                    timestamp: r.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                    action_summary: r.action_summary,
                    seal_id: r.seal_id,
                    status: r.status,
                    risk_level: r.risk_level,
                }
            }).collect();
            
            let response = DataSubjectAccessResponse {
                records: user_records,
                format: "json".to_string(),
                exported_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Error fetching user data: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch user data"
            }))
        }
    }
}

// 7. DATA SUBJECT EXPORT (GDPR Article 20 - Data Portability)
#[utoipa::path(
    get,
    path = "/data_subject/{user_id}/export",
    responses((status = 200, body = DataSubjectAccessResponse))
)]
pub async fn data_subject_export(
    path: web::Path<String>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION - Critical: data_subject.export
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "data_subject", "export").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    // Same as access, but could be extended with different format options
    // Reuse the access logic - we already have auth, so just call it
    // Note: We need to clone path since it's moved in the call
    let user_id = path.into_inner();
    
    // Get all records for this user from database
    match sqlx::query_as::<_, ComplianceRecordDb>(
        "SELECT cr.* FROM compliance_records cr
         INNER JOIN user_data_index udi ON cr.seal_id = udi.seal_id
         WHERE udi.user_id = $1
         ORDER BY cr.timestamp DESC"
    )
    .bind(&user_id)
    .fetch_all(&data.db_pool)
    .await
    {
        Ok(records) => {
            let user_records: Vec<DataSubjectRecord> = records.into_iter().map(|r| {
                DataSubjectRecord {
                    timestamp: r.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                    action_summary: r.action_summary,
                    seal_id: r.seal_id,
                    status: r.status,
                    risk_level: r.risk_level,
                }
            }).collect();
            
            let response = DataSubjectAccessResponse {
                records: user_records,
                format: "json".to_string(),
                exported_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Error fetching user data: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch user data"
            }))
        }
    }
}

// 8. DATA SUBJECT RECTIFICATION (GDPR Article 16)
#[utoipa::path(
    put,
    path = "/data_subject/{user_id}/rectify",
    request_body = DataSubjectRectificationRequest,
    responses((status = 200, body = serde_json::Value))
)]
pub async fn data_subject_rectify(
    path: web::Path<String>,
    req: web::Json<DataSubjectRectificationRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION - Critical: data_subject.rectify
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "data_subject", "rectify").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let user_id = path.into_inner();
    
    if user_id != req.user_id {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "User ID mismatch"
        }));
    }
    
    // Update record in database
    let result = sqlx::query(
        "UPDATE compliance_records 
         SET action_summary = $1, status = 'RECTIFIED (Art. 16)'
         WHERE seal_id = $2 AND user_id = $3"
    )
    .bind(format!("[RECTIFIED] {}", req.corrected_data))
    .bind(&req.seal_id)
    .bind(&user_id)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(rows) if rows.rows_affected() > 0 => {
            println!("📝 Rectified record: {} for user: {}", req.seal_id, user_id);
            HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "message": "Data rectified successfully"
            }))
        }
        _ => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Record not found or access denied"
        }))
    }
}

// ========== PRIORITY 1: HUMAN OVERSIGHT (EU AI Act Article 14) ==========

// 9. REQUIRE HUMAN OVERSIGHT
#[utoipa::path(
    post,
    path = "/action/{seal_id}/require_approval",
    request_body = HumanOversightRequest,
    responses((status = 200, body = serde_json::Value))
)]
pub async fn require_human_oversight(
    path: web::Path<String>,
    req: web::Json<HumanOversightRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let seal_id = path.into_inner();
    
    if seal_id != req.seal_id {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Seal ID mismatch"
        }));
    }
    
    // Insert or update human oversight in database
    let result = sqlx::query(
        "INSERT INTO human_oversight (seal_id, status) VALUES ($1, 'PENDING')
         ON CONFLICT (seal_id) DO UPDATE SET status = 'PENDING', updated_at = CURRENT_TIMESTAMP"
    )
    .bind(&seal_id)
    .execute(&data.db_pool)
    .await;

    // Update compliance record
    let _ = sqlx::query(
        "UPDATE compliance_records SET human_oversight_status = 'PENDING' WHERE seal_id = $1"
    )
    .bind(&seal_id)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(_) => {
            // Trigger webhook event for human oversight required
            let webhook_data = serde_json::json!({
                "seal_id": seal_id,
                "status": "PENDING",
                "timestamp": Utc::now().to_rfc3339(),
            });
            trigger_webhook_event(&data.db_pool, "human_oversight.required", webhook_data).await;
            
            HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "message": "Human oversight required",
                "seal_id": seal_id
            }))
        },
        Err(e) => {
            eprintln!("Error requiring human oversight: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to require human oversight"
            }))
        }
    }
}

// 10. APPROVE ACTION (Human Oversight)
#[utoipa::path(
    post,
    path = "/action/{seal_id}/approve",
    request_body = HumanOversightResponse,
    responses((status = 200, body = serde_json::Value))
)]
pub async fn approve_action(
    path: web::Path<String>,
    req: web::Json<HumanOversightResponse>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION - Critical: oversight.approve
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "oversight", "approve").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let seal_id = path.into_inner();
    
    // Update human oversight in database
    let result = sqlx::query(
        "UPDATE human_oversight 
         SET status = 'APPROVED', reviewer_id = $1, decided_at = CURRENT_TIMESTAMP, comments = $2
         WHERE seal_id = $3"
    )
    .bind(&req.reviewer_id)
    .bind(&req.comments)
    .bind(&seal_id)
    .execute(&data.db_pool)
    .await;

    // Update compliance record
    let _ = sqlx::query(
        "UPDATE compliance_records SET human_oversight_status = 'APPROVED' WHERE seal_id = $1"
    )
    .bind(&seal_id)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(rows) if rows.rows_affected() > 0 => {
            println!("✅ Action approved: {} by reviewer: {:?}", seal_id, req.reviewer_id);
            
            // Trigger webhook event for human oversight resolved
            let webhook_data = serde_json::json!({
                "seal_id": seal_id,
                "status": "APPROVED",
                "reviewer_id": req.reviewer_id,
                "comments": req.comments,
                "timestamp": Utc::now().to_rfc3339(),
            });
            trigger_webhook_event(&data.db_pool, "human_oversight.resolved", webhook_data).await;
            
            HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "message": "Action approved",
                "seal_id": seal_id
            }))
        }
        _ => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Human oversight record not found"
        }))
    }
}

// 11. REJECT ACTION (Human Oversight)
#[utoipa::path(
    post,
    path = "/action/{seal_id}/reject",
    request_body = HumanOversightResponse,
    responses((status = 200, body = serde_json::Value))
)]
pub async fn reject_action(
    path: web::Path<String>,
    req: web::Json<HumanOversightResponse>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION - Critical: oversight.approve
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "oversight", "approve").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let seal_id = path.into_inner();
    
    // Update human oversight in database
    let result = sqlx::query(
        "UPDATE human_oversight 
         SET status = 'REJECTED', reviewer_id = $1, decided_at = CURRENT_TIMESTAMP, comments = $2
         WHERE seal_id = $3"
    )
    .bind(&req.reviewer_id)
    .bind(&req.comments)
    .bind(&seal_id)
    .execute(&data.db_pool)
    .await;

    // Update compliance record
    let _ = sqlx::query(
        "UPDATE compliance_records 
         SET human_oversight_status = 'REJECTED', status = 'REJECTED (Human Oversight)'
         WHERE seal_id = $1"
    )
    .bind(&seal_id)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(rows) if rows.rows_affected() > 0 => {
            println!("❌ Action rejected: {} by reviewer: {:?}", seal_id, req.reviewer_id);
            
            // Trigger webhook event for human oversight resolved
            let webhook_data = serde_json::json!({
                "seal_id": seal_id,
                "status": "REJECTED",
                "reviewer_id": req.reviewer_id,
                "comments": req.comments,
                "timestamp": Utc::now().to_rfc3339(),
            });
            trigger_webhook_event(&data.db_pool, "human_oversight.resolved", webhook_data).await;
            
            HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "message": "Action rejected",
                "seal_id": seal_id
            }))
        }
        _ => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Human oversight record not found"
        }))
    }
}

// ========== PRIORITY 1: RISK ASSESSMENT (EU AI Act Article 9) ==========

// 12. GET RISK ASSESSMENT
#[utoipa::path(
    get,
    path = "/risk_assessment/{seal_id}",
    responses((status = 200, body = RiskAssessment))
)]
pub async fn get_risk_assessment(
    path: web::Path<String>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "risk", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let seal_id = path.into_inner();
    
    match sqlx::query_as::<_, RiskAssessmentDb>(
        "SELECT * FROM risk_assessments WHERE seal_id = $1"
    )
    .bind(&seal_id)
    .fetch_optional(&data.db_pool)
    .await
    {
        Ok(Some(risk_db)) => {
            let risk = RiskAssessment {
                risk_level: risk_db.risk_level,
                risk_factors: serde_json::from_value(risk_db.risk_factors).unwrap_or_default(),
                mitigation_actions: serde_json::from_value(risk_db.mitigation_actions).unwrap_or_default(),
                assessed_at: risk_db.assessed_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            HttpResponse::Ok().json(risk)
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Risk assessment not found"
        })),
        Err(e) => {
            eprintln!("Error fetching risk assessment: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch risk assessment"
            }))
        }
    }
}

// 13. GET ALL RISKS (with pagination)
#[utoipa::path(
    get,
    path = "/risks",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default: 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default: 100, max: 1000)")
    ),
    responses((status = 200, body = serde_json::Value))
)]
pub async fn get_all_risks(
    query: web::Query<std::collections::HashMap<String, String>>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "risk", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let page = query
        .get("page")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1)
        .max(1);
    let limit = query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100)
        .min(1000)
        .max(1);
    let offset = (page - 1) * limit;

    // Get total count
    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM risk_assessments")
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or(0);

    match sqlx::query_as::<_, RiskAssessmentDb>(
        "SELECT * FROM risk_assessments ORDER BY assessed_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&data.db_pool)
    .await
    {
        Ok(records) => {
            let all_risks: Vec<RiskAssessment> = records.into_iter().map(|r| {
                RiskAssessment {
                    risk_level: r.risk_level,
                    risk_factors: serde_json::from_value(r.risk_factors).unwrap_or_default(),
                    mitigation_actions: serde_json::from_value(r.mitigation_actions).unwrap_or_default(),
                    assessed_at: r.assessed_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                }
            }).collect();
            HttpResponse::Ok().json(serde_json::json!({
                "data": all_risks,
                "pagination": {
                    "page": page,
                    "limit": limit,
                    "total": total_count,
                    "total_pages": (total_count + limit - 1) / limit
                }
            }))
        }
        Err(e) => {
            eprintln!("Error fetching risks: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch risks"
            }))
        }
    }
}

// ========== PRIORITY 1: DATA BREACH (GDPR Articles 33-34) ==========

// 14. REPORT DATA BREACH
#[utoipa::path(
    post,
    path = "/breach_report",
    request_body = DataBreachReport,
    responses((status = 200, body = DataBreachResponse))
)]
pub async fn report_breach(
    req: web::Json<DataBreachReport>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION - Critical: breach.write
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "breach", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let breach_id = format!("BREACH-{}", Local::now().format("%Y%m%d-%H%M%S"));
    let breach_report = req.into_inner();
    let now = Utc::now();
    
    // Parse detected_at
    let detected_at = chrono::NaiveDateTime::parse_from_str(&breach_report.detected_at, "%Y-%m-%d %H:%M:%S")
        .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        .unwrap_or(now);
    
    // Simulate notification to authority (within 72 hours)
    let authority_notified_at = Some(now);
    
    // Simulate user notification (if high risk)
    let users_notified_at = if breach_report.breach_type == "DATA_LEAK" || breach_report.breach_type == "SYSTEM_COMPROMISE" {
        Some(now)
    } else {
        None
    };
    
    let affected_users_json = serde_json::to_value(&breach_report.affected_users).unwrap_or(serde_json::json!([]));
    
    // Store breach in database
    let result = sqlx::query(
        "INSERT INTO data_breaches (
            breach_id, description, breach_type, affected_users, detected_at,
            affected_records_count, status, authority_notified_at, users_notified_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(&breach_id)
    .bind(&breach_report.description)
    .bind(&breach_report.breach_type)
    .bind(affected_users_json)
    .bind(detected_at)
    .bind(breach_report.affected_records_count.map(|c| c as i32))
    .bind("REPORTED")
    .bind(authority_notified_at)
    .bind(users_notified_at)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(_) => {
            let response = DataBreachResponse {
                breach_id: breach_id.clone(),
                status: "REPORTED".to_string(),
                authority_notified_at: authority_notified_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                users_notified_at: users_notified_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            };
            
            println!("🚨 Data breach reported: {} | Type: {} | Affected: {} users", 
                breach_id, breach_report.breach_type, breach_report.affected_users.len());
            
            // Trigger webhook event for data breach
            let webhook_data = serde_json::json!({
                "breach_id": breach_id,
                "breach_type": breach_report.breach_type,
                "description": breach_report.description,
                "affected_users_count": breach_report.affected_users.len(),
                "detected_at": detected_at.to_rfc3339(),
                "authority_notified_at": authority_notified_at.map(|dt| dt.to_rfc3339()),
                "users_notified_at": users_notified_at.map(|dt| dt.to_rfc3339()),
            });
            trigger_webhook_event(&data.db_pool, "data_breach.detected", webhook_data).await;
            
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Error reporting breach: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to report breach"
            }))
        }
    }
}

// 15. GET ALL BREACHES (with pagination)
#[utoipa::path(
    get,
    path = "/breaches",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default: 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default: 100, max: 1000)")
    ),
    responses((status = 200, body = serde_json::Value))
)]
pub async fn get_breaches(
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION - Critical: breach.read
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "breach", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let page = query
        .get("page")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1)
        .max(1);
    let limit = query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100)
        .min(1000)
        .max(1);
    let offset = (page - 1) * limit;

    // Get total count
    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM data_breaches")
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or(0);

    match sqlx::query_as::<_, DataBreachDb>(
        "SELECT * FROM data_breaches ORDER BY detected_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&data.db_pool)
    .await
    {
        Ok(records) => {
            let breaches: Vec<DataBreachReport> = records.into_iter().map(|r| {
                DataBreachReport {
                    description: r.description,
                    breach_type: r.breach_type,
                    affected_users: serde_json::from_value(r.affected_users).unwrap_or_default(),
                    detected_at: r.detected_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    affected_records_count: r.affected_records_count.map(|c| c.max(0) as u32),
                }
            }).collect();
            HttpResponse::Ok().json(serde_json::json!({
                "data": breaches,
                "pagination": {
                    "page": page,
                    "limit": limit,
                    "total": total_count,
                    "total_pages": (total_count + limit - 1) / limit
                }
            }))
        }
        Err(e) => {
            eprintln!("Error fetching breaches: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch breaches"
            }))
        }
    }
}

// ========== PRIORITY 2: CONSENT MANAGEMENT (GDPR Articles 6, 7) ==========

// 16. GRANT CONSENT
#[utoipa::path(
    post,
    path = "/consent",
    request_body = ConsentRequest,
    responses((status = 200, body = ConsentResponse))
)]
pub async fn grant_consent(
    req: web::Json<ConsentRequest>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "consent", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let consent_req = req.into_inner();
    let now = Utc::now();
    
    // Parse expires_at if provided
    let expires_at = consent_req.expires_at.as_ref()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
        .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));
    
    // Get current version for this consent type
    let current_version: Option<i32> = sqlx::query_scalar(
        "SELECT MAX(version) FROM consent_records WHERE user_id = $1 AND consent_type = $2"
    )
    .bind(&consent_req.user_id)
    .bind(&consent_req.consent_type)
    .fetch_optional(&data.db_pool)
    .await
    .unwrap_or(None);
    
    let version = current_version.map(|v| v + 1).unwrap_or(1);
    
    // Withdraw any existing consent of this type
    let _ = sqlx::query(
        "UPDATE consent_records 
         SET granted = false, withdrawn_at = CURRENT_TIMESTAMP
         WHERE user_id = $1 AND consent_type = $2 AND granted = true AND withdrawn_at IS NULL"
    )
    .bind(&consent_req.user_id)
    .bind(&consent_req.consent_type)
    .execute(&data.db_pool)
    .await;
    
    // Insert new consent
    let consent_id = Uuid::new_v4();
    let result = sqlx::query(
        "INSERT INTO consent_records (
            id, user_id, consent_type, purpose, legal_basis, granted, granted_at,
            expires_at, consent_method, ip_address, user_agent, consent_text, version
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING id"
    )
    .bind(consent_id)
    .bind(&consent_req.user_id)
    .bind(&consent_req.consent_type)
    .bind(&consent_req.purpose)
    .bind(&consent_req.legal_basis)
    .bind(true)
    .bind(now)
    .bind(expires_at)
    .bind(&consent_req.consent_method)
    .bind(&consent_req.ip_address)
    .bind(&consent_req.user_agent)
    .bind(&consent_req.consent_text)
    .bind(version)
    .fetch_one(&data.db_pool)
    .await;

    match result {
        Ok(_) => {
            let user_id = consent_req.user_id.clone();
            let consent_type = consent_req.consent_type.clone();
            
            let response = ConsentResponse {
                consent_id: consent_id.to_string(),
                user_id: user_id.clone(),
                consent_type: consent_type.clone(),
                granted: true,
                granted_at: Some(now.format("%Y-%m-%d %H:%M:%S").to_string()),
                expires_at: expires_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                version,
            };
            
            println!("✅ Consent granted: {} for user: {}", consent_type, user_id);
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Error granting consent: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to grant consent"
            }))
        }
    }
}

// 17. WITHDRAW CONSENT
#[utoipa::path(
    post,
    path = "/consent/withdraw",
    request_body = WithdrawConsentRequest,
    responses((status = 200, body = serde_json::Value))
)]
pub async fn withdraw_consent(
    req: web::Json<WithdrawConsentRequest>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "consent", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let withdraw_req = req.into_inner();
    
    let result = if let Some(consent_type) = withdraw_req.consent_type {
        // Withdraw specific consent type
        sqlx::query(
            "UPDATE consent_records 
             SET granted = false, withdrawn_at = CURRENT_TIMESTAMP
             WHERE user_id = $1 AND consent_type = $2 AND granted = true AND withdrawn_at IS NULL"
        )
        .bind(&withdraw_req.user_id)
        .bind(&consent_type)
        .execute(&data.db_pool)
        .await
    } else {
        // Withdraw all consents for user
        sqlx::query(
            "UPDATE consent_records 
             SET granted = false, withdrawn_at = CURRENT_TIMESTAMP
             WHERE user_id = $1 AND granted = true AND withdrawn_at IS NULL"
        )
        .bind(&withdraw_req.user_id)
        .execute(&data.db_pool)
        .await
    };

    match result {
        Ok(rows) if rows.rows_affected() > 0 => {
            println!("❌ Consent withdrawn for user: {}", withdraw_req.user_id);
            HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "message": "Consent withdrawn successfully"
            }))
        }
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "No active consent found"
        })),
        Err(e) => {
            eprintln!("Error withdrawing consent: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to withdraw consent"
            }))
        }
    }
}

// 18. GET USER CONSENTS
#[utoipa::path(
    get,
    path = "/consent/{user_id}",
    responses((status = 200, body = UserConsentsResponse))
)]
pub async fn get_user_consents(
    path: web::Path<String>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "consent", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let user_id = path.into_inner();
    
    match sqlx::query_as::<_, ConsentRecordDb>(
        "SELECT * FROM consent_records 
         WHERE user_id = $1 
         ORDER BY created_at DESC"
    )
    .bind(&user_id)
    .fetch_all(&data.db_pool)
    .await
    {
        Ok(records) => {
            let consents: Vec<ConsentResponse> = records.into_iter().map(|r| {
                ConsentResponse {
                    consent_id: r.id.to_string(),
                    user_id: r.user_id,
                    consent_type: r.consent_type,
                    granted: r.granted && r.withdrawn_at.is_none(),
                    granted_at: r.granted_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                    expires_at: r.expires_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                    version: r.version,
                }
            }).collect();
            
            let response = UserConsentsResponse {
                user_id,
                consents,
            };
            
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Error fetching consents: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch consents"
            }))
        }
    }
}

// 19. CHECK CONSENT (Helper function for log_action)
pub async fn check_consent(
    db_pool: &sqlx::PgPool,
    user_id: &str,
    consent_type: &str,
) -> Result<bool, sqlx::Error> {
    let result: Option<bool> = sqlx::query_scalar(
        "SELECT granted FROM consent_records 
         WHERE user_id = $1 
           AND consent_type = $2 
           AND granted = true 
           AND withdrawn_at IS NULL
           AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
         ORDER BY version DESC
         LIMIT 1"
    )
    .bind(user_id)
    .bind(consent_type)
    .fetch_optional(db_pool)
    .await?;
    
    Ok(result.unwrap_or(false))
}

// ========== PRIORITY 2: DPIA TRACKING (GDPR Article 35) ==========

// 20. CREATE DPIA
#[utoipa::path(
    post,
    path = "/dpia",
    request_body = DpiaRequest,
    responses((status = 200, body = DpiaResponse))
)]
pub async fn create_dpia(
    req: web::Json<DpiaRequest>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "dpia", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let dpia_req = req.into_inner();
    let now = Utc::now();
    
    // Generate DPIA ID
    let dpia_id = format!("DPIA-{}-{:03}", now.format("%Y"), 
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM dpia_records WHERE dpia_id LIKE $1"
        )
        .bind(format!("DPIA-{}%", now.format("%Y")))
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or(0) + 1);
    
    // Determine if consultation is required (Article 36 - High risk)
    let consultation_required = dpia_req.risk_level == "HIGH" || 
        dpia_req.identified_risks.iter().any(|r| 
            r.to_lowercase().contains("discrimination") || 
            r.to_lowercase().contains("systematic")
        );
    
    let identified_risks_json = serde_json::to_value(&dpia_req.identified_risks).unwrap_or(serde_json::json!([]));
    let mitigation_measures_json = serde_json::to_value(&dpia_req.mitigation_measures).unwrap_or(serde_json::json!([]));
    
    let dpia_uuid = Uuid::new_v4();
    let result = sqlx::query(
        "INSERT INTO dpia_records (
            id, dpia_id, activity_name, description, legal_basis,
            data_categories, data_subject_categories, processing_purposes,
            risk_level, identified_risks, mitigation_measures, residual_risks,
            consultation_required, status, created_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING dpia_id"
    )
    .bind(dpia_uuid)
    .bind(&dpia_id)
    .bind(&dpia_req.activity_name)
    .bind(&dpia_req.description)
    .bind(&dpia_req.legal_basis)
    .bind(&dpia_req.data_categories)
    .bind(&dpia_req.data_subject_categories)
    .bind(&dpia_req.processing_purposes)
    .bind(&dpia_req.risk_level)
    .bind(identified_risks_json)
    .bind(mitigation_measures_json)
    .bind(serde_json::json!([]))
    .bind(consultation_required)
    .bind("DRAFT")
    .bind(&dpia_req.created_by)
    .fetch_one(&data.db_pool)
    .await;

    match result {
        Ok(_) => {
            let response = DpiaResponse {
                dpia_id: dpia_id.clone(),
                activity_name: dpia_req.activity_name,
                status: "DRAFT".to_string(),
                risk_level: dpia_req.risk_level,
                consultation_required,
                created_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            
            println!("📋 DPIA created: {} for activity: {}", dpia_id, response.activity_name);
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Error creating DPIA: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create DPIA"
            }))
        }
    }
}

// 21. UPDATE DPIA
#[utoipa::path(
    put,
    path = "/dpia/{dpia_id}",
    request_body = UpdateDpiaRequest,
    responses((status = 200, body = DpiaResponse))
)]
pub async fn update_dpia(
    path: web::Path<String>,
    req: web::Json<UpdateDpiaRequest>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "dpia", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let dpia_id = path.into_inner();
    let update_req = req.into_inner();
    
    // Clone values before using them
    let status_clone = update_req.status.clone();
    let reviewed_by_clone = update_req.reviewed_by.clone();
    let next_review_date_parsed = update_req.next_review_date.as_ref()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
        .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));
    let residual_risks_json = update_req.residual_risks.as_ref()
        .map(|r| serde_json::to_value(r).unwrap_or(serde_json::json!([])));
    
    // For simplicity, use a fixed query structure
    let result = if status_clone.is_some() && reviewed_by_clone.is_some() {
        sqlx::query(
            "UPDATE dpia_records 
             SET status = $1, risk_level = COALESCE($2, risk_level),
                 residual_risks = COALESCE($3, residual_risks),
                 reviewed_by = $4, reviewed_at = CURRENT_TIMESTAMP,
                 approval_date = CASE WHEN $1 = 'APPROVED' THEN CURRENT_TIMESTAMP ELSE approval_date END,
                 next_review_date = COALESCE($5, next_review_date),
                 updated_at = CURRENT_TIMESTAMP
             WHERE dpia_id = $6"
        )
        .bind(status_clone.unwrap())
        .bind(update_req.risk_level)
        .bind(residual_risks_json)
        .bind(reviewed_by_clone)
        .bind(next_review_date_parsed)
        .bind(&dpia_id)
        .execute(&data.db_pool)
        .await
    } else {
        // Simplified update for other cases
        sqlx::query(
            "UPDATE dpia_records 
             SET status = COALESCE($1, status),
                 risk_level = COALESCE($2, risk_level),
                 residual_risks = COALESCE($3, residual_risks),
                 next_review_date = COALESCE($4, next_review_date),
                 updated_at = CURRENT_TIMESTAMP
             WHERE dpia_id = $5"
        )
        .bind(status_clone)
        .bind(update_req.risk_level)
        .bind(residual_risks_json)
        .bind(next_review_date_parsed)
        .bind(&dpia_id)
        .execute(&data.db_pool)
        .await
    };

    match result {
        Ok(rows) if rows.rows_affected() > 0 => {
            // Fetch updated DPIA
            match sqlx::query_as::<_, DpiaRecordDb>(
                "SELECT * FROM dpia_records WHERE dpia_id = $1"
            )
            .bind(&dpia_id)
            .fetch_optional(&data.db_pool)
            .await
            {
                Ok(Some(dpia)) => {
                    let response = DpiaResponse {
                        dpia_id: dpia.dpia_id,
                        activity_name: dpia.activity_name,
                        status: dpia.status,
                        risk_level: dpia.risk_level,
                        consultation_required: dpia.consultation_required,
                        created_at: dpia.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    };
                    HttpResponse::Ok().json(response)
                }
                _ => HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to fetch updated DPIA"
                }))
            }
        }
        _ => HttpResponse::NotFound().json(serde_json::json!({
            "error": "DPIA not found"
        }))
    }
}

// 22. GET ALL DPIAs (with pagination)
#[utoipa::path(
    get,
    path = "/dpias",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default: 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default: 100, max: 1000)")
    ),
    responses((status = 200, body = serde_json::Value))
)]
pub async fn get_all_dpias(
    query: web::Query<std::collections::HashMap<String, String>>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "dpia", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let page = query
        .get("page")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1)
        .max(1);
    let limit = query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100)
        .min(1000)
        .max(1);
    let offset = (page - 1) * limit;

    // Get total count
    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dpia_records")
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or(0);

    match sqlx::query_as::<_, DpiaRecordDb>(
        "SELECT * FROM dpia_records ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&data.db_pool)
    .await
    {
        Ok(records) => {
            let dpias: Vec<DpiaResponse> = records.into_iter().map(|r| {
                DpiaResponse {
                    dpia_id: r.dpia_id,
                    activity_name: r.activity_name,
                    status: r.status,
                    risk_level: r.risk_level,
                    consultation_required: r.consultation_required,
                    created_at: r.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                }
            }).collect();
            
            HttpResponse::Ok().json(serde_json::json!({
                "data": dpias,
                "pagination": {
                    "page": page,
                    "limit": limit,
                    "total": total_count,
                    "total_pages": (total_count + limit - 1) / limit
                }
            }))
        }
        Err(e) => {
            eprintln!("Error fetching DPIAs: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch DPIAs"
            }))
        }
    }
}

// ========== PRIORITY 2: RETENTION PERIOD AUTOMATION (GDPR Article 5(1)(e)) ==========

// 23. CREATE RETENTION POLICY
#[utoipa::path(
    post,
    path = "/retention/policy",
    request_body = RetentionPolicyRequest,
    responses((status = 200, body = RetentionPolicyResponse))
)]
pub async fn create_retention_policy(
    req: web::Json<RetentionPolicyRequest>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "retention", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let policy_req = req.into_inner();
    let now = Utc::now();
    
    let policy_id = Uuid::new_v4();
    let result = sqlx::query(
        "INSERT INTO retention_policies (
            id, policy_name, data_category, retention_period_days,
            legal_basis, description, auto_delete, notification_days_before
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id"
    )
    .bind(policy_id)
    .bind(&policy_req.policy_name)
    .bind(&policy_req.data_category)
    .bind(policy_req.retention_period_days)
    .bind(&policy_req.legal_basis)
    .bind(&policy_req.description)
    .bind(policy_req.auto_delete.unwrap_or(true))
    .bind(policy_req.notification_days_before)
    .fetch_one(&data.db_pool)
    .await;

    match result {
        Ok(_) => {
            let response = RetentionPolicyResponse {
                policy_id: policy_id.to_string(),
                policy_name: policy_req.policy_name,
                data_category: policy_req.data_category,
                retention_period_days: policy_req.retention_period_days,
                auto_delete: policy_req.auto_delete.unwrap_or(true),
                created_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            
            println!("📅 Retention policy created: {} for category: {}", response.policy_name, response.data_category);
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Error creating retention policy: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create retention policy"
            }))
        }
    }
}

// 25. GET EXPIRING RECORDS
#[utoipa::path(
    get,
    path = "/retention/expiring",
    responses((status = 200, body = ExpiringRecordsResponse))
)]
#[allow(dead_code)]
pub async fn get_expiring_records(
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
) -> impl Responder {
    // Get days parameter (default: 30 days)
    let days: i32 = query.get("days")
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    
    let now = Utc::now();
    let expiration_threshold = now + chrono::Duration::days(days as i64);
    
    match sqlx::query_as::<_, RetentionAssignmentDb>(
        "SELECT * FROM retention_assignments 
         WHERE deleted_at IS NULL 
           AND expires_at <= $1 
           AND deletion_status = 'PENDING'
         ORDER BY expires_at ASC
         LIMIT 100"
    )
    .bind(expiration_threshold)
    .fetch_all(&data.db_pool)
    .await
    {
        Ok(records) => {
            let expiring_records: Vec<RetentionTrackingResponse> = records.into_iter().map(|r| {
                let days_until = (r.expires_at - now).num_days();
                RetentionTrackingResponse {
                    record_type: r.record_type,
                    record_id: r.record_id,
                    created_at: r.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    expires_at: r.expires_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    deletion_status: r.deletion_status,
                    days_until_expiration: Some(days_until),
                }
            }).collect();
            
            let total_count = expiring_records.len() as i64;
            let response = ExpiringRecordsResponse {
                expiring_records,
                total_count,
            };
            
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Error fetching expiring records: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch expiring records"
            }))
        }
    }
}

// 26. EXECUTE RETENTION DELETION (Manual trigger for automated deletion)
#[utoipa::path(
    post,
    path = "/retention/execute",
    responses((status = 200, body = serde_json::Value))
)]
#[allow(dead_code)]
pub async fn execute_retention_deletion(
    data: web::Data<AppState>,
) -> impl Responder {
    let now = Utc::now();
    
    // Find records that should be deleted
    let expired_records: Vec<RetentionAssignmentDb> = sqlx::query_as(
        "SELECT * FROM retention_assignments 
         WHERE deleted_at IS NULL 
           AND expires_at <= $1 
           AND deletion_status = 'PENDING'
           AND (record_type, record_id) NOT IN (
               SELECT record_type, record_id FROM retention_exemptions 
               WHERE (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
           )"
    )
    .bind(now)
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();
    
    let mut deleted_count = 0;
    let mut errors = Vec::new();
    
    for record in expired_records {
        // Mark as deleted in assignment
        let update_result = sqlx::query(
            "UPDATE retention_assignments 
             SET deleted_at = CURRENT_TIMESTAMP, deletion_status = 'DELETED'
             WHERE id = $1"
        )
        .bind(record.id)
        .execute(&data.db_pool)
        .await;
        
        if update_result.is_ok() {
            // Log deletion
            let _ = sqlx::query(
                "INSERT INTO retention_deletion_log (
                    record_type, record_id, policy_id, deletion_method, deleted_by, records_affected
                ) VALUES ($1, $2, $3, 'AUTO', 'system', 1)"
            )
            .bind(&record.record_type)
            .bind(&record.record_id)
            .bind(&record.policy_id)
            .execute(&data.db_pool)
            .await;
            
            // Actually delete the record based on type
            match record.record_type.as_str() {
                "COMPLIANCE_RECORD" => {
                    // Delete compliance record and related data
                    let _ = sqlx::query("DELETE FROM compliance_records WHERE seal_id = $1")
                        .bind(&record.record_id)
                        .execute(&data.db_pool)
                        .await;
                }
                "CONSENT" => {
                    let _ = sqlx::query("DELETE FROM consent_records WHERE id::text = $1")
                        .bind(&record.record_id)
                        .execute(&data.db_pool)
                        .await;
                }
                "DPIA" => {
                    let _ = sqlx::query("DELETE FROM dpia_records WHERE dpia_id = $1")
                        .bind(&record.record_id)
                        .execute(&data.db_pool)
                        .await;
                }
                "BREACH" => {
                    let _ = sqlx::query("DELETE FROM data_breaches WHERE breach_id = $1")
                        .bind(&record.record_id)
                        .execute(&data.db_pool)
                        .await;
                }
                _ => {
                    errors.push(format!("Unknown record type: {}", record.record_type));
                }
            }
            
            deleted_count += 1;
        } else {
            errors.push(format!("Failed to update tracking for record: {}", record.record_id));
        }
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "status": "SUCCESS",
        "deleted_count": deleted_count,
        "errors": errors
    }))
}

// ========== PRIORITY 2: RETENTION PERIOD AUTOMATION (GDPR Article 5(1)(e)) ==========

// 24. ASSIGN RETENTION POLICY TO RECORD
#[utoipa::path(
    post,
    path = "/retention/assign",
    request_body = AssignRetentionRequest,
    responses((status = 200, body = RetentionStatusResponse))
)]
pub async fn assign_retention_policy(
    req: web::Json<AssignRetentionRequest>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "retention", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let assign_req = req.into_inner();
    
    // Get policy details
    let policy_result: Result<Option<RetentionPolicyDb>, _> = sqlx::query_as(
        "SELECT * FROM retention_policies WHERE policy_name = $1"
    )
    .bind(&assign_req.policy_name)
    .fetch_optional(&data.db_pool)
    .await;
    
    let policy = match policy_result {
        Ok(Some(p)) => p,
        Ok(None) => return HttpResponse::NotFound().json(serde_json::json!({
            "error": "Retention policy not found"
        })),
        Err(e) => {
            eprintln!("Error fetching policy: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch retention policy"
            }));
        }
    };
    
    // Calculate expiration date
    let created_at = Utc::now();
    let expires_at = created_at + chrono::Duration::days(policy.retention_period_days as i64);
    
    let assignment_id = Uuid::new_v4();
    let result = sqlx::query(
        "INSERT INTO retention_assignments (
            id, record_type, record_id, policy_id, created_at, expires_at, deletion_status
        ) VALUES ($1, $2, $3, $4, $5, $6, 'PENDING')
        ON CONFLICT (record_type, record_id) 
        DO UPDATE SET policy_id = EXCLUDED.policy_id, expires_at = EXCLUDED.expires_at, deletion_status = 'PENDING'
        RETURNING expires_at"
    )
    .bind(assignment_id)
    .bind(&assign_req.record_type)
    .bind(&assign_req.record_id)
    .bind(policy.id)
    .bind(created_at)
    .bind(expires_at)
    .fetch_one(&data.db_pool)
    .await;

    match result {
        Ok(row) => {
            let expires_at_db: chrono::DateTime<chrono::Utc> = sqlx::Row::get(&row, 0);
            let days_until = (expires_at_db - Utc::now()).num_days();
            
            // Update record's retention_expires_at if applicable
            match assign_req.record_type.as_str() {
                "COMPLIANCE_RECORD" => {
                    let _ = sqlx::query(
                        "UPDATE compliance_records SET retention_expires_at = $1 WHERE seal_id = $2"
                    )
                    .bind(expires_at_db)
                    .bind(&assign_req.record_id)
                    .execute(&data.db_pool)
                    .await;
                }
                "CONSENT_RECORD" => {
                    let _ = sqlx::query(
                        "UPDATE consent_records SET retention_expires_at = $1 WHERE id::text = $2"
                    )
                    .bind(expires_at_db)
                    .bind(&assign_req.record_id)
                    .execute(&data.db_pool)
                    .await;
                }
                _ => {}
            }
            
            let policy_name_clone = policy.policy_name.clone();
            let record_id_clone = assign_req.record_id.clone();
            let record_type_clone = assign_req.record_type.clone();
            
            let response = RetentionStatusResponse {
                record_type: record_type_clone.clone(),
                record_id: record_id_clone.clone(),
                policy_name: policy_name_clone.clone(),
                expires_at: expires_at_db.format("%Y-%m-%d %H:%M:%S").to_string(),
                days_until_expiration: days_until,
                deletion_status: "PENDING".to_string(),
            };
            
            println!("📅 Retention policy assigned: {} to {} ({})", policy_name_clone, record_id_clone, record_type_clone);
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Error assigning retention policy: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to assign retention policy"
            }))
        }
    }
}

// 25. GET RETENTION STATUS
#[utoipa::path(
    get,
    path = "/retention/status/{record_type}/{record_id}",
    responses((status = 200, body = RetentionStatusResponse))
)]
pub async fn get_retention_status(
    path: web::Path<(String, String)>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "retention", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let (record_type, record_id) = path.into_inner();
    
    match sqlx::query_as::<_, RetentionAssignmentDb>(
        "SELECT ra.* FROM retention_assignments ra
         INNER JOIN retention_policies rp ON ra.policy_id = rp.id
         WHERE ra.record_type = $1 AND ra.record_id = $2 AND ra.deleted_at IS NULL
         ORDER BY ra.created_at DESC
         LIMIT 1"
    )
    .bind(&record_type)
    .bind(&record_id)
    .fetch_optional(&data.db_pool)
    .await
    {
        Ok(Some(assignment)) => {
            // Get policy name
            let policy_name: String = sqlx::query_scalar(
                "SELECT policy_name FROM retention_policies WHERE id = $1"
            )
            .bind(assignment.policy_id)
            .fetch_one(&data.db_pool)
            .await
            .unwrap_or_else(|_| "UNKNOWN".to_string());
            
            let days_until = (assignment.expires_at - Utc::now()).num_days();
            
            let response = RetentionStatusResponse {
                record_type,
                record_id,
                policy_name,
                expires_at: assignment.expires_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                days_until_expiration: days_until,
                deletion_status: assignment.deletion_status,
            };
            
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Retention assignment not found"
        })),
        Err(e) => {
            eprintln!("Error fetching retention status: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch retention status"
            }))
        }
    }
}

// 26. GET ALL RETENTION POLICIES
#[utoipa::path(
    get,
    path = "/retention/policies",
    responses((status = 200, body = RetentionPoliciesResponse))
)]
pub async fn get_all_retention_policies(
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "retention", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    match sqlx::query_as::<_, RetentionPolicyDb>(
        "SELECT * FROM retention_policies ORDER BY created_at DESC"
    )
    .fetch_all(&data.db_pool)
    .await
    {
        Ok(records) => {
            let policies: Vec<RetentionPolicyResponse> = records.into_iter().map(|r| {
                RetentionPolicyResponse {
                    policy_id: r.id.to_string(),
                    policy_name: r.policy_name,
                    data_category: r.data_category,
                    retention_period_days: r.retention_period_days,
                    auto_delete: r.auto_delete,
                    created_at: r.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                }
            }).collect();
            
            let response = RetentionPoliciesResponse { policies };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Error fetching retention policies: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch retention policies"
            }))
        }
    }
}

// 27. EXECUTE RETENTION DELETION (Background job - can be called manually)
#[utoipa::path(
    post,
    path = "/retention/execute_deletions",
    responses((status = 200, body = serde_json::Value))
)]
pub async fn execute_retention_deletions(
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION - Critical: admin only
    let auth_service = AuthService::new().unwrap();
    let claims = match extract_claims(&http_req, &auth_service) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    
    // Only admin can execute retention deletions
    if !claims.has_role("admin") {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Forbidden",
            "message": "Only administrators can execute retention deletions"
        }));
    }
    // Find all expired assignments that haven't been deleted
    let expired_assignments: Vec<RetentionAssignmentDb> = sqlx::query_as(
        "SELECT ra.* FROM retention_assignments ra
         INNER JOIN retention_policies rp ON ra.policy_id = rp.id
         WHERE ra.expires_at <= CURRENT_TIMESTAMP 
           AND ra.deleted_at IS NULL 
           AND ra.deletion_status = 'PENDING'
           AND rp.auto_delete = true
         LIMIT 100"
    )
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();
    
    let mut deleted_count = 0;
    let mut errors = Vec::new();
    
    for assignment in expired_assignments {
        // Mark as scheduled
        let _ = sqlx::query(
            "UPDATE retention_assignments SET deletion_status = 'SCHEDULED' WHERE id = $1"
        )
        .bind(assignment.id)
        .execute(&data.db_pool)
        .await;
        
        // Delete based on record type
        let deletion_result = match assignment.record_type.as_str() {
            "COMPLIANCE_RECORD" => {
                // Use crypto-shredder for compliance records
                let tx_id_result: Result<Option<String>, _> = sqlx::query_scalar(
                    "SELECT tx_id FROM compliance_records WHERE seal_id = $1"
                )
                .bind(&assignment.record_id)
                .fetch_optional(&data.db_pool)
                .await;
                
                if let Ok(Some(tx_id)) = tx_id_result {
                    // Shred the key
                    data.key_store.shred_key(&tx_id);
                    
                    // Mark key as shredded
                    let _ = sqlx::query(
                        "UPDATE encrypted_log_keys SET shredded_at = CURRENT_TIMESTAMP WHERE log_id = $1"
                    )
                    .bind(&tx_id)
                    .execute(&data.db_pool)
                    .await;
                    
                    // Update compliance record
                    sqlx::query(
                        "UPDATE compliance_records 
                         SET action_summary = '[RETENTION EXPIRED] Data Automatically Deleted',
                             status = 'DELETED (Retention Period)'
                         WHERE seal_id = $1"
                    )
                    .bind(&assignment.record_id)
                    .execute(&data.db_pool)
                    .await
                } else {
                    Ok(sqlx::postgres::PgQueryResult::default())
                }
            }
            "CONSENT_RECORD" => {
                // Mark consent as withdrawn
                sqlx::query(
                    "UPDATE consent_records 
                     SET granted = false, withdrawn_at = CURRENT_TIMESTAMP
                     WHERE id::text = $1"
                )
                .bind(&assignment.record_id)
                .execute(&data.db_pool)
                .await
            }
            _ => {
                // For other types, just mark as deleted
                Ok(sqlx::postgres::PgQueryResult::default())
            }
        };
        
        match deletion_result {
            Ok(_) => {
                // Mark assignment as deleted
                let _ = sqlx::query(
                    "UPDATE retention_assignments 
                     SET deleted_at = CURRENT_TIMESTAMP, deletion_status = 'DELETED'
                     WHERE id = $1"
                )
                .bind(assignment.id)
                .execute(&data.db_pool)
                .await;
                
                // Log deletion
                let policy_name: String = sqlx::query_scalar(
                    "SELECT policy_name FROM retention_policies WHERE id = $1"
                )
                .bind(assignment.policy_id)
                .fetch_one(&data.db_pool)
                .await
                .unwrap_or_else(|_| "UNKNOWN".to_string());
                
                let _ = sqlx::query(
                    "INSERT INTO retention_deletion_log (
                        assignment_id, record_type, record_id, policy_name, deletion_method
                    ) VALUES ($1, $2, $3, $4, 'AUTO')"
                )
                .bind(assignment.id)
                .bind(&assignment.record_type)
                .bind(&assignment.record_id)
                .bind(&policy_name)
                .execute(&data.db_pool)
                .await;
                
                deleted_count += 1;
            }
            Err(e) => {
                errors.push(format!("Failed to delete {} {}: {}", assignment.record_type, assignment.record_id, e));
            }
        }
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "status": "SUCCESS",
        "deleted_count": deleted_count,
        "errors": errors
    }))
}

// ========== PRIORITY 2: POST-MARKET MONITORING (EU AI Act Article 72) ==========

// 28. CREATE MONITORING EVENT
#[utoipa::path(
    post,
    path = "/monitoring/event",
    request_body = MonitoringEventRequest,
    responses((status = 200, body = MonitoringEventResponse))
)]
pub async fn create_monitoring_event(
    req: web::Json<MonitoringEventRequest>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "monitoring", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let event_req = req.into_inner();
    let now = Utc::now();
    
    // Generate event ID
    let event_id = format!("EVT-{}-{:06}", now.format("%Y%m%d"), 
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM monitoring_events WHERE event_id LIKE $1"
        )
        .bind(format!("EVT-{}%", now.format("%Y%m%d")))
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or(0) + 1);
    
    let detected_at = event_req.detected_at.as_ref()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
        .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        .unwrap_or(now);
    
    let affected_users_json = serde_json::to_value(event_req.affected_users.unwrap_or_default()).unwrap_or(serde_json::json!([]));
    let affected_count = affected_users_json.as_array().map(|a| a.len() as i32).unwrap_or(0);
    
    let event_uuid = Uuid::new_v4();
    let result = sqlx::query(
        "INSERT INTO monitoring_events (
            id, event_id, event_type, severity, system_id, system_version,
            description, affected_users, affected_records_count, detected_at,
            resolution_status
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'OPEN')
        RETURNING event_id"
    )
    .bind(event_uuid)
    .bind(&event_id)
    .bind(&event_req.event_type)
    .bind(&event_req.severity)
    .bind(&event_req.system_id)
    .bind(&event_req.system_version)
    .bind(&event_req.description)
    .bind(affected_users_json)
    .bind(affected_count)
    .bind(detected_at)
    .fetch_one(&data.db_pool)
    .await;

    match result {
        Ok(_) => {
            let response = MonitoringEventResponse {
                event_id: event_id.clone(),
                event_type: event_req.event_type,
                severity: event_req.severity,
                system_id: event_req.system_id,
                resolution_status: "OPEN".to_string(),
                detected_at: detected_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            
            println!("📊 Monitoring event created: {} for system: {}", event_id, response.system_id);
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Error creating monitoring event: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create monitoring event"
            }))
        }
    }
}

// 29. UPDATE EVENT RESOLUTION
#[utoipa::path(
    put,
    path = "/monitoring/event/{event_id}",
    request_body = UpdateEventResolutionRequest,
    responses((status = 200, body = MonitoringEventResponse))
)]
pub async fn update_event_resolution(
    path: web::Path<String>,
    req: web::Json<UpdateEventResolutionRequest>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "monitoring", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let event_id = path.into_inner();
    let update_req = req.into_inner();
    
    let resolved_at = if update_req.resolution_status == "RESOLVED" {
        Some(Utc::now())
    } else {
        None
    };
    
    // Check if event exists
    let _event_exists = match sqlx::query_as::<_, MonitoringEventDb>(
        "SELECT * FROM monitoring_events WHERE event_id = $1"
    )
    .bind(&event_id)
    .fetch_optional(&data.db_pool)
    .await {
        Ok(Some(_)) => {} // Event exists, continue
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Monitoring event not found"
            }));
        }
        Err(e) => {
            eprintln!("Error checking event existence: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error"
            }));
        }
    };

    // Update the event
    let update_result = sqlx::query(
        "UPDATE monitoring_events 
         SET resolution_status = $1, resolution_notes = $2,
             corrective_action_taken = $3, preventive_measures = $4,
             resolved_at = $5, updated_at = CURRENT_TIMESTAMP
         WHERE event_id = $6"
    )
    .bind(&update_req.resolution_status)
    .bind(&update_req.resolution_notes)
    .bind(&update_req.corrective_action_taken)
    .bind(&update_req.preventive_measures)
    .bind(resolved_at)
    .bind(&event_id)
    .execute(&data.db_pool)
    .await;

    // Check if update succeeded
    match update_result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error updating event: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update monitoring event"
            }));
        }
    }

    // Fetch updated event
    let updated_event = sqlx::query_as::<_, MonitoringEventDb>(
        "SELECT * FROM monitoring_events WHERE event_id = $1"
    )
    .bind(&event_id)
    .fetch_optional(&data.db_pool)
    .await;

    match updated_event {
        Ok(Some(event)) => {
            let response = MonitoringEventResponse {
                event_id: event.event_id,
                event_type: event.event_type,
                severity: event.severity,
                system_id: event.system_id,
                resolution_status: event.resolution_status,
                detected_at: event.detected_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Monitoring event not found"
        })),
        Err(e) => {
            eprintln!("Error updating event resolution: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update event resolution"
            }))
        }
    }
}

// 30. GET ALL MONITORING EVENTS (with pagination)
#[utoipa::path(
    get,
    path = "/monitoring/events",
    params(
        ("system_id" = Option<String>, Query, description = "Filter by system ID"),
        ("page" = Option<i64>, Query, description = "Page number (default: 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default: 100, max: 1000)")
    ),
    responses((status = 200, body = MonitoringEventsResponse))
)]
pub async fn get_all_monitoring_events(
    query: web::Query<std::collections::HashMap<String, String>>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "monitoring", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let system_id = query.get("system_id");
    let page = query
        .get("page")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1)
        .max(1);
    let limit = query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100)
        .min(1000)
        .max(1);
    let offset = (page - 1) * limit;
    
    // Get total count
    let total_count: i64 = if let Some(sid) = system_id {
        sqlx::query_scalar("SELECT COUNT(*) FROM monitoring_events WHERE system_id = $1")
            .bind(sid)
            .fetch_one(&data.db_pool)
            .await
            .unwrap_or(0)
    } else {
        sqlx::query_scalar("SELECT COUNT(*) FROM monitoring_events")
            .fetch_one(&data.db_pool)
            .await
            .unwrap_or(0)
    };
    
    let query_final = if let Some(sid) = system_id {
        sqlx::query_as::<_, MonitoringEventDb>(
            "SELECT * FROM monitoring_events WHERE system_id = $1 ORDER BY detected_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(sid)
        .bind(limit)
        .bind(offset)
        .fetch_all(&data.db_pool)
        .await
    } else {
        sqlx::query_as::<_, MonitoringEventDb>(
            "SELECT * FROM monitoring_events ORDER BY detected_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&data.db_pool)
        .await
    };

    match query_final {
        Ok(records) => {
            let events: Vec<MonitoringEventResponse> = records.into_iter().map(|r| {
                MonitoringEventResponse {
                    event_id: r.event_id,
                    event_type: r.event_type,
                    severity: r.severity,
                    system_id: r.system_id,
                    resolution_status: r.resolution_status,
                    detected_at: r.detected_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                }
            }).collect();
            
            let response = MonitoringEventsResponse {
                events,
                total_count,
            };
            
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Error fetching monitoring events: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch monitoring events"
            }))
        }
    }
}

// 31. GET SYSTEM HEALTH STATUS
#[utoipa::path(
    get,
    path = "/monitoring/health/{system_id}",
    responses((status = 200, body = SystemHealthStatusResponse))
)]
pub async fn get_system_health(
    path: web::Path<String>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "monitoring", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let system_id = path.into_inner();
    
    match sqlx::query_as::<_, SystemHealthStatusDb>(
        "SELECT * FROM system_health_status WHERE system_id = $1"
    )
    .bind(&system_id)
    .fetch_optional(&data.db_pool)
    .await
    {
        Ok(Some(health)) => {
            let response = SystemHealthStatusResponse {
                system_id: health.system_id,
                overall_status: health.overall_status,
                compliance_status: health.compliance_status,
                active_incidents_count: health.active_incidents_count,
                critical_incidents_count: health.critical_incidents_count,
                performance_score: health.performance_score,
                compliance_score: health.compliance_score,
                last_health_check: health.last_health_check.format("%Y-%m-%d %H:%M:%S").to_string(),
            };
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "System health status not found"
        })),
        Err(e) => {
            eprintln!("Error fetching system health: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch system health"
            }))
        }
    }
}

// ========== AI-BOM (CycloneDX) EXPORT ==========

// 32. EXPORT AI-BOM (CycloneDX format)
#[utoipa::path(
    get,
    path = "/ai_bom/{system_id}",
    params(
        ("system_id" = String, Path, description = "System ID to export"),
        ("format" = Option<String>, Query, description = "Export format (default: cyclonedx)")
    ),
    responses((status = 200, body = CycloneDxBom))
)]
pub async fn export_ai_bom(
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
) -> impl Responder {
    let system_id = path.into_inner();
    let format = query.get("format").map(|s| s.as_str()).unwrap_or("cyclonedx");

    if format != "cyclonedx" {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Only 'cyclonedx' format is currently supported"
        }));
    }

    // Fetch system inventory from database
    let inventory_result: Result<Option<serde_json::Value>, _> = sqlx::query_scalar(
        "SELECT row_to_json(ai_system_inventory.*) FROM ai_system_inventory WHERE system_id = $1"
    )
    .bind(&system_id)
    .fetch_optional(&data.db_pool)
    .await;

    let inventory = match inventory_result {
        Ok(Some(inv)) => inv,
        Ok(None) => {
            // If no inventory record, create a basic one from monitoring events
            let monitoring_result: Result<Option<MonitoringEventDb>, _> = sqlx::query_as(
                "SELECT * FROM monitoring_events WHERE system_id = $1 ORDER BY detected_at DESC LIMIT 1"
            )
            .bind(&system_id)
            .fetch_optional(&data.db_pool)
            .await;

            if let Ok(Some(event)) = monitoring_result {
                serde_json::json!({
                    "system_id": system_id,
                    "system_name": format!("AI System {}", system_id),
                    "system_version": event.system_version,
                    "system_type": "MODEL",
                    "description": event.description,
                })
            } else {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "System not found"
                }));
            }
        }
        Err(e) => {
            eprintln!("Error fetching system inventory: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch system inventory"
            }));
        }
    };

    // Fetch related DPIA if available
    let dpia_id: Option<String> = inventory.get("dpia_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut components = Vec::new();

    // Main component
    let mut properties = Vec::new();
    if let Some(risk_level) = inventory.get("risk_level").and_then(|v| v.as_str()) {
        properties.push(crate::compliance_models::CycloneDxProperty {
            name: "ai:risk_level".to_string(),
            value: risk_level.to_string(),
        });
    }
    if let Some(compliance_status) = inventory.get("compliance_status").and_then(|v| v.as_str()) {
        properties.push(crate::compliance_models::CycloneDxProperty {
            name: "ai:compliance_status".to_string(),
            value: compliance_status.to_string(),
        });
    }
    if let Some(dpia) = dpia_id {
        properties.push(crate::compliance_models::CycloneDxProperty {
            name: "ai:dpia_id".to_string(),
            value: dpia,
        });
    }

    components.push(crate::compliance_models::CycloneDxComponent {
        component_type: "application".to_string(),
        name: inventory.get("system_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&system_id)
            .to_string(),
        version: inventory.get("system_version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        description: inventory.get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        purl: inventory.get("source_url")
            .and_then(|v| v.as_str())
            .map(|s| format!("pkg:generic/{}@{}", system_id, s)),
        properties: Some(properties),
        bom_ref: Some(system_id.clone()),
    });

    // Add dependencies if available
    if let Some(deps) = inventory.get("dependencies").and_then(|v| v.as_array()) {
        for dep in deps {
            if let Some(dep_id) = dep.as_str() {
                components.push(crate::compliance_models::CycloneDxComponent {
                    component_type: "library".to_string(),
                    name: dep_id.to_string(),
                    version: None,
                    description: Some(format!("Dependency of {}", system_id)),
                    purl: None,
                    properties: None,
                    bom_ref: Some(dep_id.to_string()),
                });
            }
        }
    }

    let bom = crate::compliance_models::CycloneDxBom {
        bom_format: "CycloneDX".to_string(),
        spec_version: "1.5".to_string(),
        version: 1,
        metadata: crate::compliance_models::CycloneDxMetadata {
            timestamp: Utc::now().to_rfc3339(),
            tools: Some(vec![crate::compliance_models::CycloneDxTool {
                name: "Veridion Nexus".to_string(),
                version: "1.0.0".to_string(),
                vendor: Some("Veridion".to_string()),
            }]),
            properties: Some(vec![
                crate::compliance_models::CycloneDxProperty {
                    name: "ai:spec_version".to_string(),
                    value: "1.5".to_string(),
                },
            ]),
        },
        components,
    };

    HttpResponse::Ok().json(bom)
}

// 33. REGISTER AI SYSTEM INVENTORY
#[utoipa::path(
    post,
    path = "/ai_bom/inventory",
    request_body = AiSystemInventoryRequest,
    responses((status = 200, body = serde_json::Value))
)]
pub async fn register_ai_system(
    req: web::Json<AiSystemInventoryRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let inv_req = req.into_inner();
    
    let result = sqlx::query(
        "INSERT INTO ai_system_inventory (
            system_id, system_name, system_version, system_type,
            description, vendor, license, source_url, checksum_sha256,
            dependencies, training_data_info, risk_level, dpia_id,
            compliance_status
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 'REVIEW_REQUIRED')
        ON CONFLICT (system_id) DO UPDATE SET
            system_name = EXCLUDED.system_name,
            system_version = EXCLUDED.system_version,
            description = EXCLUDED.description,
            vendor = EXCLUDED.vendor,
            license = EXCLUDED.license,
            source_url = EXCLUDED.source_url,
            checksum_sha256 = EXCLUDED.checksum_sha256,
            dependencies = EXCLUDED.dependencies,
            training_data_info = EXCLUDED.training_data_info,
            risk_level = EXCLUDED.risk_level,
            dpia_id = EXCLUDED.dpia_id,
            updated_at = CURRENT_TIMESTAMP
        RETURNING system_id"
    )
    .bind(&inv_req.system_id)
    .bind(&inv_req.system_name)
    .bind(&inv_req.system_version)
    .bind(&inv_req.system_type)
    .bind(&inv_req.description)
    .bind(&inv_req.vendor)
    .bind(&inv_req.license)
    .bind(&inv_req.source_url)
    .bind(&inv_req.checksum_sha256)
    .bind(inv_req.dependencies.map(|d| serde_json::to_value(d).unwrap_or(serde_json::json!([]))))
    .bind(&inv_req.training_data_info)
    .bind(&inv_req.risk_level)
    .bind(&inv_req.dpia_id)
    .fetch_one(&data.db_pool)
    .await;

    match result {
        Ok(_) => {
            println!("📦 AI system registered: {}", inv_req.system_id);
            HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "system_id": inv_req.system_id
            }))
        }
        Err(e) => {
            eprintln!("Error registering AI system: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to register AI system"
            }))
        }
    }
}

// ========== WEBHOOK SUPPORT ==========

/// Helper function to trigger webhook events asynchronously
async fn trigger_webhook_event(
    db_pool: &sqlx::PgPool,
    event_type: &str,
    event_data: serde_json::Value,
) {
    // Find active webhooks subscribed to this event type
    let webhooks_result: Result<Vec<WebhookEndpointDb>, _> = sqlx::query_as(
        "SELECT * FROM webhook_endpoints 
         WHERE active = true 
         AND $1 = ANY(event_types)"
    )
    .bind(event_type)
    .fetch_all(db_pool)
    .await;

    let webhooks = match webhooks_result {
        Ok(wh) => wh,
        Err(e) => {
            eprintln!("Error fetching webhooks: {}", e);
            return;
        }
    };

    if webhooks.is_empty() {
        return;
    }

    let timestamp = Utc::now().to_rfc3339();

    // Create webhook event
    let webhook_event = WebhookEvent {
        event_type: event_type.to_string(),
        timestamp: timestamp.clone(),
        data: event_data,
        signature: None, // Will be added by webhook service
    };

    // Deliver to each webhook endpoint
    for webhook in webhooks {
        let endpoint_url = webhook.endpoint_url.clone();
        let secret_key = webhook.secret_key.clone();
        let timeout = webhook.timeout_seconds as u64;
        let max_retries = webhook.retry_count;
        let webhook_id = webhook.id;
        let event_type_clone = event_type.to_string();

        // Clone event for each webhook
        let event_clone = webhook_event.clone();

        // Spawn async task for webhook delivery
        let db_pool_clone = db_pool.clone();
        tokio::spawn(async move {
            let webhook_service = WebhookService::new();
            let (success, attempts, response) = webhook_service
                .deliver_with_retry(
                    &endpoint_url,
                    &event_clone,
                    &secret_key,
                    timeout,
                    max_retries,
                )
                .await;

            // Log delivery result
            let status = if success { "delivered" } else { "failed" };
            let response_code = if success {
                response.as_ref()
                    .and_then(|r| r.split(':').next())
                    .and_then(|s| s.split_whitespace().nth(1))
                    .and_then(|s| s.parse::<i32>().ok())
            } else {
                None
            };

            let _ = sqlx::query(
                "INSERT INTO webhook_deliveries (
                    webhook_endpoint_id, event_type, event_payload,
                    status, response_code, response_body, attempts, delivered_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
            )
            .bind(webhook_id)
            .bind(&event_type_clone)
            .bind(serde_json::to_value(&event_clone).unwrap_or(serde_json::json!({})))
            .bind(status)
            .bind(response_code)
            .bind(&response)
            .bind(attempts)
            .bind(if success { Some(Utc::now()) } else { None })
            .execute(&db_pool_clone)
            .await;
        });
    }
}

// 34. REGISTER WEBHOOK ENDPOINT
#[utoipa::path(
    post,
    path = "/webhooks",
    request_body = WebhookEndpointRequest,
    responses((status = 200, body = WebhookEndpointResponse))
)]
pub async fn register_webhook(
    req: web::Json<WebhookEndpointRequest>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "webhook", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let webhook_req = req.into_inner();

    // Generate secret key if not provided
    let secret_key = webhook_req.secret_key.unwrap_or_else(|| {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    });

    let webhook_id = Uuid::new_v4();
    let retry_count = webhook_req.retry_count.unwrap_or(3);
    let timeout_seconds = webhook_req.timeout_seconds.unwrap_or(30);

    let result = sqlx::query_as::<_, WebhookEndpointDb>(
        "INSERT INTO webhook_endpoints (
            id, endpoint_url, secret_key, event_types,
            active, retry_count, timeout_seconds
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *"
    )
    .bind(webhook_id)
    .bind(&webhook_req.endpoint_url)
    .bind(&secret_key)
    .bind(&webhook_req.event_types)
    .bind(true)
    .bind(retry_count)
    .bind(timeout_seconds)
    .fetch_one(&data.db_pool)
    .await;

    match result {
        Ok(webhook) => {
            println!("🔔 Webhook registered: {}", webhook.endpoint_url);
            HttpResponse::Ok().json(WebhookEndpointResponse {
                id: webhook.id.to_string(),
                endpoint_url: webhook.endpoint_url,
                event_types: webhook.event_types,
                active: webhook.active,
                retry_count: webhook.retry_count,
                timeout_seconds: webhook.timeout_seconds,
                created_at: webhook.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        }
        Err(e) => {
            eprintln!("Error registering webhook: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to register webhook"
            }))
        }
    }
}

// 35. LIST WEBHOOK ENDPOINTS (with pagination)
#[utoipa::path(
    get,
    path = "/webhooks",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default: 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default: 100, max: 1000)")
    ),
    responses((status = 200, body = WebhookEndpointsResponse))
)]
pub async fn list_webhooks(
    query: web::Query<std::collections::HashMap<String, String>>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "webhook", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let page = query
        .get("page")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(1)
        .max(1);
    let limit = query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100)
        .min(1000)
        .max(1);
    let offset = (page - 1) * limit;

    // Get total count
    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM webhook_endpoints")
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or(0);

    match sqlx::query_as::<_, WebhookEndpointDb>(
        "SELECT * FROM webhook_endpoints ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&data.db_pool)
    .await
    {
        Ok(webhooks) => {
            let endpoints: Vec<WebhookEndpointResponse> = webhooks.into_iter().map(|w| {
                WebhookEndpointResponse {
                    id: w.id.to_string(),
                    endpoint_url: w.endpoint_url,
                    event_types: w.event_types,
                    active: w.active,
                    retry_count: w.retry_count,
                    timeout_seconds: w.timeout_seconds,
                    created_at: w.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                }
            }).collect();

            HttpResponse::Ok().json(WebhookEndpointsResponse {
                endpoints,
                total_count,
            })
        }
        Err(e) => {
            eprintln!("Error fetching webhooks: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch webhooks"
            }))
        }
    }
}

// 36. UPDATE WEBHOOK ENDPOINT
#[utoipa::path(
    put,
    path = "/webhooks/{id}",
    request_body = UpdateWebhookEndpointRequest,
    responses((status = 200, body = WebhookEndpointResponse))
)]
pub async fn update_webhook(
    path: web::Path<String>,
    req: web::Json<UpdateWebhookEndpointRequest>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "webhook", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let webhook_id = path.into_inner();
    let update_req = req.into_inner();

    // Parse UUID
    let id = match Uuid::parse_str(&webhook_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid webhook ID"
            }));
        }
    };

    let result = sqlx::query_as::<_, WebhookEndpointDb>(
        "UPDATE webhook_endpoints SET
            endpoint_url = COALESCE($1, endpoint_url),
            event_types = COALESCE($2, event_types),
            active = COALESCE($3, active),
            retry_count = COALESCE($4, retry_count),
            timeout_seconds = COALESCE($5, timeout_seconds),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $6
        RETURNING *"
    )
    .bind(update_req.endpoint_url)
    .bind(update_req.event_types)
    .bind(update_req.active)
    .bind(update_req.retry_count)
    .bind(update_req.timeout_seconds)
    .bind(id)
    .fetch_optional(&data.db_pool)
    .await;

    match result {
        Ok(Some(webhook)) => {
            HttpResponse::Ok().json(WebhookEndpointResponse {
                id: webhook.id.to_string(),
                endpoint_url: webhook.endpoint_url,
                event_types: webhook.event_types,
                active: webhook.active,
                retry_count: webhook.retry_count,
                timeout_seconds: webhook.timeout_seconds,
                created_at: webhook.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Webhook not found"
        })),
        Err(e) => {
            eprintln!("Error updating webhook: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update webhook"
            }))
        }
    }
}

// 37. DELETE WEBHOOK ENDPOINT
#[utoipa::path(
    delete,
    path = "/webhooks/{id}",
    responses((status = 200, body = serde_json::Value))
)]
pub async fn delete_webhook(
    path: web::Path<String>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "webhook", "delete").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let webhook_id = path.into_inner();
    let id = match Uuid::parse_str(&webhook_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid webhook ID"
            }));
        }
    };

    let result = sqlx::query("DELETE FROM webhook_endpoints WHERE id = $1")
        .bind(id)
        .execute(&data.db_pool)
        .await;

    match result {
        Ok(rows) if rows.rows_affected() > 0 => {
            println!("🔔 Webhook deleted: {}", webhook_id);
            HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "message": "Webhook deleted"
            }))
        }
        _ => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Webhook not found"
        }))
    }
}

// 38. GET WEBHOOK DELIVERIES
#[utoipa::path(
    get,
    path = "/webhooks/{id}/deliveries",
    responses((status = 200, body = WebhookDeliveriesResponse))
)]
pub async fn get_webhook_deliveries(
    path: web::Path<String>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "webhook", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let webhook_id = path.into_inner();
    let id = match Uuid::parse_str(&webhook_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid webhook ID"
            }));
        }
    };

    match sqlx::query_as::<_, WebhookDeliveryDb>(
        "SELECT * FROM webhook_deliveries 
         WHERE webhook_endpoint_id = $1 
         ORDER BY created_at DESC 
         LIMIT 100"
    )
    .bind(id)
    .fetch_all(&data.db_pool)
    .await
    {
        Ok(deliveries) => {
            let delivery_responses: Vec<WebhookDeliveryResponse> = deliveries.into_iter().map(|d| {
                WebhookDeliveryResponse {
                    id: d.id.to_string(),
                    event_type: d.event_type,
                    status: d.status,
                    response_code: d.response_code,
                    attempts: d.attempts,
                    created_at: d.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    delivered_at: d.delivered_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                }
            }).collect();

            let total_count = delivery_responses.len() as i64;
            HttpResponse::Ok().json(WebhookDeliveriesResponse {
                deliveries: delivery_responses,
                total_count,
            })
        }
        Err(e) => {
            eprintln!("Error fetching webhook deliveries: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch webhook deliveries"
            }))
        }
    }
}
