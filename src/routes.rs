use sqlx::Row;
use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::api_state::AppState;
use crate::security::{
    AuthService, extract_claims, RbacService, require_permission, AuditService, Claims,
    generate_request_id, log_error_safely, create_error_response
};
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
use crate::integration::notifications::NotificationChannel;
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
    /// Target region for data processing. Use ISO country codes (e.g., "EU", "DE", "SK") for allowed regions.
    /// Blocked regions: "US", "CN", "RU" (case-insensitive, also blocks "US-*" patterns like "US-EAST-1", "US-WEST-2")
    /// Examples: "EU" (allowed), "DE" (allowed), "US" (blocked), "us-east-1" (blocked), "CN" (blocked)
    #[schema(example = "EU")]
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
/// Log AI action for compliance tracking
/// 
/// Logs an AI agent action with full compliance tracking. Automatically enforces:
/// - Sovereign Lock (blocks US/CN/RU regions and US-* patterns like us-east-1)
/// - Risk Assessment
/// - eIDAS Sealing
/// - Audit Logging
/// 
/// **Authentication:** Requires JWT token in `Authorization: Bearer <token>` header.
/// 
/// **Example allowed regions:** EU, DE, SK, FR, IT
/// **Example blocked regions:** US, CN, RU, us-east-1, US-WEST-2
#[utoipa::path(
    post,
    path = "/log_action",
    request_body = LogRequest,
    responses(
        (status = 200, description = "Action logged successfully (COMPLIANT)", body = LogResponse),
        (status = 403, description = "Action blocked (SOVEREIGNTY violation or other compliance issue)", body = LogResponse)
    ),
    tag = "Compliance"
)]
pub async fn log_action(
    req: web::Json<LogRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // INPUT VALIDATION - SECURITY: Prevent DoS and injection attacks
    // Validate agent_id
    if req.agent_id.is_empty() || req.agent_id.len() > 255 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid input",
            "message": "agent_id must be between 1 and 255 characters"
        }));
    }
    
    // Validate action
    if req.action.is_empty() || req.action.len() > 255 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid input",
            "message": "action must be between 1 and 255 characters"
        }));
    }
    
    // Validate payload length (prevent DoS via large payloads)
    if req.payload.len() > 1_000_000 { // 1MB limit
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid input",
            "message": "payload exceeds maximum size of 1MB"
        }));
    }
    
    // Validate user_id if provided
    if let Some(ref user_id) = req.user_id {
        if user_id.is_empty() || user_id.len() > 255 {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid input",
                "message": "user_id must be between 1 and 255 characters if provided"
            }));
        }
    }
    
    // Validate target_region if provided
    if let Some(ref region) = req.target_region {
        if region.len() > 50 {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid input",
                "message": "target_region must not exceed 50 characters"
            }));
        }
    }
    
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

    // A.1. PROCESSING OBJECTION CHECK (GDPR Article 21) - if user_id is provided
    // Check objections FIRST as they override consent (objection is stronger right)
    if let Some(ref user_id) = req.user_id {
        // Check for active processing objections
        #[derive(sqlx::FromRow)]
        struct ObjectionCheck {
            objection_type: String,
            objected_actions: serde_json::Value,
        }

        let objection_result: Option<ObjectionCheck> = sqlx::query_as(
            "SELECT objection_type, objected_actions
             FROM processing_objections
             WHERE user_id = $1 AND status = 'ACTIVE'
             LIMIT 1"
        )
        .bind(user_id)
        .fetch_optional(&data.db_pool)
        .await
        .unwrap_or(None);

        if let Some(objection) = objection_result {
            // Check if objection applies to this action
            let is_blocked = match objection.objection_type.as_str() {
                "FULL" => true, // Block all processing
                "DIRECT_MARKETING" => {
                    // Block direct marketing actions
                    req.action.contains("marketing") || req.action.contains("advertising")
                }
                "PROFILING" => {
                    // Block profiling actions
                    req.action.contains("profiling") || req.action.contains("automated_decision")
                }
                "PARTIAL" | "SPECIFIC_ACTION" => {
                    // Check if this specific action is in the objected list
                    if let Some(objected_actions) = objection.objected_actions.as_array() {
                        objected_actions.iter()
                            .any(|action| action.as_str() == Some(&req.action))
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if is_blocked {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "status": "BLOCKED (OBJECTION)",
                    "reason": format!("Processing objected by user: {} (Type: {})", user_id, objection.objection_type),
                    "user_id": user_id,
                    "objection_type": objection.objection_type
                }));
            }
        }
    }

    // A.2. PROCESSING RESTRICTION CHECK (GDPR Article 18) - if user_id is provided
    if let Some(ref user_id) = req.user_id {
        // Check for active processing restrictions
        #[derive(sqlx::FromRow)]
        struct RestrictionCheck {
            restriction_type: String,
            restricted_actions: serde_json::Value,
        }

        let restriction_result: Option<RestrictionCheck> = sqlx::query_as(
            "SELECT restriction_type, restricted_actions
             FROM processing_restrictions
             WHERE user_id = $1 AND status = 'ACTIVE'
               AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
             LIMIT 1"
        )
        .bind(user_id)
        .fetch_optional(&data.db_pool)
        .await
        .unwrap_or(None);

        if let Some(restriction) = restriction_result {
            // Check if restriction applies to this action
            let is_blocked = match restriction.restriction_type.as_str() {
                "FULL" => true, // Block all processing
                "PARTIAL" | "SPECIFIC_ACTION" => {
                    // Check if this specific action is in the restricted list
                    if let Some(restricted_actions) = restriction.restricted_actions.as_array() {
                        restricted_actions.iter()
                            .any(|action| action.as_str() == Some(&req.action))
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if is_blocked {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "status": "BLOCKED (RESTRICTION)",
                    "reason": format!("Processing restricted for user: {} (Type: {})", user_id, restriction.restriction_type),
                    "user_id": user_id,
                    "restriction_type": restriction.restriction_type
                }));
            }
        }
    }

    // A.3. CONSENT CHECK (GDPR Article 6, 7) - if user_id is provided
    // Check consent AFTER objections and restrictions (consent is checked last)
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
    // Check for blocked regions (case-insensitive, also catches AWS region patterns like "us-east-1")
    let target = req.target_region.as_deref().unwrap_or("").to_uppercase();
    let is_violation = target == "US" 
        || target == "CN"
        || target == "RU"
        || target == "USA"
        || target.starts_with("US-")  // Catches "US-EAST-1", "US-WEST-2", etc.
        || target.contains("UNITED STATES")
        || target == "CHINA"
        || target == "RUSSIA";
    
    let status = if is_violation { "BLOCKED (SOVEREIGNTY)" } else { "COMPLIANT" };

    // C. RISK ASSESSMENT (EU AI Act Article 9)
    // Enhanced risk assessment using context-aware methodology
    let risk_assessment_result = crate::core::risk_assessment::RiskAssessmentService::assess_risk(
        &data.db_pool,
        &req.action,
        &req.payload,
        req.user_id.as_deref(),
        Some(&req.agent_id),
        is_violation,
    ).await;

    let risk_level = risk_assessment_result.risk_level.to_string();
    let risk_factors: Vec<String> = risk_assessment_result.risk_factors.iter()
        .map(|rf| format!("{}: {} (score: {:.2})", rf.name, rf.description, rf.score))
        .collect();
    let mitigation_actions = if is_violation {
        vec!["Action blocked".to_string()]
    } else {
        // Use suggestions from risk assessment, with fallback
        if risk_assessment_result.mitigation_suggestions.is_empty() {
        vec!["Sovereign lock active".to_string(), "Encryption enabled".to_string()]
        } else {
            risk_assessment_result.mitigation_suggestions
        }
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
        let request_id = crate::security::generate_request_id();
        crate::security::log_error_safely("storing compliance record", &e, &request_id);
        return crate::security::create_error_response(&request_id);
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

    // H. AUTOMATED DECISION-MAKING DETECTION (GDPR Article 22)
    // Detect if this action constitutes automated decision-making
    if let Some(ref user_id) = req.user_id {
        if is_automated_decision(&req.action, &req.payload) {
            let decision_id = format!("DECISION-{}-{}", Local::now().format("%Y%m%d-%H%M%S"), Uuid::new_v4().to_string().chars().take(8).collect::<String>());
            let decision_outcome = extract_decision_outcome(&req.payload);
            let decision_reasoning = format!("Automated decision based on action: {} | Payload analysis indicates: {}", req.action, decision_outcome);
            
            // Determine legal effect based on action type
            let legal_effect = match req.action.to_lowercase().as_str() {
                action if action.contains("credit") => Some("Credit application decision".to_string()),
                action if action.contains("loan") => Some("Loan application decision".to_string()),
                action if action.contains("job") || action.contains("hiring") => Some("Job application decision".to_string()),
                action if action.contains("insurance") => Some("Insurance application decision".to_string()),
                _ => Some("Automated decision with legal effect".to_string()),
            };

            // Store automated decision record
            let _ = sqlx::query(
                "INSERT INTO automated_decisions (
                    decision_id, user_id, seal_id, action_type, decision_outcome,
                    decision_reasoning, legal_effect, significant_impact,
                    status, human_review_required, decision_timestamp
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
            )
            .bind(&decision_id)
            .bind(user_id)
            .bind(&seal_id)
            .bind(&req.action)
            .bind(&decision_outcome)
            .bind(&decision_reasoning)
            .bind(&legal_effect)
            .bind(true) // significant_impact
            .bind("PENDING_REVIEW")
            .bind(true) // human_review_required
            .bind(now)
            .execute(&data.db_pool)
            .await;

            // Send notification to user about automated decision (GDPR Article 22)
            let notification_service = data.notification_service.clone();
            let db_pool = data.db_pool.clone();
            let user_id_clone = user_id.clone();
            let seal_id_clone = seal_id.clone();
            let action_clone = req.action.clone();
            let decision_outcome_clone = decision_outcome.clone();
            let now_clone = now;
            let legal_effect_clone = legal_effect.clone();

            // Spawn async task to send notification (non-blocking)
            tokio::spawn(async move {
                let _ = notification_service.notify_high_risk_ai_action(
                    &db_pool,
                    &user_id_clone,
                    &seal_id_clone,
                    &format!("Automated Decision: {}", action_clone),
                    "HIGH",
                    &now_clone,
                    legal_effect_clone.as_deref(),
                    crate::integration::notifications::NotificationChannel::Email,
                ).await;
            });

            println!("🤖 Automated decision detected: {} | User: {} | Outcome: {}", 
                decision_id, user_id, decision_outcome);
        }
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

    // Send notification for high-risk AI actions (EU AI Act Article 13)
    // Also send for MEDIUM risk if confidence is high or historical context shows increasing trend
    let should_notify = (risk_level == "HIGH" || risk_level == "CRITICAL") 
        || (risk_level == "MEDIUM" && risk_assessment_result.confidence > 0.8 
            && risk_assessment_result.historical_context.as_ref()
                .map(|ctx| ctx.trend == "INCREASING")
                .unwrap_or(false));
    
    if should_notify && req.user_id.is_some() && !is_violation {
        let notification_service = data.notification_service.clone();
        let db_pool = data.db_pool.clone();
        let user_id = req.user_id.clone().unwrap();
        let seal_id_clone = seal_id.clone();
        let action_clone = req.action.clone();
        let risk_level_clone = risk_level.clone();
        let now_clone = now;
        let purpose = Some(format!("AI processing: {}", req.action));
        
        // Spawn async task to send notification (non-blocking)
        tokio::spawn(async move {
            // Try email first
            let email_result = notification_service.notify_high_risk_ai_action(
                &db_pool,
                &user_id,
                &seal_id_clone,
                &action_clone,
                &risk_level_clone,
                &now_clone,
                purpose.as_deref(),
                NotificationChannel::Email,
            ).await;
            
            if email_result.is_err() {
                // Fallback to SMS
                let _ = notification_service.notify_high_risk_ai_action(
                    &db_pool,
                    &user_id,
                    &seal_id_clone,
                    &action_clone,
                    &risk_level_clone,
                    &now_clone,
                    purpose.as_deref(),
                    NotificationChannel::Sms,
                ).await;
            }
        });
    }

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

/// Detect if an action constitutes automated decision-making (GDPR Article 22)
/// Automated decision-making includes decisions that:
/// 1. Are based solely on automated processing
/// 2. Produce legal effects or significantly affect the individual
fn is_automated_decision(action: &str, payload: &str) -> bool {
    let action_lower = action.to_lowercase();
    let payload_lower = payload.to_lowercase();
    
    // Keywords that indicate automated decision-making
    let automated_decision_keywords = [
        "credit_scoring", "credit_score", "credit_decision",
        "loan_approval", "loan_denial", "loan_decision",
        "job_screening", "hiring_decision", "recruitment",
        "insurance_underwriting", "insurance_decision",
        "automated_decision", "automated_screening",
        "profiling", "behavioral_analysis",
        "risk_assessment", "eligibility_check",
        "fraud_detection", "suspicious_activity",
    ];
    
    // Check if action matches automated decision keywords
    if automated_decision_keywords.iter().any(|kw| action_lower.contains(kw)) {
        return true;
    }
    
    // Check payload for decision-related terms
    let decision_indicators = [
        "approved", "rejected", "denied", "accepted",
        "score", "rating", "threshold", "eligibility",
        "qualify", "disqualify", "pass", "fail",
    ];
    
    if decision_indicators.iter().any(|ind| payload_lower.contains(ind)) {
        // Additional check: if action involves scoring/rating, it's likely automated decision
        if action_lower.contains("score") || action_lower.contains("rate") || action_lower.contains("assess") {
            return true;
        }
    }
    
    false
}

/// Extract decision outcome from payload (if available)
fn extract_decision_outcome(payload: &str) -> String {
    let payload_lower = payload.to_lowercase();
    
    if payload_lower.contains("approved") || payload_lower.contains("accept") || payload_lower.contains("pass") {
        "APPROVED".to_string()
    } else if payload_lower.contains("rejected") || payload_lower.contains("denied") || payload_lower.contains("fail") {
        "REJECTED".to_string()
    } else if payload_lower.contains("pending") || payload_lower.contains("review") {
        "PENDING".to_string()
    } else if payload_lower.contains("conditional") || payload_lower.contains("subject to") {
        "CONDITIONAL".to_string()
    } else {
        "PENDING".to_string()
    }
}

// 2. GET LOGS (with pagination and filtering)
#[utoipa::path(
    get,
    path = "/logs",
    params(
        ("page" = Option<i64>, Query, description = "Page number (default: 1)"),
        ("limit" = Option<i64>, Query, description = "Items per page (default: 100, max: 1000)"),
        ("seal_id" = Option<String>, Query, description = "Filter by seal_id"),
        ("agent_id" = Option<String>, Query, description = "Filter by agent_id")
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
    let limit = (query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100))
        .min(1000)
        .max(1);
    let offset = (page - 1) * limit;

    let seal_id = query.get("seal_id");
    let agent_id = query.get("agent_id");

    // Build dynamic queries based on filters
    let (total_count, records_result) = if let Some(sid) = seal_id {
        // Filter by seal_id
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM compliance_records WHERE seal_id = $1")
            .bind(sid)
            .fetch_one(&data.db_pool)
            .await
            .unwrap_or(0);
        
        let records = sqlx::query_as::<_, ComplianceRecordDb>(
            "SELECT * FROM compliance_records WHERE seal_id = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3"
        )
        .bind(sid)
        .bind(limit)
        .bind(offset)
        .fetch_all(&data.db_pool)
        .await;
        
        (count, records)
    } else if let Some(aid) = agent_id {
        // Filter by agent_id
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM compliance_records WHERE agent_id = $1")
            .bind(aid)
            .fetch_one(&data.db_pool)
            .await
            .unwrap_or(0);
        
        let records = sqlx::query_as::<_, ComplianceRecordDb>(
            "SELECT * FROM compliance_records WHERE agent_id = $1 ORDER BY timestamp DESC LIMIT $2 OFFSET $3"
        )
        .bind(aid)
        .bind(limit)
        .bind(offset)
        .fetch_all(&data.db_pool)
        .await;
        
        (count, records)
    } else {
        // No filter - get all records
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM compliance_records")
            .fetch_one(&data.db_pool)
            .await
            .unwrap_or(0);
        
        let records = sqlx::query_as::<_, ComplianceRecordDb>(
            "SELECT * FROM compliance_records ORDER BY timestamp DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&data.db_pool)
        .await;
        
        (count, records)
    };

    match records_result {
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
            let request_id = generate_request_id();
            log_error_safely("fetching logs", &e, &request_id);
            create_error_response(&request_id)
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
            let request_id = generate_request_id();
            log_error_safely("fetching tx_id", &e, &request_id);
            return create_error_response(&request_id);
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
    let erased_summary = "[GDPR PURGED] Data Cryptographically Erased";
    let erased_status = "ERASED (Art. 17)";
    let result = sqlx::query(
        "UPDATE compliance_records 
         SET action_summary = $2,
             status = $3
         WHERE seal_id = $1"
    )
    .bind(&req.seal_id)
    .bind(erased_summary)
    .bind(erased_status)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(rows) if rows.rows_affected() > 0 => {
            println!("🗑️ Shredded record: {}", req.seal_id);
            
            // GDPR Article 19: Notify recipients of erasure
            let notification_service = data.notification_service.clone();
            let db_pool = data.db_pool.clone();
            let seal_id_clone = req.seal_id.clone();
            let now = Utc::now();
            
            // Spawn async task to send notifications (non-blocking)
            tokio::spawn(async move {
                // Get user_id from compliance record
                let user_id: Option<String> = sqlx::query_scalar(
                    "SELECT user_id FROM compliance_records WHERE seal_id = $1"
                )
                .bind(&seal_id_clone)
                .fetch_optional(&db_pool)
                .await
                .unwrap_or(None);
                
                if let Some(uid) = user_id {
                    // Get all recipients who received this user's data
                    let recipients: Vec<(String, String)> = sqlx::query_as(
                        "SELECT recipient_name, recipient_contact 
                         FROM data_recipients 
                         WHERE user_id = $1 AND seal_id = $2"
                    )
                    .bind(&uid)
                    .bind(&seal_id_clone)
                    .fetch_all(&db_pool)
                    .await
                    .unwrap_or_default();
                    
                    // Send notification to user
                    let _ = notification_service.send_notification(
                        &db_pool,
                        crate::integration::notifications::NotificationRequest {
                            user_id: uid.clone(),
                            notification_type: crate::integration::notifications::NotificationType::ErasureDone,
                            channel: crate::integration::notifications::NotificationChannel::Email,
                            subject: Some("Data Erasure Notification".to_string()),
                            body: format!(
                                "Dear User,\n\nYour personal data has been erased as requested (GDPR Article 17).\n\nSeal ID: {}\n\nAs required by GDPR Article 19, we have notified all recipients of your personal data about this erasure.\n\nBest regards,\nVeridion Nexus Compliance Team",
                                seal_id_clone
                            ),
                            language: Some("en".to_string()),
                            related_entity_type: Some("COMPLIANCE_RECORD".to_string()),
                            related_entity_id: Some(seal_id_clone.clone()),
                        }
                    ).await;
                    
                    // Log notification to recipients (GDPR Article 19 requirement)
                    for (name, contact) in recipients {
                        let _ = sqlx::query(
                            "INSERT INTO user_notifications (
                                id, notification_id, user_id, notification_type, channel,
                                subject, body, status, language, related_entity_type, related_entity_id
                            ) VALUES (gen_random_uuid(), $1, $2, 'ERASURE_NOTIFICATION', 'EMAIL', 
                                      $3, $4, 'PENDING', 'en', 'RECIPIENT', $5)"
                        )
                        .bind(format!("NOTIF-RECIP-{}", uuid::Uuid::new_v4().to_string().replace("-", "").chars().take(12).collect::<String>()))
                        .bind(&uid)
                        .bind(Some(format!("Data Erasure - {}", name)))
                        .bind(format!("Data subject {} has requested erasure of their personal data. Please delete their data from your systems.", uid))
                        .bind(Some(contact))
                        .execute(&db_pool)
                        .await;
                    }
                }
            });
            
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

// 4. DOWNLOAD REPORT (Enhanced with format support and extended Annex IV fields)
#[utoipa::path(
    get,
    path = "/download_report",
    params(
        ("format" = Option<String>, Query, description = "Export format: pdf, json, xml (default: pdf)"),
        ("seal_id" = Option<String>, Query, description = "Filter by specific seal_id")
    ),
    responses(
        (status = 200, description = "Report generated successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Reports"
)]
pub async fn download_report(
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
) -> impl Responder {
    let format_str = query.get("format").map(|s| s.as_str()).unwrap_or("pdf");
    let format = crate::core::annex_iv::ExportFormat::from_str(format_str)
        .unwrap_or(crate::core::annex_iv::ExportFormat::Pdf);
    
    let seal_id_filter = query.get("seal_id");
    
    // Build query with optional seal_id filter
    let query_str = if seal_id_filter.is_some() {
        "SELECT cr.* FROM compliance_records cr WHERE cr.seal_id = $1 ORDER BY cr.timestamp DESC"
    } else {
        "SELECT cr.* FROM compliance_records cr ORDER BY cr.timestamp DESC LIMIT 1000"
    };
    
    let result = if let Some(seal_id) = seal_id_filter {
        sqlx::query_as::<_, ComplianceRecordDb>(query_str)
            .bind(seal_id)
    .fetch_all(&data.db_pool)
    .await
    } else {
        sqlx::query_as::<_, ComplianceRecordDb>(query_str)
            .fetch_all(&data.db_pool)
            .await
    };
    
    match result {
        Ok(records) => {
            // Convert to extended ComplianceRecord with Annex IV fields
            let mut compliance_records: Vec<crate::core::annex_iv::ComplianceRecord> = Vec::new();
            
            for r in records {
                let mut record: crate::core::annex_iv::ComplianceRecord = r.clone().into();
                
                // Add extended Annex IV fields
                // Try to extract lifecycle stage from action_summary or use default
                record.lifecycle_stage = Some("DEPLOYMENT".to_string()); // Default, can be enhanced
                
                // Get risk assessment data for mitigation actions
                if let Ok(Some(ra)) = sqlx::query_as::<_, RiskAssessmentDb>(
                    "SELECT * FROM risk_assessments WHERE seal_id = $1 LIMIT 1"
                )
                .bind(&r.seal_id)
                .fetch_optional(&data.db_pool)
                .await
                {
                    // Extract risk management measures from mitigation_actions
                    if let Ok(measures) = serde_json::from_value::<Vec<String>>(ra.mitigation_actions.clone()) {
                        record.risk_management_measures = Some(measures);
                    }
                }
                
                // Get post-market monitoring if available
                if let Ok(Some(pm)) = sqlx::query_scalar::<_, Option<String>>(
                    "SELECT monitoring_results FROM post_market_monitoring WHERE seal_id = $1 LIMIT 1"
                )
                .bind(&r.seal_id)
                .fetch_optional(&data.db_pool)
                .await
                {
                    record.post_market_monitoring = pm;
                }
                
                compliance_records.push(record);
            }
            
            let filename = format!("Veridion_Annex_IV_{}.{}", 
                chrono::Utc::now().format("%Y%m%d_%H%M%S"),
                format.file_extension()
            );
            let temp_file = format!("temp_report_{}.{}", 
                uuid::Uuid::new_v4().to_string().replace("-", ""),
                format.file_extension()
            );
            
            // Generate report in requested format
            let export_result = match format {
                crate::core::annex_iv::ExportFormat::Pdf => {
                    crate::core::annex_iv::generate_report(&compliance_records, &temp_file)
                }
                crate::core::annex_iv::ExportFormat::Json => {
                    crate::core::annex_iv::export_to_json(&compliance_records, &temp_file)
                }
                crate::core::annex_iv::ExportFormat::Xml => {
                    crate::core::annex_iv::export_to_xml(&compliance_records, &temp_file)
                }
            };
            
            match export_result {
                Ok(_) => {
                    if let Ok(bytes) = fs::read(&temp_file) {
                        // Clean up temp file
                        let _ = fs::remove_file(&temp_file);
                        
                        HttpResponse::Ok()
                            .content_type(format.content_type())
                            .append_header(ContentDisposition::attachment(&filename))
                            .body(bytes)
            } else { 
                        HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": "Failed to read generated report"
                        }))
                    }
                }
                Err(e) => {
                    let request_id = generate_request_id();
                    // e is a String, convert to &dyn Error
                    let error_msg = format!("{}", e);
                    log::error!("Error generating report: {} (Request ID: {})", error_msg, request_id);
                    HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to generate report"
                    }))
                }
            }
        }
        Err(e) => {
            let request_id = generate_request_id();
            log_error_safely("fetching records for report", &e, &request_id);
            create_error_response(&request_id)
        }
    }
}

// 5. REVOKE ACCESS
#[utoipa::path(post, path = "/revoke_access")]
pub async fn revoke_access(data: web::Data<AppState>) -> impl Responder {
    match data.set_locked_down(true).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "SUCCESS"})),
        Err(e) => {
            let request_id = generate_request_id();
            log_error_safely("setting lockdown", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching user data", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching user data", &e, &request_id);
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
            "error": "User ID does not match"
        }));
    }
    
    // Update record in database
    let status_text = "RECTIFIED (Art. 16)";
    let result = sqlx::query(
        "UPDATE compliance_records 
         SET action_summary = $1, status = $4
         WHERE seal_id = $2 AND user_id = $3"
    )
    .bind(format!("[RECTIFIED] {}", req.corrected_data))
    .bind(&req.seal_id)
    .bind(&user_id)
    .bind(status_text)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(rows) if rows.rows_affected() > 0 => {
            println!("📝 Rectified record: {} for user: {}", req.seal_id, user_id);
            
            // GDPR Article 19: Notify recipients of rectification
            let notification_service = data.notification_service.clone();
            let db_pool = data.db_pool.clone();
            let user_id_clone = user_id.clone();
            let seal_id_clone = req.seal_id.clone();
            let corrected_data_clone = req.corrected_data.clone();
            let now = Utc::now();
            
            // Spawn async task to send notifications to recipients (non-blocking)
            tokio::spawn(async move {
                // Get all recipients who received this user's data
                let recipients: Vec<(String, String)> = sqlx::query_as(
                    "SELECT recipient_name, recipient_contact 
                     FROM data_recipients 
                     WHERE user_id = $1 AND seal_id = $2"
                )
                .bind(&user_id_clone)
                .bind(&seal_id_clone)
                .fetch_all(&db_pool)
                .await
                .unwrap_or_default();
                
                // Send notification to user
                let _ = notification_service.send_notification(
                    &db_pool,
                    crate::integration::notifications::NotificationRequest {
                        user_id: user_id_clone.clone(),
                        notification_type: crate::integration::notifications::NotificationType::RectificationDone,
                        channel: crate::integration::notifications::NotificationChannel::Email,
                        subject: Some("Data Rectification Notification".to_string()),
                        body: format!(
                            "Dear User,\n\nYour personal data has been rectified as requested.\n\nSeal ID: {}\nCorrected Data: {}\n\nAs required by GDPR Article 19, we have notified all recipients of your personal data about this rectification.\n\nBest regards,\nVeridion Nexus Compliance Team",
                            seal_id_clone, corrected_data_clone
                        ),
                        language: Some("en".to_string()),
                        related_entity_type: Some("COMPLIANCE_RECORD".to_string()),
                        related_entity_id: Some(seal_id_clone.clone()),
                    }
                ).await;
                
                // Log notification to recipients (GDPR Article 19 requirement)
                for (name, contact) in recipients {
                    let _ = sqlx::query(
                        "INSERT INTO user_notifications (
                            id, notification_id, user_id, notification_type, channel,
                            subject, body, status, language, related_entity_type, related_entity_id
                        ) VALUES (gen_random_uuid(), $1, $2, 'RECTIFICATION_NOTIFICATION', 'EMAIL', 
                                  $3, $4, 'PENDING', 'en', 'RECIPIENT', $5)"
                    )
                    .bind(format!("NOTIF-RECIP-{}", uuid::Uuid::new_v4().to_string().replace("-", "").chars().take(12).collect::<String>()))
                    .bind(&user_id_clone)
                    .bind(Some(format!("Data Rectification - {}", name)))
                    .bind(format!("Data subject {} has requested rectification of their personal data. Please update your records accordingly.", user_id_clone))
                    .bind(Some(contact))
                    .execute(&db_pool)
                    .await;
                }
            });
            
            HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "message": "Data rectified successfully. Recipients will be notified as per GDPR Article 19."
            }))
        }
        _ => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Record not found or access denied"
        }))
    }
}

// ========== GDPR Article 18: Right to Restriction of Processing ==========

// 8.1. REQUEST PROCESSING RESTRICTION (GDPR Article 18)
#[utoipa::path(
    post,
    path = "/data_subject/{user_id}/restrict",
    request_body = ProcessingRestrictionRequest,
    responses((status = 200, body = ProcessingRestrictionResponse))
)]
pub async fn request_processing_restriction(
    path: web::Path<String>,
    req: web::Json<ProcessingRestrictionRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION - Critical: data_subject.write
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "data_subject", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let user_id = path.into_inner();
    if user_id != req.user_id {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "User ID does not match"
        }));
    }

    // Check if there's already an active restriction
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT restriction_id FROM processing_restrictions 
         WHERE user_id = $1 AND status = 'ACTIVE'"
    )
    .bind(&user_id)
    .fetch_optional(&data.db_pool)
    .await
    .unwrap_or(None);

    if existing.is_some() {
        return HttpResponse::Conflict().json(serde_json::json!({
            "error": "User already has an active processing restriction",
            "restriction_id": existing.unwrap().0
        }));
    }

    let restriction_id = format!("RESTRICT-{}", Local::now().format("%Y%m%d-%H%M%S"));
    let now = Utc::now();
    
    // Parse expires_at if provided
    let expires_at = req.expires_at.as_ref()
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok())
        .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));

    let restricted_actions_json = serde_json::to_value(req.restricted_actions.as_ref().unwrap_or(&vec![]))
        .unwrap_or(serde_json::json!([]));

    let result = sqlx::query(
        "INSERT INTO processing_restrictions (
            restriction_id, user_id, restriction_type, restricted_actions,
            reason, status, requested_at, expires_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(&restriction_id)
    .bind(&user_id)
    .bind(&req.restriction_type)
    .bind(restricted_actions_json)
    .bind(&req.reason)
    .bind("ACTIVE")
    .bind(now)
    .bind(expires_at)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(_) => {
            let response = ProcessingRestrictionResponse {
                restriction_id: restriction_id.clone(),
                user_id: user_id.clone(),
                restriction_type: req.restriction_type.clone(),
                status: "ACTIVE".to_string(),
                requested_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                expires_at: expires_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            };

            println!("🔒 Processing restriction requested: {} | User: {} | Type: {}", 
                restriction_id, user_id, req.restriction_type);

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("creating restriction", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create processing restriction"
            }))
        }
    }
}

// 8.2. LIFT PROCESSING RESTRICTION (GDPR Article 18)
#[utoipa::path(
    post,
    path = "/data_subject/{user_id}/lift_restriction",
    request_body = LiftRestrictionRequest,
    responses((status = 200, body = serde_json::Value))
)]
pub async fn lift_processing_restriction(
    path: web::Path<String>,
    req: web::Json<LiftRestrictionRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION - Critical: data_subject.write
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "data_subject", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let user_id = path.into_inner();
    if user_id != req.user_id {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "User ID does not match"
        }));
    }

    let now = Utc::now();
    let lifted_by = req.lifted_by.as_deref().unwrap_or("SYSTEM");

    let result = sqlx::query(
        "UPDATE processing_restrictions 
         SET status = 'LIFTED', lifted_at = $1, lifted_by = $2, lift_reason = $3
         WHERE user_id = $4 AND status = 'ACTIVE'"
    )
    .bind(now)
    .bind(lifted_by)
    .bind(&req.reason)
    .bind(&user_id)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(rows) => {
            if rows.rows_affected() == 0 {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "No active restriction found for this user"
                }));
            }

            println!("🔓 Processing restriction lifted: User: {} | Lifted by: {}", user_id, lifted_by);

            HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "message": "Processing restriction lifted successfully",
                "user_id": user_id,
                "lifted_at": now.format("%Y-%m-%d %H:%M:%S").to_string()
            }))
        }
        Err(e) => {
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("lifting restriction", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to lift processing restriction"
            }))
        }
    }
}

// 8.3. GET PROCESSING RESTRICTIONS (GDPR Article 18)
#[utoipa::path(
    get,
    path = "/data_subject/{user_id}/restrictions",
    responses((status = 200, body = RestrictionsResponse))
)]
pub async fn get_processing_restrictions(
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

    #[derive(sqlx::FromRow)]
    struct RestrictionRow {
        restriction_id: String,
        user_id: String,
        restriction_type: String,
        status: String,
        requested_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
    }

    let result = sqlx::query_as::<_, RestrictionRow>(
        "SELECT restriction_id, user_id, restriction_type, status, requested_at, expires_at
         FROM processing_restrictions
         WHERE user_id = $1
         ORDER BY requested_at DESC"
    )
    .bind(&user_id)
    .fetch_all(&data.db_pool)
    .await;

    match result {
        Ok(rows) => {
            let restrictions: Vec<ProcessingRestrictionResponse> = rows.into_iter().map(|r| {
                ProcessingRestrictionResponse {
                    restriction_id: r.restriction_id,
                    user_id: r.user_id,
                    restriction_type: r.restriction_type,
                    status: r.status,
                    requested_at: r.requested_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    expires_at: r.expires_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                }
            }).collect();

            let response = RestrictionsResponse {
                user_id: user_id.clone(),
                restrictions,
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching restrictions", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch processing restrictions"
            }))
        }
    }
}

// ========== GDPR Article 21: Right to Object ==========

// 8.4. REQUEST PROCESSING OBJECTION (GDPR Article 21)
#[utoipa::path(
    post,
    path = "/data_subject/{user_id}/object",
    request_body = ProcessingObjectionRequest,
    responses((status = 200, body = ProcessingObjectionResponse))
)]
pub async fn request_processing_objection(
    path: web::Path<String>,
    req: web::Json<ProcessingObjectionRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION - Critical: data_subject.write
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "data_subject", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let user_id = path.into_inner();
    if user_id != req.user_id {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "User ID does not match"
        }));
    }

    // Check if there's already an active objection of this type
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT objection_id FROM processing_objections 
         WHERE user_id = $1 AND objection_type = $2 AND status = 'ACTIVE'"
    )
    .bind(&user_id)
    .bind(&req.objection_type)
    .fetch_optional(&data.db_pool)
    .await
    .unwrap_or(None);

    if existing.is_some() {
        return HttpResponse::Conflict().json(serde_json::json!({
            "error": format!("User already has an active {} objection", req.objection_type),
            "objection_id": existing.unwrap().0
        }));
    }

    let objection_id = format!("OBJECT-{}-{}", Local::now().format("%Y%m%d-%H%M%S"), Uuid::new_v4().to_string().chars().take(8).collect::<String>());
    let now = Utc::now();

    let objected_actions_json = serde_json::to_value(req.objected_actions.as_ref().unwrap_or(&vec![]))
        .unwrap_or(serde_json::json!([]));

    let result = sqlx::query(
        "INSERT INTO processing_objections (
            objection_id, user_id, objection_type, objected_actions,
            legal_basis, reason, status, requested_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(&objection_id)
    .bind(&user_id)
    .bind(&req.objection_type)
    .bind(objected_actions_json)
    .bind(&req.legal_basis)
    .bind(&req.reason)
    .bind("ACTIVE")
    .bind(now)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(_) => {
            let response = ProcessingObjectionResponse {
                objection_id: objection_id.clone(),
                user_id: user_id.clone(),
                objection_type: req.objection_type.clone(),
                status: "ACTIVE".to_string(),
                requested_at: now.format("%Y-%m-%d %H:%M:%S").to_string(),
                rejection_reason: None,
            };

            println!("🚫 Processing objection requested: {} | User: {} | Type: {}", 
                objection_id, user_id, req.objection_type);

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("creating objection", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create processing objection"
            }))
        }
    }
}

// 8.5. WITHDRAW PROCESSING OBJECTION (GDPR Article 21)
#[utoipa::path(
    post,
    path = "/data_subject/{user_id}/withdraw_objection",
    request_body = WithdrawObjectionRequest,
    responses((status = 200, body = serde_json::Value))
)]
pub async fn withdraw_processing_objection(
    path: web::Path<String>,
    req: web::Json<WithdrawObjectionRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION - Critical: data_subject.write
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "data_subject", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let user_id = path.into_inner();
    if user_id != req.user_id {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "User ID does not match"
        }));
    }

    let now = Utc::now();
    let withdrawn_by = "USER"; // User withdrawing their own objection

    let result = sqlx::query(
        "UPDATE processing_objections 
         SET status = 'WITHDRAWN', withdrawn_at = $1, withdrawn_by = $2, withdraw_reason = $3
         WHERE user_id = $4 AND status = 'ACTIVE'"
    )
    .bind(now)
    .bind(withdrawn_by)
    .bind(&req.reason)
    .bind(&user_id)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(rows) => {
            if rows.rows_affected() == 0 {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "No active objection found for this user"
                }));
            }

            println!("✅ Processing objection withdrawn: User: {}", user_id);

            HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "message": "Processing objection withdrawn successfully",
                "user_id": user_id,
                "withdrawn_at": now.format("%Y-%m-%d %H:%M:%S").to_string()
            }))
        }
        Err(e) => {
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("withdrawing objection", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to withdraw processing objection"
            }))
        }
    }
}

// 8.6. REJECT PROCESSING OBJECTION (GDPR Article 21(1) - must provide reason)
#[utoipa::path(
    post,
    path = "/data_subject/{user_id}/reject_objection",
    request_body = RejectObjectionRequest,
    responses((status = 200, body = serde_json::Value))
)]
pub async fn reject_processing_objection(
    path: web::Path<String>,
    req: web::Json<RejectObjectionRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION - Critical: data_subject.write (admin only for rejection)
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "data_subject", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let user_id = path.into_inner();
    if user_id != req.user_id {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "User ID does not match"
        }));
    }

    // GDPR Article 21(1) requires a reason for rejection
    if req.rejection_reason.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Rejection reason is required per GDPR Article 21(1)"
        }));
    }

    let now = Utc::now();
    let rejected_by = req.rejected_by.as_deref().unwrap_or("SYSTEM");

    let result = sqlx::query(
        "UPDATE processing_objections 
         SET status = 'REJECTED', rejected_at = $1, rejected_by = $2, rejection_reason = $3
         WHERE user_id = $4 AND status = 'ACTIVE'"
    )
    .bind(now)
    .bind(rejected_by)
    .bind(&req.rejection_reason)
    .bind(&user_id)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(rows) => {
            if rows.rows_affected() == 0 {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "No active objection found for this user"
                }));
            }

            println!("❌ Processing objection rejected: User: {} | By: {} | Reason: {}", 
                user_id, rejected_by, req.rejection_reason);

            HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "message": "Processing objection rejected",
                "user_id": user_id,
                "rejection_reason": req.rejection_reason,
                "rejected_at": now.format("%Y-%m-%d %H:%M:%S").to_string()
            }))
        }
        Err(e) => {
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("rejecting objection", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to reject processing objection"
            }))
        }
    }
}

// 8.7. GET PROCESSING OBJECTIONS (GDPR Article 21)
#[utoipa::path(
    get,
    path = "/data_subject/{user_id}/objections",
    responses((status = 200, body = ObjectionsResponse))
)]
pub async fn get_processing_objections(
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

    #[derive(sqlx::FromRow)]
    struct ObjectionRow {
        objection_id: String,
        user_id: String,
        objection_type: String,
        status: String,
        requested_at: DateTime<Utc>,
        rejection_reason: Option<String>,
    }

    let result = sqlx::query_as::<_, ObjectionRow>(
        "SELECT objection_id, user_id, objection_type, status, requested_at, rejection_reason
         FROM processing_objections
         WHERE user_id = $1
         ORDER BY requested_at DESC"
    )
    .bind(&user_id)
    .fetch_all(&data.db_pool)
    .await;

    match result {
        Ok(rows) => {
            let objections: Vec<ProcessingObjectionResponse> = rows.into_iter().map(|r| {
                ProcessingObjectionResponse {
                    objection_id: r.objection_id,
                    user_id: r.user_id,
                    objection_type: r.objection_type,
                    status: r.status,
                    requested_at: r.requested_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    rejection_reason: r.rejection_reason,
                }
            }).collect();

            let response = ObjectionsResponse {
                user_id: user_id.clone(),
                objections,
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching objections", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch processing objections"
            }))
        }
    }
}

// ========== GDPR Article 22: Automated Decision-Making ==========

// 8.8. REQUEST HUMAN REVIEW (GDPR Article 22)
#[utoipa::path(
    post,
    path = "/data_subject/{user_id}/request_review",
    request_body = RequestReviewRequest,
    responses((status = 200, body = RequestReviewResponse))
)]
pub async fn request_human_review(
    path: web::Path<String>,
    req: web::Json<RequestReviewRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION - Critical: data_subject.write
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "data_subject", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let user_id = path.into_inner();
    if user_id != req.user_id {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "User ID does not match"
        }));
    }

    // Find decision by decision_id or seal_id
    let decision_result: Option<(String, String)> = if let Some(ref decision_id) = req.decision_id {
        sqlx::query_as(
            "SELECT decision_id, seal_id FROM automated_decisions 
             WHERE decision_id = $1 AND user_id = $2"
        )
        .bind(decision_id)
        .bind(&user_id)
        .fetch_optional(&data.db_pool)
        .await
        .unwrap_or(None)
    } else if let Some(ref seal_id) = req.seal_id {
        sqlx::query_as(
            "SELECT decision_id, seal_id FROM automated_decisions 
             WHERE seal_id = $1 AND user_id = $2"
        )
        .bind(seal_id)
        .bind(&user_id)
        .fetch_optional(&data.db_pool)
        .await
        .unwrap_or(None)
    } else {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Either decision_id or seal_id must be provided"
        }));
    };

    let (decision_id, seal_id) = match decision_result {
        Some((did, sid)) => (did, sid),
        None => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Automated decision not found"
            }));
        }
    };

    // Update decision status to UNDER_REVIEW and create human oversight record
    let now = Utc::now();
    
    // Update automated decision
    let update_result = sqlx::query(
        "UPDATE automated_decisions 
         SET status = 'UNDER_REVIEW', human_review_required = true, updated_at = $1
         WHERE decision_id = $2"
    )
    .bind(now)
    .bind(&decision_id)
    .execute(&data.db_pool)
    .await;

    match update_result {
        Ok(_) => {
            // Create human oversight record (if not exists)
            let _ = sqlx::query(
                "INSERT INTO human_oversight (seal_id, status) VALUES ($1, 'PENDING')
                 ON CONFLICT (seal_id) DO UPDATE SET status = 'PENDING', updated_at = CURRENT_TIMESTAMP"
            )
            .bind(&seal_id)
            .execute(&data.db_pool)
            .await;

            println!("👤 Human review requested: Decision: {} | User: {}", decision_id, user_id);

            let response = RequestReviewResponse {
                decision_id: decision_id.clone(),
                status: "UNDER_REVIEW".to_string(),
                message: "Human review requested successfully. The decision will be reviewed by a human reviewer.".to_string(),
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("requesting review", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to request human review"
            }))
        }
    }
}

// 8.9. APPEAL AUTOMATED DECISION (GDPR Article 22)
#[utoipa::path(
    post,
    path = "/data_subject/{user_id}/appeal_decision",
    request_body = AppealDecisionRequest,
    responses((status = 200, body = serde_json::Value))
)]
pub async fn appeal_automated_decision(
    path: web::Path<String>,
    req: web::Json<AppealDecisionRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION - Critical: data_subject.write
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "data_subject", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let user_id = path.into_inner();
    if user_id != req.user_id {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "User ID does not match"
        }));
    }

    if req.appeal_reason.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Appeal reason is required"
        }));
    }

    let now = Utc::now();

    let result = sqlx::query(
        "UPDATE automated_decisions 
         SET appeal_requested = true, appeal_requested_at = $1, appeal_reason = $2, status = 'APPEALED'
         WHERE decision_id = $3 AND user_id = $4"
    )
    .bind(now)
    .bind(&req.appeal_reason)
    .bind(&req.decision_id)
    .bind(&user_id)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(rows) => {
            if rows.rows_affected() == 0 {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Automated decision not found or access denied"
                }));
            }

            println!("📝 Appeal requested: Decision: {} | User: {}", req.decision_id, user_id);

            HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "message": "Appeal requested successfully",
                "decision_id": req.decision_id,
                "appeal_requested_at": now.format("%Y-%m-%d %H:%M:%S").to_string()
            }))
        }
        Err(e) => {
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("appealing decision", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to appeal automated decision"
            }))
        }
    }
}

// 8.10. GET AUTOMATED DECISIONS (GDPR Article 22)
#[utoipa::path(
    get,
    path = "/data_subject/{user_id}/automated_decisions",
    responses((status = 200, body = AutomatedDecisionsResponse))
)]
pub async fn get_automated_decisions(
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

    #[derive(sqlx::FromRow)]
    struct DecisionRow {
        decision_id: String,
        user_id: String,
        seal_id: String,
        action_type: String,
        decision_outcome: String,
        decision_reasoning: Option<String>,
        legal_effect: Option<String>,
        significant_impact: bool,
        status: String,
        decision_timestamp: DateTime<Utc>,
        human_review_required: bool,
    }

    let result = sqlx::query_as::<_, DecisionRow>(
        "SELECT decision_id, user_id, seal_id, action_type, decision_outcome,
                decision_reasoning, legal_effect, significant_impact, status,
                decision_timestamp, human_review_required
         FROM automated_decisions
         WHERE user_id = $1
         ORDER BY decision_timestamp DESC"
    )
    .bind(&user_id)
    .fetch_all(&data.db_pool)
    .await;

    match result {
        Ok(rows) => {
            let decisions: Vec<AutomatedDecisionResponse> = rows.into_iter().map(|r| {
                AutomatedDecisionResponse {
                    decision_id: r.decision_id,
                    user_id: r.user_id,
                    seal_id: r.seal_id,
                    action_type: r.action_type,
                    decision_outcome: r.decision_outcome,
                    decision_reasoning: r.decision_reasoning,
                    legal_effect: r.legal_effect,
                    significant_impact: r.significant_impact,
                    status: r.status,
                    decision_timestamp: r.decision_timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                    human_review_required: r.human_review_required,
                }
            }).collect();

            let response = AutomatedDecisionsResponse {
                user_id: user_id.clone(),
                decisions,
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching automated decisions", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch automated decisions"
            }))
        }
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
            "error": "Seal ID does not match"
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("requiring human oversight", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching risk assessment", &e, &request_id);
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
    let limit = (query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100))
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching risks", &e, &request_id);
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
            
            // Send notifications to affected users (GDPR Article 33)
            if users_notified_at.is_some() {
                let notification_service = data.notification_service.clone();
                let db_pool = data.db_pool.clone();
                let breach_id_clone = breach_id.clone();
                let breach_type_clone = breach_report.breach_type.clone();
                let description_clone = breach_report.description.clone();
                let detected_at_clone = detected_at;
                let affected_users_clone = breach_report.affected_users.clone();
                
                // Spawn async task to send notifications (non-blocking)
                tokio::spawn(async move {
                    for user_id in &affected_users_clone {
                        // Try email first, fallback to SMS if email fails
                        let email_result = notification_service.notify_data_breach(
                            &db_pool,
                            user_id,
                            &breach_id_clone,
                            &breach_type_clone,
                            &detected_at_clone,
                            &description_clone,
                            NotificationChannel::Email,
                        ).await;
                        
                        if email_result.is_err() {
                            // Fallback to SMS
                            let _ = notification_service.notify_data_breach(
                                &db_pool,
                                user_id,
                                &breach_id_clone,
                                &breach_type_clone,
                                &detected_at_clone,
                                &description_clone,
                                NotificationChannel::Sms,
                            ).await;
                        }
                    }
                });
            }
            
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("reporting breach", &e, &request_id);
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
    let limit = (query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100))
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching breaches", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("granting consent", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("withdrawing consent", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching consents", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("creating DPIA", &e, &request_id);
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
    let limit = (query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100))
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching DPIAs", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("creating retention policy", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching expiring records", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching policy", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("assigning retention policy", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching retention status", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching retention policies", &e, &request_id);
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
                    let retention_summary = "[RETENTION EXPIRED] Data Automatically Deleted";
                    let retention_status = "DELETED (Retention Period)";
                    sqlx::query(
                        "UPDATE compliance_records 
                         SET action_summary = $2,
                             status = $3
                         WHERE seal_id = $1"
                    )
                    .bind(&assignment.record_id)
                    .bind(retention_summary)
                    .bind(retention_status)
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("creating monitoring event", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("checking event existence", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("updating event", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("updating event resolution", &e, &request_id);
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
    let limit = (query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100))
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching monitoring events", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching system health", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching system inventory", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("registering AI system", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching webhooks", &e, &request_id);
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
            let request_id = generate_request_id();
            log_error_safely("registering webhook", &e, &request_id);
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
    let limit = (query
        .get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100))
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
            let request_id = generate_request_id();
            log_error_safely("fetching webhooks", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("updating webhook", &e, &request_id);
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
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching webhook deliveries", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch webhook deliveries"
            }))
        }
    }
}

// ============================================================================
// Priority 2: User Notification Preferences (EU AI Act Article 13)
// ============================================================================

#[derive(Serialize, Deserialize, ToSchema)]
pub struct NotificationPreferenceRequest {
    #[schema(example = "HIGH_RISK_AI_ACTION")]
    pub notification_type: String,
    #[schema(example = r#"["EMAIL", "SMS", "IN_APP"]"#)]
    pub preferred_channels: Vec<String>,
    #[schema(example = "en")]
    pub language: String,
    #[schema(example = true)]
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct NotificationPreferenceResponse {
    pub notification_type: String,
    pub preferred_channels: Vec<String>,
    pub language: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Set user notification preferences (EU AI Act Article 13)
#[utoipa::path(
    post,
    path = "/user/{user_id}/notification_preferences",
    request_body = NotificationPreferenceRequest,
    responses(
        (status = 200, description = "Preferences updated successfully", body = NotificationPreferenceResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    tag = "Notifications"
)]
pub async fn set_notification_preferences(
    user_id: web::Path<String>,
    req: web::Json<NotificationPreferenceRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "notifications", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let user_id_str = user_id.into_inner();
    let pref = req.into_inner();
    let channels_json = serde_json::to_value(&pref.preferred_channels).unwrap_or(serde_json::json!([]));

    let result = sqlx::query(
        "INSERT INTO user_notification_preferences (
            user_id, notification_type, preferred_channels, language, enabled
        ) VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (user_id, notification_type) DO UPDATE SET
            preferred_channels = EXCLUDED.preferred_channels,
            language = EXCLUDED.language,
            enabled = EXCLUDED.enabled,
            updated_at = CURRENT_TIMESTAMP
        RETURNING created_at, updated_at"
    )
    .bind(&user_id_str)
    .bind(&pref.notification_type)
    .bind(channels_json)
    .bind(&pref.language)
    .bind(pref.enabled)
    .fetch_one(&data.db_pool)
    .await;

    match result {
        Ok(row) => {
            let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at").unwrap_or_else(|_| chrono::Utc::now());
            let updated_at: chrono::DateTime<chrono::Utc> = row.try_get("updated_at").unwrap_or_else(|_| chrono::Utc::now());
            HttpResponse::Ok().json(NotificationPreferenceResponse {
                notification_type: pref.notification_type,
                preferred_channels: pref.preferred_channels,
                language: pref.language,
                enabled: pref.enabled,
                created_at: created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        }
        Err(e) => {
            let request_id = generate_request_id();
            log_error_safely("setting notification preferences", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to set notification preferences"
            }))
        }
    }
}

/// Get user notification preferences
#[utoipa::path(
    get,
    path = "/user/{user_id}/notification_preferences",
    responses(
        (status = 200, description = "Preferences retrieved successfully", body = Vec<NotificationPreferenceResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    tag = "Notifications"
)]
pub async fn get_notification_preferences(
    user_id: web::Path<String>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "notifications", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let user_id_str = user_id.into_inner();

    #[derive(sqlx::FromRow)]
    struct PreferenceRow {
        notification_type: String,
        preferred_channels: serde_json::Value,
        language: String,
        enabled: bool,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    }

    match sqlx::query_as::<_, PreferenceRow>(
        "SELECT notification_type, preferred_channels, language, enabled, created_at, updated_at
         FROM user_notification_preferences
         WHERE user_id = $1
         ORDER BY notification_type"
    )
    .bind(&user_id_str)
    .fetch_all(&data.db_pool)
    .await
    {
        Ok(prefs) => {
            let responses: Vec<NotificationPreferenceResponse> = prefs.into_iter().map(|p| {
                let channels: Vec<String> = serde_json::from_value(p.preferred_channels)
                    .unwrap_or_else(|_| vec![]);
                NotificationPreferenceResponse {
                    notification_type: p.notification_type,
                    preferred_channels: channels,
                    language: p.language,
                    enabled: p.enabled,
                    created_at: p.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    updated_at: p.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                }
            }).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => {
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching notification preferences", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch notification preferences"
            }))
        }
    }
}

/// Get notification history for a user
#[utoipa::path(
    get,
    path = "/user/{user_id}/notifications",
    params(
        ("user_id" = String, Path, description = "User ID"),
        ("limit" = Option<i32>, Query, description = "Maximum number of notifications to return (default: 50)")
    ),
    responses(
        (status = 200, description = "Notifications retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    tag = "Notifications"
)]
pub async fn get_user_notifications(
    user_id: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "notifications", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let user_id_str = user_id.into_inner();
    let limit = query.get("limit")
        .and_then(|l| l.parse::<i32>().ok())
        .unwrap_or(50)
        .min(100); // Cap at 100

    #[derive(sqlx::FromRow)]
    struct NotificationRow {
        notification_id: String,
        notification_type: String,
        channel: String,
        subject: Option<String>,
        body: String,
        status: String,
        sent_at: Option<chrono::DateTime<chrono::Utc>>,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    match sqlx::query_as::<_, NotificationRow>(
        "SELECT notification_id, notification_type, channel, subject, body, status, sent_at, created_at
         FROM user_notifications
         WHERE user_id = $1
         ORDER BY created_at DESC
         LIMIT $2"
    )
    .bind(&user_id_str)
    .bind(limit)
    .fetch_all(&data.db_pool)
    .await
    {
        Ok(notifications) => {
            let responses: Vec<serde_json::Value> = notifications.into_iter().map(|n| {
                serde_json::json!({
                    "notification_id": n.notification_id,
                    "notification_type": n.notification_type,
                    "channel": n.channel,
                    "subject": n.subject,
                    "body": n.body,
                    "status": n.status,
                    "sent_at": n.sent_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
                    "created_at": n.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                })
            }).collect();
            HttpResponse::Ok().json(serde_json::json!({
                "notifications": responses,
                "total": responses.len()
            }))
        }
        Err(e) => {
            let request_id = generate_request_id();
            log_error_safely("fetching user notifications", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch notifications"
            }))
        }
    }
}

// ============================================================================
// Priority 3: GDPR Article 30 - Records of Processing Activities
// ============================================================================

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ProcessingRecordResponse {
    pub activity_name: String,
    pub purpose: String,
    pub legal_basis: String,
    pub data_categories: Vec<String>,
    pub data_subject_categories: Vec<String>,
    pub recipients: Vec<String>,
    pub third_country_transfers: bool,
    pub third_countries: Vec<String>,
    pub retention_period_days: Option<i32>,
    pub security_measures: Vec<String>,
    pub seal_id: Option<String>,
    pub timestamp: String,
}

/// Get processing records in GDPR Article 30 format
#[utoipa::path(
    get,
    path = "/processing_records",
    params(
        ("format" = Option<String>, Query, description = "Export format: json, csv (default: json)")
    ),
    responses(
        (status = 200, description = "Processing records retrieved successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    tag = "Compliance"
)]
pub async fn get_processing_records(
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "compliance", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let format_str = query.get("format").map(|s| s.as_str()).unwrap_or("json");

    // Get processing activities from database
    match sqlx::query_as::<_, ProcessingActivityDb>(
        "SELECT * FROM processing_activities ORDER BY created_at DESC"
    )
    .fetch_all(&data.db_pool)
    .await
    {
        Ok(activities) => {
            // Also get compliance records to enrich with seal_id and timestamp
            let mut records: Vec<ProcessingRecordResponse> = Vec::new();
            
            for activity in activities {
                // Try to find related compliance record
                let seal_id: Option<String> = sqlx::query_scalar(
                    "SELECT seal_id FROM compliance_records 
                     WHERE action_summary LIKE $1 
                     ORDER BY timestamp DESC LIMIT 1"
                )
                .bind(format!("%{}%", activity.activity_name))
                .fetch_optional(&data.db_pool)
                .await
                .unwrap_or(None);
                
                let timestamp: Option<chrono::DateTime<chrono::Utc>> = if let Some(ref sid) = seal_id {
                    sqlx::query_scalar(
                        "SELECT timestamp FROM compliance_records WHERE seal_id = $1"
                    )
                    .bind(sid)
                    .fetch_optional(&data.db_pool)
                    .await
                    .unwrap_or(None)
                } else {
                    Some(activity.created_at)
                };
                
                records.push(ProcessingRecordResponse {
                    activity_name: activity.activity_name,
                    purpose: activity.purpose,
                    legal_basis: activity.legal_basis,
                    data_categories: activity.data_categories,
                    data_subject_categories: activity.data_subject_categories,
                    recipients: activity.recipients,
                    third_country_transfers: activity.third_country_transfers,
                    third_countries: activity.third_countries,
                    retention_period_days: activity.retention_period_days,
                    security_measures: activity.security_measures,
                    seal_id,
                    timestamp: timestamp.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| activity.created_at.format("%Y-%m-%d %H:%M:%S").to_string()),
                });
            }
            
            // If no processing_activities exist, generate from compliance_records
            if records.is_empty() {
                let compliance_records: Vec<ComplianceRecordDb> = sqlx::query_as(
                    "SELECT * FROM compliance_records 
                     WHERE user_id IS NOT NULL 
                     ORDER BY timestamp DESC LIMIT 100"
                )
                .fetch_all(&data.db_pool)
                .await
                .unwrap_or_default();
                
                for cr in compliance_records {
                    records.push(ProcessingRecordResponse {
                        activity_name: cr.action_summary.clone(),
                        purpose: "AI system processing".to_string(),
                        legal_basis: "CONSENT".to_string(), // Default, should be from consent_records
                        data_categories: vec!["Personal data".to_string()],
                        data_subject_categories: vec!["Data subjects".to_string()],
                        recipients: vec![],
                        third_country_transfers: false,
                        third_countries: vec![],
                        retention_period_days: None,
                        security_measures: vec!["Encryption".to_string(), "Access controls".to_string()],
                        seal_id: Some(cr.seal_id),
                        timestamp: cr.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                    });
                }
            }
            
            if format_str == "csv" {
                // Generate CSV
                let mut csv = String::from("Activity Name,Purpose,Legal Basis,Data Categories,Data Subject Categories,Recipients,Third Country Transfers,Third Countries,Retention Period (Days),Security Measures,Seal ID,Timestamp\n");
                
                for record in &records {
                    csv.push_str(&format!(
                        "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{},\"{}\",{},\"{}\",\"{}\",\"{}\"\n",
                        record.activity_name,
                        record.purpose,
                        record.legal_basis,
                        record.data_categories.join("; "),
                        record.data_subject_categories.join("; "),
                        record.recipients.join("; "),
                        record.third_country_transfers,
                        record.third_countries.join("; "),
                        record.retention_period_days.map(|d| d.to_string()).unwrap_or_else(|| "N/A".to_string()),
                        record.security_measures.join("; "),
                        record.seal_id.as_deref().unwrap_or("N/A"),
                        record.timestamp
                    ));
                }
                
                HttpResponse::Ok()
                    .content_type("text/csv")
                    .append_header(actix_web::http::header::ContentDisposition::attachment("processing_records_article30.csv"))
                    .body(csv)
            } else {
                HttpResponse::Ok().json(serde_json::json!({
                    "records": records,
                    "total": records.len(),
                    "format": "GDPR Article 30"
                }))
            }
        }
        Err(e) => {
            let request_id = generate_request_id();
            log_error_safely("fetching processing records", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch processing records"
            }))
        }
    }
}

// ============================================================================
// Priority 3: EU AI Act Article 8 - Conformity Assessment
// ============================================================================

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ConformityAssessmentRequest {
    pub system_id: String,
    pub system_name: String,
    pub assessment_type: String, // 'SELF_ASSESSMENT', 'THIRD_PARTY', 'NOTIFIED_BODY'
    pub assessment_date: String,
    pub expiration_date: Option<String>,
    pub assessment_result: serde_json::Value,
    pub assessor_name: Option<String>,
    pub assessor_contact: Option<String>,
    pub certificate_number: Option<String>,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ConformityAssessmentResponse {
    pub assessment_id: String,
    pub system_id: String,
    pub system_name: String,
    pub assessment_type: String,
    pub assessment_date: String,
    pub expiration_date: Option<String>,
    pub status: String,
    pub days_until_expiration: Option<i64>,
    pub assessor_name: Option<String>,
    pub certificate_number: Option<String>,
}

/// Create or update conformity assessment
#[utoipa::path(
    post,
    path = "/conformity_assessments",
    request_body = ConformityAssessmentRequest,
    responses(
        (status = 200, description = "Assessment created/updated successfully", body = ConformityAssessmentResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    ),
    tag = "Compliance"
)]
pub async fn create_conformity_assessment(
    req: web::Json<ConformityAssessmentRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "compliance", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let assessment_id = format!("ASSESS-{}", uuid::Uuid::new_v4().to_string().replace("-", "").chars().take(12).collect::<String>());
    let assessment_req = req.into_inner();
    
    let assessment_date = chrono::NaiveDateTime::parse_from_str(&assessment_req.assessment_date, "%Y-%m-%d %H:%M:%S")
        .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        .unwrap_or(Utc::now());
    
    let expiration_date = assessment_req.expiration_date.and_then(|d| {
        chrono::NaiveDateTime::parse_from_str(&d, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    });
    
    // Determine status based on expiration
    let status = if let Some(exp) = expiration_date {
        if exp < Utc::now() {
            "EXPIRED"
        } else {
            "PASSED"
        }
    } else {
        "PASSED"
    }.to_string();
    
    let result = sqlx::query(
        "INSERT INTO conformity_assessments (
            assessment_id, system_id, system_name, assessment_type,
            assessment_date, expiration_date, status, assessment_result,
            assessor_name, assessor_contact, certificate_number, notes
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        ON CONFLICT (assessment_id) DO UPDATE SET
            system_name = EXCLUDED.system_name,
            assessment_type = EXCLUDED.assessment_type,
            assessment_date = EXCLUDED.assessment_date,
            expiration_date = EXCLUDED.expiration_date,
            status = EXCLUDED.status,
            assessment_result = EXCLUDED.assessment_result,
            assessor_name = EXCLUDED.assessor_name,
            assessor_contact = EXCLUDED.assessor_contact,
            certificate_number = EXCLUDED.certificate_number,
            notes = EXCLUDED.notes,
            updated_at = CURRENT_TIMESTAMP"
    )
    .bind(&assessment_id)
    .bind(&assessment_req.system_id)
    .bind(&assessment_req.system_name)
    .bind(&assessment_req.assessment_type)
    .bind(assessment_date)
    .bind(expiration_date)
    .bind(&status)
    .bind(&assessment_req.assessment_result)
    .bind(&assessment_req.assessor_name)
    .bind(&assessment_req.assessor_contact)
    .bind(&assessment_req.certificate_number)
    .bind(&assessment_req.notes)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(_) => {
            let days_until_expiration = expiration_date.map(|exp| {
                (exp - Utc::now()).num_days()
            });
            
            // Check if expiration is within 30 days and send notification
            if let Some(days) = days_until_expiration {
                if days <= 30 && days > 0 {
                    // Send notification about expiring assessment
                    let notification_service = data.notification_service.clone();
                    let db_pool = data.db_pool.clone();
                    let system_name_clone = assessment_req.system_name.clone();
                    let assessment_id_clone = assessment_id.clone();
                    
                    tokio::spawn(async move {
                        // Notify compliance team (using system admin user or default)
                        let _ = notification_service.send_notification(
                            &db_pool,
                            crate::integration::notifications::NotificationRequest {
                                user_id: "SYSTEM_ADMIN".to_string(),
                                notification_type: crate::integration::notifications::NotificationType::HighRiskAiAction,
                                channel: crate::integration::notifications::NotificationChannel::Email,
                                subject: Some(format!("Conformity Assessment Expiring Soon - {}", system_name_clone)),
                                body: format!(
                                    "The conformity assessment for AI system '{}' will expire in {} days.\n\nAssessment ID: {}\n\nPlease schedule a new assessment before expiration.\n\nBest regards,\nVeridion Nexus Compliance Team",
                                    system_name_clone, days, assessment_id_clone
                                ),
                                language: Some("en".to_string()),
                                related_entity_type: Some("CONFORMITY_ASSESSMENT".to_string()),
                                related_entity_id: Some(assessment_id_clone),
                            }
                        ).await;
                    });
                }
            }
            
            HttpResponse::Ok().json(ConformityAssessmentResponse {
                assessment_id,
                system_id: assessment_req.system_id,
                system_name: assessment_req.system_name,
                assessment_type: assessment_req.assessment_type,
                assessment_date: assessment_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                expiration_date: expiration_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
                status,
                days_until_expiration,
                assessor_name: assessment_req.assessor_name,
                certificate_number: assessment_req.certificate_number,
            })
        }
        Err(e) => {
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("creating conformity assessment", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create conformity assessment"
            }))
        }
    }
}

/// Get all conformity assessments
#[utoipa::path(
    get,
    path = "/conformity_assessments",
    params(
        ("system_id" = Option<String>, Query, description = "Filter by system_id"),
        ("status" = Option<String>, Query, description = "Filter by status")
    ),
    responses(
        (status = 200, description = "Assessments retrieved successfully", body = Vec<ConformityAssessmentResponse>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Compliance"
)]
pub async fn get_conformity_assessments(
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "compliance", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let system_id_filter = query.get("system_id");
    let status_filter = query.get("status");

    let mut query_str = "SELECT assessment_id, system_id, system_name, assessment_type, assessment_date, expiration_date, status, assessor_name, certificate_number FROM conformity_assessments WHERE 1=1".to_string();
    let mut bindings = Vec::new();
    
    if let Some(sid) = system_id_filter {
        query_str.push_str(" AND system_id = $1");
        bindings.push(sid.clone());
    }
    
    if let Some(st) = status_filter {
        let param_num = if system_id_filter.is_some() { 2 } else { 1 };
        query_str.push_str(&format!(" AND status = ${}", param_num));
        bindings.push(st.clone());
    }
    
    query_str.push_str(" ORDER BY assessment_date DESC");

    #[derive(sqlx::FromRow)]
    struct AssessmentRow {
        assessment_id: String,
        system_id: String,
        system_name: String,
        assessment_type: String,
        assessment_date: chrono::DateTime<chrono::Utc>,
        expiration_date: Option<chrono::DateTime<chrono::Utc>>,
        status: String,
        assessor_name: Option<String>,
        certificate_number: Option<String>,
    }

    let result = if system_id_filter.is_some() && status_filter.is_some() {
        sqlx::query_as::<_, AssessmentRow>(&query_str)
            .bind(&bindings[0])
            .bind(&bindings[1])
            .fetch_all(&data.db_pool)
            .await
    } else if system_id_filter.is_some() {
        sqlx::query_as::<_, AssessmentRow>(&query_str)
            .bind(&bindings[0])
            .fetch_all(&data.db_pool)
            .await
    } else if status_filter.is_some() {
        sqlx::query_as::<_, AssessmentRow>(&query_str)
            .bind(&bindings[0])
            .fetch_all(&data.db_pool)
            .await
    } else {
        sqlx::query_as::<_, AssessmentRow>(&query_str)
            .fetch_all(&data.db_pool)
            .await
    };

    match result {
        Ok(assessments) => {
            let responses: Vec<ConformityAssessmentResponse> = assessments.into_iter().map(|a| {
                let days_until_expiration = a.expiration_date.map(|exp| {
                    (exp - Utc::now()).num_days()
                });
                
                ConformityAssessmentResponse {
                    assessment_id: a.assessment_id,
                    system_id: a.system_id,
                    system_name: a.system_name,
                    assessment_type: a.assessment_type,
                    assessment_date: a.assessment_date.format("%Y-%m-%d %H:%M:%S").to_string(),
                    expiration_date: a.expiration_date.map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string()),
                    status: a.status,
                    days_until_expiration,
                    assessor_name: a.assessor_name,
                    certificate_number: a.certificate_number,
                }
            }).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => {
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("fetching conformity assessments", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch conformity assessments"
            }))
        }
    }
}

// ============================================================================
// Priority 3: EU AI Act Article 11 - Data Governance Extension
// ============================================================================

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DataQualityMetricRequest {
    pub seal_id: String,
    pub data_source: Option<String>,
    pub metric_type: String, // 'COMPLETENESS', 'ACCURACY', 'CONSISTENCY', 'VALIDITY', 'TIMELINESS'
    pub metric_value: f64,
    pub threshold: Option<f64>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DataBiasDetectionRequest {
    pub seal_id: String,
    pub bias_type: String, // 'DEMOGRAPHIC', 'GEOGRAPHIC', 'TEMPORAL', 'REPRESENTATION'
    pub bias_metric: f64,
    pub affected_groups: serde_json::Value,
    pub mitigation_applied: Option<bool>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DataLineageRequest {
    pub seal_id: String,
    pub source_seal_id: Option<String>,
    pub transformation_type: Option<String>,
    pub transformation_details: Option<serde_json::Value>,
}

/// Record data quality metric
#[utoipa::path(
    post,
    path = "/data_quality/metrics",
    request_body = DataQualityMetricRequest,
    responses(
        (status = 200, description = "Metric recorded successfully"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Data Governance"
)]
pub async fn record_data_quality_metric(
    req: web::Json<DataQualityMetricRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "compliance", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let result = sqlx::query(
        "INSERT INTO data_quality_metrics (
            seal_id, data_source, metric_type, metric_value, threshold
        ) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&req.seal_id)
    .bind(&req.data_source)
    .bind(&req.metric_type)
    .bind(req.metric_value as f64)
    .bind(req.threshold.map(|t| t as f64))
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(_) => {
            // Check if metric is below threshold
            if let Some(threshold) = req.threshold {
                if req.metric_value < threshold {
                    println!("⚠️ Data quality alert: {} metric {} is below threshold {}", 
                        req.metric_type, req.metric_value, threshold);
                }
            }
            HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "message": "Data quality metric recorded"
            }))
        }
        Err(e) => {
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("recording data quality metric", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to record metric"
            }))
        }
    }
}

/// Record data bias detection
#[utoipa::path(
    post,
    path = "/data_quality/bias",
    request_body = DataBiasDetectionRequest,
    responses(
        (status = 200, description = "Bias detection recorded successfully"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Data Governance"
)]
pub async fn record_data_bias(
    req: web::Json<DataBiasDetectionRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "compliance", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let result = sqlx::query(
        "INSERT INTO data_bias_detections (
            seal_id, bias_type, bias_metric, affected_groups, mitigation_applied
        ) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&req.seal_id)
    .bind(&req.bias_type)
    .bind(req.bias_metric as f64)
    .bind(&req.affected_groups)
    .bind(req.mitigation_applied.unwrap_or(false))
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(_) => {
            println!("⚠️ Data bias detected: {} bias (metric: {}) for seal: {}", 
                req.bias_type, req.bias_metric, req.seal_id);
            HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "message": "Bias detection recorded"
            }))
        }
        Err(e) => {
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("recording bias detection", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to record bias detection"
            }))
        }
    }
}

/// Record data lineage
#[utoipa::path(
    post,
    path = "/data_quality/lineage",
    request_body = DataLineageRequest,
    responses(
        (status = 200, description = "Lineage recorded successfully"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Data Governance"
)]
pub async fn record_data_lineage(
    req: web::Json<DataLineageRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "compliance", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    // Build lineage path
    let mut lineage_path = Vec::new();
    if let Some(ref source) = req.source_seal_id {
        lineage_path.push(source.clone());
    }
    lineage_path.push(req.seal_id.clone());

    let result = sqlx::query(
        "INSERT INTO data_lineage (
            seal_id, source_seal_id, transformation_type, transformation_details, lineage_path
        ) VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(&req.seal_id)
    .bind(&req.source_seal_id)
    .bind(&req.transformation_type)
    .bind(&req.transformation_details)
    .bind(&lineage_path)
    .execute(&data.db_pool)
    .await;

    match result {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "message": "Data lineage recorded"
            }))
        }
        Err(e) => {
            let request_id = generate_request_id();
            let request_id = generate_request_id();
            log_error_safely("recording data lineage", &e, &request_id);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to record lineage"
            }))
        }
    }
}

/// Get data quality report for a seal
#[utoipa::path(
    get,
    path = "/data_quality/report/{seal_id}",
    responses(
        (status = 200, description = "Quality report retrieved successfully"),
        (status = 401, description = "Unauthorized")
    ),
    tag = "Data Governance"
)]
pub async fn get_data_quality_report(
    seal_id: web::Path<String>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "compliance", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let seal_id_str = seal_id.into_inner();

    // Get quality metrics
    #[derive(sqlx::FromRow)]
    struct QualityRow {
        metric_type: String,
        metric_value: f64,
        measured_at: chrono::DateTime<chrono::Utc>,
    }

    let metrics: Vec<QualityRow> = sqlx::query_as(
        "SELECT metric_type, metric_value, measured_at 
         FROM data_quality_metrics 
         WHERE seal_id = $1 
         ORDER BY measured_at DESC"
    )
    .bind(&seal_id_str)
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    // Get bias detections
    #[derive(sqlx::FromRow)]
    struct BiasRow {
        bias_type: String,
        bias_metric: f64,
        affected_groups: serde_json::Value,
        mitigation_applied: bool,
        detected_at: chrono::DateTime<chrono::Utc>,
    }

    let biases: Vec<BiasRow> = sqlx::query_as(
        "SELECT bias_type, bias_metric, affected_groups, mitigation_applied, detected_at 
         FROM data_bias_detections 
         WHERE seal_id = $1 
         ORDER BY detected_at DESC"
    )
    .bind(&seal_id_str)
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    // Get lineage
    #[derive(sqlx::FromRow)]
    struct LineageRow {
        source_seal_id: Option<String>,
        transformation_type: Option<String>,
        lineage_path: Vec<String>,
    }

    let lineage: Vec<LineageRow> = sqlx::query_as(
        "SELECT source_seal_id, transformation_type, lineage_path 
         FROM data_lineage 
         WHERE seal_id = $1"
    )
    .bind(&seal_id_str)
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    HttpResponse::Ok().json(serde_json::json!({
        "seal_id": seal_id_str,
        "quality_metrics": metrics.iter().map(|m| serde_json::json!({
            "type": m.metric_type,
            "value": m.metric_value,
            "measured_at": m.measured_at.format("%Y-%m-%d %H:%M:%S").to_string()
        })).collect::<Vec<_>>(),
        "bias_detections": biases.iter().map(|b| serde_json::json!({
            "type": b.bias_type,
            "metric": b.bias_metric,
            "affected_groups": b.affected_groups,
            "mitigation_applied": b.mitigation_applied,
            "detected_at": b.detected_at.format("%Y-%m-%d %H:%M:%S").to_string()
        })).collect::<Vec<_>>(),
        "lineage": lineage.iter().map(|l| serde_json::json!({
            "source_seal_id": l.source_seal_id,
            "transformation_type": l.transformation_type,
            "lineage_path": l.lineage_path
        })).collect::<Vec<_>>()
    }))
}

// PROXY MODE - Network-level compliance enforcement
// Endpoint: POST /api/v1/proxy
#[utoipa::path(
    post,
    path = "/proxy",
    request_body = crate::integration::proxy::ProxyRequest,
    responses(
        (status = 200, description = "Request proxied successfully"),
        (status = 403, description = "Sovereignty violation - blocked"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Proxy error")
    ),
    tag = "Proxy Mode"
)]
pub async fn proxy_request(
    req: HttpRequest,
    body: web::Json<crate::integration::proxy::ProxyRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    use crate::integration::proxy::ProxyService;
    
    let proxy_req = body.into_inner();
    let proxy_service = ProxyService::new();

    // 1. Check data sovereignty BEFORE forwarding
    match proxy_service.check_sovereignty(&proxy_req.target_url).await {
        Ok((is_eu, country)) => {
            if !is_eu {
                // Log the violation attempt
                let agent_id = req.headers()
                    .get("X-Agent-ID")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("unknown")
                    .to_string();
                
                let user_id = extract_claims(&req, &crate::security::AuthService::new().unwrap())
                    .ok()
                    .map(|c| c.sub);

                // Log compliance action
                let _ = sqlx::query(
                    "INSERT INTO compliance_records (
                        seal_id, tx_id, agent_id, action_summary, status, 
                        risk_level, user_id, timestamp, target_region
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
                )
                .bind(&uuid::Uuid::new_v4().to_string())
                .bind(&uuid::Uuid::new_v4().to_string())
                .bind(&agent_id)
                .bind(&format!("PROXY_BLOCKED: Attempted connection to {} ({})", proxy_req.target_url, country))
                .bind("BLOCKED (SOVEREIGNTY)")
                .bind("HIGH")
                .bind(user_id.as_deref())
                .bind(chrono::Utc::now())
                .bind(&country)
                .execute(&data.db_pool)
                .await;

                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "SOVEREIGN_LOCK_VIOLATION",
                    "message": format!("Data sovereignty violation: target server in {} is not in EU/EEA", country),
                    "target_url": proxy_req.target_url,
                    "detected_country": country,
                    "status": "BLOCKED"
                }));
            }
        }
        Err(e) => {
            // If we can't determine sovereignty, be conservative and block
            log::warn!("Could not determine sovereignty for {}: {}", proxy_req.target_url, e);
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "SOVEREIGNTY_CHECK_FAILED",
                "message": "Could not verify data sovereignty. Request blocked for safety.",
                "target_url": proxy_req.target_url
            }));
        }
    }

    // 2. Forward request to target
    match proxy_service.forward_request(&proxy_req).await {
        Ok(response) => {
            let status = response.status();
            let headers = response.headers().clone();
            
            // Get response body
            let body_bytes = match response.bytes().await {
                Ok(bytes) => bytes,
                Err(e) => {
                    log_error_safely("reading proxy response", &e, &generate_request_id());
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to read response from target server"
                    }));
                }
            };

            // 3. Log successful proxy action (async, non-blocking)
            let db_pool = data.db_pool.clone();
            let target_url = proxy_req.target_url.clone();
            let agent_id = req.headers()
                .get("X-Agent-ID")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("unknown")
                .to_string();
            
            tokio::spawn(async move {
                let _ = sqlx::query(
                    "INSERT INTO compliance_records (
                        seal_id, tx_id, agent_id, action_summary, status, 
                        risk_level, timestamp, target_region
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
                )
                .bind(&uuid::Uuid::new_v4().to_string())
                .bind(&uuid::Uuid::new_v4().to_string())
                .bind(&agent_id)
                .bind(&format!("PROXY_ALLOWED: {}", target_url))
                .bind("COMPLIANT")
                .bind("LOW")
                .bind(chrono::Utc::now())
                .bind("EU")
                .execute(&db_pool)
                .await;
            });

            // Build response with original headers and body
            let mut http_response = HttpResponse::build(status);
            
            // Copy relevant headers (exclude connection, content-encoding, etc.)
            for (key, value) in headers.iter() {
                let key_str = key.as_str();
                if !matches!(key_str, "connection" | "content-encoding" | "transfer-encoding" | "content-length") {
                    if let Ok(val_str) = value.to_str() {
                        http_response.append_header((key_str, val_str));
                    }
                }
            }

            http_response.body(body_bytes)
        }
        Err(e) => {
            let request_id = generate_request_id();
            log_error_safely("forwarding proxy request", &e, &request_id);
            HttpResponse::BadGateway().json(serde_json::json!({
                "error": "PROXY_ERROR",
                "message": "Failed to forward request to target server",
                "request_id": request_id
            }))
        }
    }
}
