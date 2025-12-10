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
    let auth_service = AuthService::new()
        .map_err(|e| HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to initialize auth service: {}", e)
        })))?;
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
use std::collections::HashMap;
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

    // A. AGENT REVOCATION CHECK (granular per agent)
    if data.is_agent_revoked(&req.agent_id) {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "status": "AGENT_REVOKED",
            "reason": "Agent access has been revoked",
            "agent_id": req.agent_id
        }));
    }

    // B. GLOBAL SYSTEM LOCKDOWN CHECK (panic button)
    if let Ok(true) = data.is_locked_down().await {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "status": "SYSTEM_LOCKDOWN",
            "reason": "System-wide lockdown active"
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

    // 0. CHECK ENFORCEMENT MODE (Shadow Mode Infrastructure)
    // Get system-wide enforcement mode
    let system_enforcement_mode: String = sqlx::query_scalar("SELECT get_system_enforcement_mode()")
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or_else(|_| "ENFORCING".to_string());
    
    // Check if policy has override
    let policy_enforcement_mode: Option<String> = sqlx::query_scalar(
        "SELECT enforcement_mode_override FROM policy_versions 
         WHERE policy_type = 'SOVEREIGN_LOCK' AND is_active = true 
         LIMIT 1"
    )
    .fetch_optional(&data.db_pool)
    .await
    .ok()
    .flatten();
    
    // Use policy override if available, otherwise system mode
    let enforcement_mode = policy_enforcement_mode.as_deref()
        .unwrap_or(&system_enforcement_mode);
    
    let is_shadow_mode = enforcement_mode == "SHADOW" || enforcement_mode == "DRY_RUN";

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
    
    // In shadow mode, log what would happen but don't block
    if is_shadow_mode && is_violation {
        // Log to shadow_mode_logs instead of blocking
        let record_id = uuid::Uuid::new_v4();
        let log_hash = crate::core::privacy_bridge::hash_payload(&req.payload);
        let _ = sqlx::query(
            "INSERT INTO shadow_mode_logs (
                id, agent_id, action_summary, action_type, payload_hash,
                target_region, would_block, would_allow, policy_applied,
                risk_level, detected_country, timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
        )
        .bind(record_id)
        .bind(&req.agent_id)
        .bind(&format!("{}: {}", req.agent_id, req.action))
        .bind(&req.action)
        .bind(&log_hash)
        .bind(&target)
        .bind(true)  // would_block
        .bind(false) // would_allow
        .bind("SOVEREIGN_LOCK")
        .bind("HIGH")
        .bind(&target)
        .bind(chrono::Utc::now())
        .execute(&data.db_pool)
        .await;
        
        log::warn!("SHADOW MODE: Would block {} -> {} (Country: {})", req.agent_id, req.action, target);
        
        // Send shadow mode alert (async, don't block)
        let notification_service = crate::integration::notifications::NotificationService::new();
        let agent_id_clone = req.agent_id.clone();
        let action_clone = req.action.clone();
        let target_clone = target.clone();
        let db_pool_clone = data.db_pool.clone();
        tokio::spawn(async move {
            let _ = notification_service.send_shadow_mode_alert(
                &db_pool_clone,
                &agent_id_clone,
                &action_clone,
                &target_clone,
                "SOVEREIGN_LOCK",
                None, // Will use system user
            ).await;
        });
        
        // Continue processing - don't block in shadow mode
    } else if is_shadow_mode && !is_violation {
        // Log allowed actions in shadow mode too
        let record_id = uuid::Uuid::new_v4();
        let log_hash = crate::core::privacy_bridge::hash_payload(&req.payload);
        let _ = sqlx::query(
            "INSERT INTO shadow_mode_logs (
                id, agent_id, action_summary, action_type, payload_hash,
                target_region, would_block, would_allow, policy_applied,
                risk_level, detected_country, timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
        )
        .bind(record_id)
        .bind(&req.agent_id)
        .bind(&format!("{}: {}", req.agent_id, req.action))
        .bind(&req.action)
        .bind(&log_hash)
        .bind(&target)
        .bind(false) // would_block
        .bind(true)  // would_allow
        .bind("SOVEREIGN_LOCK")
        .bind("LOW")
        .bind(&target)
        .bind(chrono::Utc::now())
        .execute(&data.db_pool)
        .await;
    }
    
    // If in shadow mode and violation, skip blocking but continue logging
    if is_shadow_mode && is_violation {
        // Don't return early - continue to log but mark as shadow mode
        // We'll handle this after risk assessment
    }

    // C. ASSET-BASED POLICY ENGINE
    // Get asset context (business function, location, risk profile) from agent_id
    let asset_context = crate::core::asset_policy_engine::AssetPolicyEngine::get_asset_context_from_agent(
        &data.db_pool,
        &req.agent_id,
    ).await.ok().flatten();

    // If asset not found, try to infer business function from action
    let business_function = if let Some(ctx) = &asset_context {
        ctx.business_function.clone()
    } else {
        crate::core::asset_policy_engine::AssetPolicyEngine::infer_business_function_from_action(&req.action)
    };

    // Get applicable asset-based policies
    let asset_policies = if let Some(ctx) = &asset_context {
        crate::core::asset_policy_engine::AssetPolicyEngine::get_applicable_policies(
            &data.db_pool,
            ctx,
        ).await.ok().unwrap_or_default()
    } else if let Some(bf) = &business_function {
        // Create temporary context from inferred business function
        let temp_context = crate::core::asset_policy_engine::AssetContext {
            asset_id: None,
            business_function: Some(bf.clone()),
            department: None,
            location: req.target_region.clone(),
            risk_profile: None,
            tags: None,
        };
        crate::core::asset_policy_engine::AssetPolicyEngine::get_applicable_policies(
            &data.db_pool,
            &temp_context,
        ).await.ok().unwrap_or_default()
    } else {
        Vec::new()
    };

    // C. RISK ASSESSMENT (EU AI Act Article 9)
    // Enhanced risk assessment using context-aware methodology (now with business function context)
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

    // In shadow mode, never block - always return OK but mark status appropriately
    if is_violation && !is_shadow_mode {
        HttpResponse::Forbidden().json(LogResponse {
            status: status.to_string(),
            seal_id: "N/A (Connection Refused)".to_string(),
            tx_id: "0000".to_string(),
            risk_level: Some(risk_level),
            human_oversight_status,
        })
    } else {
        // Shadow mode: return OK even if violation (already logged to shadow_mode_logs)
        let final_status = if is_shadow_mode && is_violation {
            format!("SHADOW_MODE: {}", status)
        } else {
            status.to_string()
        };
        
        HttpResponse::Ok().json(LogResponse {
            status: final_status,
            seal_id: if is_shadow_mode && is_violation { 
                format!("SHADOW-{}", seal_id) 
            } else { 
                seal_id 
            },
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
        ("seal_id" = Option<String>, Query, description = "Filter by specific seal_id"),
        ("report_type" = Option<String>, Query, description = "Report type: annex_iv, dora_register (default: annex_iv)")
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
    
    let report_type = query.get("report_type").map(|s| s.as_str()).unwrap_or("annex_iv");
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
            
            // Determine filename, temp_file, and generation function based on report_type
            let (filename, temp_file, export_result) = match report_type {
                "dora_register" => {
                    // DORA Register: Simplified vendor supply chain report (PDF only for now)
                    let filename = format!("Veridion_DORA_Register_{}.pdf", 
                        chrono::Utc::now().format("%Y%m%d_%H%M%S")
                    );
                    let temp_file = format!("temp_report_{}.pdf", 
                        uuid::Uuid::new_v4().to_string().replace("-", "")
                    );
                    
                    let result = crate::core::annex_iv::generate_dora_register(&compliance_records, &temp_file);
                    (filename, temp_file, result)
                }
                _ => {
                    // Default: Annex IV report
            let filename = format!("Veridion_Annex_IV_{}.{}", 
                chrono::Utc::now().format("%Y%m%d_%H%M%S"),
                format.file_extension()
            );
            let temp_file = format!("temp_report_{}.{}", 
                uuid::Uuid::new_v4().to_string().replace("-", ""),
                format.file_extension()
            );
            
                    let result = match format {
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
                    (filename, temp_file, result)
                }
            };
            
            match export_result {
                Ok(_) => {
                    if let Ok(bytes) = fs::read(&temp_file) {
                        // Clean up temp file
                        let _ = fs::remove_file(&temp_file);
                        
                        // Determine content type based on report type
                        let content_type = if report_type == "dora_register" {
                            "application/pdf"
                        } else {
                            format.content_type()
                        };
                        
                        HttpResponse::Ok()
                            .content_type(content_type)
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
#[derive(Deserialize, ToSchema)]
pub struct RevokeAccessRequest {
    /// Optional agent_id to revoke. If not provided, triggers global system lockdown.
    #[schema(example = "agent-001")]
    pub agent_id: Option<String>,
}

#[utoipa::path(
    post,
    path = "/revoke_access",
    request_body = RevokeAccessRequest,
    responses(
        (status = 200, description = "Access revoked successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "System Management"
)]
pub async fn revoke_access(
    req: Option<web::Json<RevokeAccessRequest>>,
    data: web::Data<AppState>,
) -> impl Responder {
    // Check if agent_id is provided in request body
    if let Some(body) = req {
        if let Some(agent_id) = &body.agent_id {
            // Granular revocation: revoke specific agent
            data.revoke_agent(agent_id);
            return HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "message": format!("Agent '{}' access revoked", agent_id),
                "agent_id": agent_id
            }));
        }
    }

    // No agent_id provided: trigger global system lockdown (panic button)
    match data.set_locked_down(true).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "SUCCESS",
            "message": "Global system lockdown activated"
        })),
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
    let auth_service = match AuthService::new() {
        Ok(service) => service,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to initialize auth service: {}", e)
            }));
        }
    };
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

    // 0. Check policy state (test mode, rollout percentage, circuit breaker)
    #[derive(sqlx::FromRow)]
    struct PolicyState {
        id: uuid::Uuid,
        is_test_mode: bool,
        rollout_percentage: i32,
        circuit_breaker_enabled: bool,
        circuit_breaker_state: Option<String>,
        circuit_breaker_error_threshold: Option<f64>,
    }
    
    let policy_state: Option<PolicyState> = sqlx::query_as(
        "SELECT id, COALESCE(is_test_mode, false) as is_test_mode, 
                COALESCE(rollout_percentage, 100) as rollout_percentage,
                COALESCE(circuit_breaker_enabled, false) as circuit_breaker_enabled,
                circuit_breaker_state,
                circuit_breaker_error_threshold
         FROM policy_versions 
         WHERE policy_type = 'SOVEREIGN_LOCK' AND is_active = true 
         LIMIT 1"
    )
    .fetch_optional(&data.db_pool)
    .await
    .ok()
    .flatten();
    
    let is_test_mode = policy_state.as_ref().map(|p| p.is_test_mode).unwrap_or(false);
    let rollout_percentage = policy_state.as_ref().map(|p| p.rollout_percentage).unwrap_or(100);
    let circuit_breaker_enabled = policy_state.as_ref().map(|p| p.circuit_breaker_enabled).unwrap_or(false);
    let circuit_breaker_state = policy_state.as_ref()
        .and_then(|p| p.circuit_breaker_state.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("CLOSED");
    let policy_version_id = policy_state.as_ref().map(|p| p.id);
    
    // Check circuit breaker state - if OPEN, skip policy enforcement
    if circuit_breaker_enabled && circuit_breaker_state == "OPEN" {
        log::warn!("CIRCUIT BREAKER OPEN: Skipping policy enforcement for {}", proxy_req.target_url);
        // Track this as a bypass (not an error, but a circuit breaker bypass)
        if let Some(pv_id) = policy_version_id {
            let _ = sqlx::query(
                "INSERT INTO policy_error_tracking (
                    policy_version_id, policy_type, error_type, 
                    error_count, total_requests, error_rate, 
                    window_start, window_end
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                ON CONFLICT DO NOTHING"
            )
            .bind(pv_id)
            .bind("SOVEREIGN_LOCK")
            .bind("CIRCUIT_BYPASS")
            .bind(0) // Not an error, just bypass
            .bind(1) // Total request
            .bind(0.0) // Error rate
            .bind(chrono::Utc::now())
            .bind(chrono::Utc::now())
            .execute(&data.db_pool)
            .await;
        }
        // Continue without policy enforcement
    }
    
    // Check if this request should be subject to policy (gradual rollout)
    let should_apply_policy = if rollout_percentage < 100 {
        // Use agent_id as deterministic seed for rollout
        let agent_id = req.headers()
            .get("X-Agent-ID")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown");
        // Simple hash-based rollout (consistent per agent)
        let hash = agent_id.chars().map(|c| c as u32).sum::<u32>();
        (hash % 100) < (rollout_percentage as u32)
    } else {
        true // 100% rollout - apply to all
    };

    // 1. Check data sovereignty BEFORE forwarding
    let detected_country = match proxy_service.check_sovereignty(&proxy_req.target_url).await {
        Ok((is_eu, country)) => {
            if !is_eu {
                // Skip policy enforcement if not in rollout percentage
                if !should_apply_policy {
                    log::debug!("ROLLOUT: Skipping policy for {} (rollout: {}%)", proxy_req.target_url, rollout_percentage);
                    // Continue to forward - not in rollout yet, return country and proceed
                } else if is_test_mode {
                    log::warn!("TEST MODE: Would block {} -> {} (Country: {})", proxy_req.target_url, country, country);
                    // Log test mode action but continue
                    let agent_id = req.headers()
                        .get("X-Agent-ID")
                        .and_then(|h| h.to_str().ok())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    let record_id = uuid::Uuid::new_v4();
                    let seal_id = uuid::Uuid::new_v4().to_string();
                    let tx_id = uuid::Uuid::new_v4().to_string();
                    let action_summary = format!("TEST_MODE: Would block {} ({})", proxy_req.target_url, country);
                    let _ = sqlx::query(
                        "INSERT INTO compliance_records (
                            id, seal_id, tx_id, agent_id, action_summary, status, 
                            risk_level, timestamp, payload_hash
                        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
                    )
                    .bind(record_id)
                    .bind(&seal_id)
                    .bind(&tx_id)
                    .bind(&agent_id)
                    .bind(&action_summary)
                    .bind("TEST_MODE (WOULD_BLOCK)")
                    .bind("HIGH")
                    .bind(chrono::Utc::now())
                    .bind(&format!("region:{}", country))
                    .execute(&data.db_pool)
                    .await;
                    // Continue to forward in test mode - return country and proceed
                    // Note: This is inside a match arm, so we just continue to the end
                } else {
                // Log the violation attempt
                let agent_id = req.headers()
                    .get("X-Agent-ID")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("unknown")
                    .to_string();
                
                let user_id = crate::security::AuthService::new()
                    .ok()
                    .and_then(|auth_service| extract_claims(&req, &auth_service).ok())
                    .map(|c| c.sub);

                // Log compliance action
                let record_id = uuid::Uuid::new_v4();
                let seal_id = uuid::Uuid::new_v4().to_string();
                let tx_id = uuid::Uuid::new_v4().to_string();
                let action_summary = format!("PROXY_BLOCKED: Attempted connection to {} ({})", proxy_req.target_url, country);
                if let Err(e) = sqlx::query(
                    "INSERT INTO compliance_records (
                        id, seal_id, tx_id, agent_id, action_summary, status, 
                        risk_level, user_id, timestamp, payload_hash
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
                )
                .bind(record_id)
                .bind(&seal_id)
                .bind(&tx_id)
                .bind(&agent_id)
                .bind(&action_summary)
                .bind("BLOCKED (SOVEREIGNTY)")
                .bind("HIGH")
                .bind(user_id.as_deref())
                .bind(chrono::Utc::now())
                .bind(&format!("region:{}", country))
                .execute(&data.db_pool)
                .await {
                    log::error!("Failed to log proxy block: {}", e);
                }
                
                // Track error for circuit breaker (if enabled)
                if let Some(pv_id) = policy_version_id {
                    let now = chrono::Utc::now();
                    let window_start = now - chrono::Duration::minutes(5);
                    
                    // Update error tracking
                    let _ = sqlx::query(
                        "INSERT INTO policy_error_tracking (
                            policy_version_id, policy_type, error_type,
                            error_count, total_requests, error_rate,
                            window_start, window_end
                        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                        ON CONFLICT DO UPDATE SET
                            error_count = policy_error_tracking.error_count + 1,
                            total_requests = policy_error_tracking.total_requests + 1,
                            error_rate = ((policy_error_tracking.error_count + 1)::DECIMAL / 
                                         (policy_error_tracking.total_requests + 1)::DECIMAL) * 100.0,
                            updated_at = CURRENT_TIMESTAMP"
                    )
                    .bind(pv_id)
                    .bind("SOVEREIGN_LOCK")
                    .bind("BLOCKED")
                    .bind(1)
                    .bind(1)
                    .bind(100.0) // 100% error rate for blocked requests
                    .bind(window_start)
                    .bind(now)
                    .execute(&data.db_pool)
                    .await;
                    
                    // Check if circuit breaker should open
                    let should_open: Option<bool> = sqlx::query_scalar(
                        "SELECT should_open_circuit_breaker($1)"
                    )
                    .bind(pv_id)
                    .fetch_optional(&data.db_pool)
                    .await
                    .ok()
                    .flatten();
                    
                    if should_open.unwrap_or(false) {
                        // Open circuit breaker
                        let _ = sqlx::query(
                            "UPDATE policy_versions 
                             SET circuit_breaker_state = 'OPEN',
                                 circuit_breaker_opened_at = CURRENT_TIMESTAMP,
                                 circuit_breaker_last_error_at = CURRENT_TIMESTAMP
                             WHERE id = $1"
                        )
                        .bind(pv_id)
                        .execute(&data.db_pool)
                        .await;
                        
                        // Log circuit breaker history
                        let _ = sqlx::query(
                            "INSERT INTO circuit_breaker_history (
                                policy_version_id, state_transition, error_rate,
                                error_count, total_requests, triggered_by, notes
                            ) VALUES ($1, $2, $3, $4, $5, $6, $7)"
                        )
                        .bind(pv_id)
                        .bind("OPENED")
                        .bind(100.0)
                        .bind(1)
                        .bind(1)
                        .bind("THRESHOLD")
                        .bind(format!("Auto-opened due to error rate threshold"))
                        .execute(&data.db_pool)
                        .await;
                        
                        log::error!("CIRCUIT BREAKER OPENED: Policy {} exceeded error threshold", pv_id);
                        
                        // Send circuit breaker alert
                        let notification_service = crate::integration::notifications::NotificationService::new();
                        let policy_id_str = pv_id.to_string();
                        let db_pool_clone = data.db_pool.clone();
                        tokio::spawn(async move {
                            let _ = notification_service.send_circuit_breaker_alert(
                                &db_pool_clone,
                                &policy_id_str,
                                "SOVEREIGN_LOCK",
                                100.0,
                                1,
                                1,
                                None,
                            ).await;
                        });
                    }
                    
                    // Track canary metrics for blocked requests
                    let rollout_pct = rollout_percentage;
                    let now = chrono::Utc::now();
                    let window_start = now - chrono::Duration::minutes(10);
                    
                    let _ = sqlx::query(
                        "INSERT INTO canary_metrics (
                            policy_version_id, traffic_percentage, total_requests,
                            successful_requests, failed_requests, blocked_requests,
                            success_rate, window_start, window_end
                        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                        ON CONFLICT (policy_version_id, traffic_percentage, window_start) 
                        DO UPDATE SET
                            total_requests = canary_metrics.total_requests + 1,
                            blocked_requests = canary_metrics.blocked_requests + 1,
                            success_rate = (canary_metrics.successful_requests::DECIMAL / 
                                           (canary_metrics.total_requests + 1)::DECIMAL) * 100.0,
                            updated_at = CURRENT_TIMESTAMP"
                    )
                    .bind(pv_id)
                    .bind(rollout_pct)
                    .bind(1)
                    .bind(0)
                    .bind(0)
                    .bind(1)
                    .bind(0.0)
                    .bind(window_start)
                    .bind(now)
                    .execute(&data.db_pool)
                    .await;
                    
                    // Check if should auto-rollback
                    let should_rollback: Option<bool> = sqlx::query_scalar(
                        "SELECT should_rollback_canary($1)"
                    )
                    .bind(pv_id)
                    .fetch_optional(&data.db_pool)
                    .await
                    .ok()
                    .flatten();
                    
                    if should_rollback.unwrap_or(false) {
                        // Auto-rollback to previous percentage tier
                        let prev_percentage = match rollout_pct {
                            100 => 50,
                            50 => 25,
                            25 => 10,
                            10 => 5,
                            5 => 1,
                            _ => 0,
                        };
                        
                        let success_rate: Option<f64> = sqlx::query_scalar(
                            "SELECT calculate_canary_success_rate($1, $2, 10)"
                        )
                        .bind(pv_id)
                        .bind(rollout_pct)
                        .fetch_optional(&data.db_pool)
                        .await
                        .ok()
                        .flatten();
                        
                        let _ = sqlx::query(
                            "UPDATE policy_versions 
                             SET rollout_percentage = $1
                             WHERE id = $2"
                        )
                        .bind(prev_percentage)
                        .bind(pv_id)
                        .execute(&data.db_pool)
                        .await;
                        
                        // Log rollback
                        let _ = sqlx::query(
                            "INSERT INTO canary_deployment_history (
                                policy_version_id, action, from_percentage, to_percentage,
                                success_rate, triggered_by, notes
                            ) VALUES ($1, $2, $3, $4, $5, $6, $7)"
                        )
                        .bind(pv_id)
                        .bind("ROLLED_BACK")
                        .bind(rollout_pct)
                        .bind(prev_percentage)
                        .bind(success_rate.unwrap_or(0.0))
                        .bind("AUTO")
                        .bind(format!("Auto-rolled back from {}% to {}% due to high failure rate", rollout_pct, prev_percentage))
                        .execute(&data.db_pool)
                        .await;
                        
                        log::warn!("CANARY AUTO-ROLLBACK: Policy {} from {}% to {}%", pv_id, rollout_pct, prev_percentage);
                        
                        // Send rollback notification
                        let notification_service = crate::integration::notifications::NotificationService::new();
                        let policy_name: Option<String> = sqlx::query_scalar(
                            "SELECT policy_name FROM policy_versions WHERE id = $1"
                        )
                        .bind(pv_id)
                        .fetch_optional(&data.db_pool)
                        .await
                        .ok()
                        .flatten();
                        
                        let policy_creator: Option<String> = sqlx::query_scalar(
                            "SELECT created_by FROM policy_versions WHERE id = $1"
                        )
                        .bind(pv_id)
                        .fetch_optional(&data.db_pool)
                        .await
                        .ok()
                        .flatten();
                        
                        let policy_name_str = policy_name.clone().unwrap_or_else(|| "Unknown Policy".to_string());
                        let policy_name_for_webhook = policy_name_str.clone();
                        
                        if let Some(creator) = policy_creator {
                            let db_pool_clone = data.db_pool.clone();
                            let success_rate_val = success_rate.unwrap_or(0.0);
                            tokio::spawn(async move {
                                let request = crate::integration::notifications::NotificationRequest {
                                    user_id: creator,
                                    notification_type: crate::integration::notifications::NotificationType::HighRiskAiAction,
                                    channel: crate::integration::notifications::NotificationChannel::Email,
                                    subject: Some("Policy Auto-Rollback Alert".to_string()),
                                    body: format!(
                                        "A policy has been automatically rolled back due to high failure rate.\n\n\
                                        Policy: {}\n\
                                        Rolled back from: {}% to {}%\n\
                                        Success Rate: {:.1}%\n\
                                        Reason: Success rate below threshold\n\n\
                                        Please review the policy configuration and metrics before attempting to redeploy.",
                                        policy_name_str, rollout_pct, prev_percentage, success_rate_val
                                    ),
                                    language: Some("en".to_string()),
                                    related_entity_type: Some("POLICY".to_string()),
                                    related_entity_id: Some(pv_id.to_string()),
                                };
                                let _ = notification_service.send_notification(&db_pool_clone, request).await;
                            });
                        }
                        
                        // Trigger webhook event
                        let success_rate_for_webhook = success_rate;
                        let db_pool_for_webhook = data.db_pool.clone();
                        tokio::spawn(async move {
                            let webhook_data = serde_json::json!({
                                "policy_id": pv_id,
                                "policy_name": policy_name_for_webhook,
                                "action": "AUTO_ROLLBACK",
                                "from_percentage": rollout_pct,
                                "to_percentage": prev_percentage,
                                "success_rate": success_rate_for_webhook,
                                "reason": "High failure rate detected"
                            });
                            trigger_webhook_event(&db_pool_for_webhook, "policy.rollback.auto", webhook_data).await;
                        });
                    }
                }

                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "SOVEREIGN_LOCK_VIOLATION",
                    "message": format!("Data sovereignty violation: target server in {} is not in EU/EEA", country),
                    "target_url": proxy_req.target_url,
                    "detected_country": country,
                    "status": "BLOCKED"
                }));
                }
            }
            // Store the detected country for logging allowed requests
            Some(country)
        }
        Err(e) => {
            // If we can't determine sovereignty, be conservative and block
            log::warn!("Could not determine sovereignty for {}: {}", proxy_req.target_url, e);
            
            // Log the failed check attempt
            let agent_id = req.headers()
                .get("X-Agent-ID")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("unknown")
                .to_string();
            
            let user_id = crate::security::AuthService::new()
                .ok()
                .and_then(|auth_service| extract_claims(&req, &auth_service).ok())
                .map(|c| c.sub);
            
            // Log compliance action
            let record_id = uuid::Uuid::new_v4();
            let seal_id = uuid::Uuid::new_v4().to_string();
            let tx_id = uuid::Uuid::new_v4().to_string();
            let action_summary = format!("PROXY_BLOCKED: Could not determine sovereignty for {}", proxy_req.target_url);
            if let Err(e) = sqlx::query(
                "INSERT INTO compliance_records (
                    id, seal_id, tx_id, agent_id, action_summary, status, 
                    risk_level, user_id, timestamp, payload_hash
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
            )
            .bind(record_id)
            .bind(&seal_id)
            .bind(&tx_id)
            .bind(&agent_id)
            .bind(&action_summary)
            .bind("BLOCKED (SOVEREIGNTY)")
            .bind("HIGH")
            .bind(user_id.as_deref())
            .bind(chrono::Utc::now())
            .bind("region:UNKNOWN")
            .execute(&data.db_pool)
            .await {
                log::error!("Failed to log proxy block: {}", e);
            }
            
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "SOVEREIGNTY_CHECK_FAILED",
                "message": "Could not verify data sovereignty. Request blocked for safety.",
                "target_url": proxy_req.target_url,
                "detected_country": "UNKNOWN",
                "status": "BLOCKED"
            }));
        }
    };

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
            let country_for_log = detected_country.unwrap_or_else(|| "UNKNOWN".to_string());
            let agent_id = req.headers()
                .get("X-Agent-ID")
                .and_then(|h| h.to_str().ok())
                .unwrap_or("unknown")
                .to_string();
            
            tokio::spawn(async move {
                let record_id = uuid::Uuid::new_v4();
                let seal_id = uuid::Uuid::new_v4().to_string();
                let tx_id = uuid::Uuid::new_v4().to_string();
                let action_summary = format!("PROXY_ALLOWED: {} ({})", target_url, country_for_log);
                
                // Track canary metrics (if policy has rollout_percentage)
                if let Some(pv_id) = policy_version_id {
                    let rollout_pct = rollout_percentage;
                    let now = chrono::Utc::now();
                    let window_start = now - chrono::Duration::minutes(10);
                    
                    let _ = sqlx::query(
                        "INSERT INTO canary_metrics (
                            policy_version_id, traffic_percentage, total_requests,
                            successful_requests, failed_requests, blocked_requests,
                            success_rate, window_start, window_end
                        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                        ON CONFLICT (policy_version_id, traffic_percentage, window_start) 
                        DO UPDATE SET
                            total_requests = canary_metrics.total_requests + 1,
                            successful_requests = canary_metrics.successful_requests + 1,
                            success_rate = ((canary_metrics.successful_requests + 1)::DECIMAL / 
                                           (canary_metrics.total_requests + 1)::DECIMAL) * 100.0,
                            updated_at = CURRENT_TIMESTAMP"
                    )
                    .bind(pv_id)
                    .bind(rollout_pct)
                    .bind(1)
                    .bind(1)
                    .bind(0)
                    .bind(0)
                    .bind(100.0)
                    .bind(window_start)
                    .bind(now)
                    .execute(&db_pool)
                    .await;
                    
                    // Check if should auto-promote
                    let should_promote: Option<bool> = sqlx::query_scalar(
                        "SELECT should_promote_canary($1)"
                    )
                    .bind(pv_id)
                    .fetch_optional(&db_pool)
                    .await
                    .ok()
                    .flatten();
                    
                    if should_promote.unwrap_or(false) {
                        // Auto-promote to next percentage tier
                        let next_percentage = match rollout_pct {
                            1 => 5,
                            5 => 10,
                            10 => 25,
                            25 => 50,
                            50 => 100,
                            _ => 100,
                        };
                        
                        let success_rate: Option<f64> = sqlx::query_scalar(
                            "SELECT calculate_canary_success_rate($1, $2, 10)"
                        )
                        .bind(pv_id)
                        .bind(rollout_pct)
                        .fetch_optional(&db_pool)
                        .await
                        .ok()
                        .flatten();
                        
                        let _ = sqlx::query(
                            "UPDATE policy_versions 
                             SET rollout_percentage = $1
                             WHERE id = $2"
                        )
                        .bind(next_percentage)
                        .bind(pv_id)
                        .execute(&db_pool)
                        .await;
                        
                        // Log promotion
                        let _ = sqlx::query(
                            "INSERT INTO canary_deployment_history (
                                policy_version_id, action, from_percentage, to_percentage,
                                success_rate, triggered_by, notes
                            ) VALUES ($1, $2, $3, $4, $5, $6, $7)"
                        )
                        .bind(pv_id)
                        .bind("PROMOTED")
                        .bind(rollout_pct)
                        .bind(next_percentage)
                        .bind(success_rate.unwrap_or(0.0))
                        .bind("AUTO")
                        .bind(format!("Auto-promoted from {}% to {}% based on success rate", rollout_pct, next_percentage))
                        .execute(&db_pool)
                        .await;
                        
                        log::info!("CANARY AUTO-PROMOTED: Policy {} from {}% to {}%", pv_id, rollout_pct, next_percentage);
                    }
                }
                
                if let Err(e) = sqlx::query(
                    "INSERT INTO compliance_records (
                        id, seal_id, tx_id, agent_id, action_summary, status, 
                        risk_level, timestamp, payload_hash
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
                )
                .bind(record_id)
                .bind(&seal_id)
                .bind(&tx_id)
                .bind(&agent_id)
                .bind(&action_summary)
                .bind("COMPLIANT")
                .bind("LOW")
                .bind(chrono::Utc::now())
                .bind(&format!("region:{}", country_for_log))
                .execute(&db_pool)
                .await {
                    log::error!("Failed to log proxy success: {}", e);
                }
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

            http_response.body(body_bytes.to_vec())
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

// ========== POLICY SIMULATION & OPERATIONAL SAFETY ==========

/// Simulate policy change impact before enforcement
#[utoipa::path(
    post,
    path = "/policies/simulate",
    tag = "Policy Management",
    request_body = SimulationRequest,
    responses(
        (status = 200, description = "Simulation result", body = SimulationResult),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn simulate_policy(
    req: web::Json<crate::core::policy_simulator::SimulationRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let claims = match authenticate_and_authorize(&http_req, &data.db_pool, "policy", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    // Run simulation
    match crate::core::policy_simulator::PolicySimulator::simulate(
        &data.db_pool,
        req.into_inner(),
    )
    .await
    {
        Ok(result) => {
            // Cache the result
            let _ = sqlx::query(
                "INSERT INTO policy_simulation_results (policy_type, policy_config, simulation_result, time_range_days, simulated_by)
                 VALUES ($1, $2, $3, $4, $5)"
            )
            .bind(format!("{:?}", result.policy_type))
            .bind(serde_json::json!({}))
            .bind(serde_json::to_value(&result).unwrap_or(serde_json::Value::Null))
            .bind(result.total_requests)
            .bind(&claims.sub)
            .execute(&data.db_pool)
            .await;

            HttpResponse::Ok().json(result)
        }
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": "SIMULATION_FAILED",
            "message": e
        }))
    }
}

#[derive(Deserialize, ToSchema)]
pub struct RollbackRequest {
    #[schema(example = 3)]
    pub target_version: Option<i32>,
    #[schema(example = "Reverting due to production issues")]
    pub notes: Option<String>,
}

/// Rollback a policy to a previous version
#[utoipa::path(
    post,
    path = "/policies/{policy_id}/rollback",
    tag = "Policy Management",
    params(
        ("policy_id" = String, Path, description = "Policy version ID to rollback to")
    ),
    request_body = RollbackRequest,
    responses(
        (status = 200, description = "Policy rolled back successfully"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Policy not found"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn rollback_policy(
    path: web::Path<String>,
    req: web::Json<RollbackRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let claims = match authenticate_and_authorize(&http_req, &data.db_pool, "policy", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let policy_id = path.into_inner();
    let rollback_req = req.into_inner();

    // Get current active policy
    let current_policy: Option<(String, i32)> = sqlx::query_as(
        "SELECT policy_type, version_number FROM policy_versions WHERE id = $1 AND is_active = true"
    )
    .bind(&policy_id)
    .fetch_optional(&data.db_pool)
    .await
    .ok()
    .flatten();

    if current_policy.is_none() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "POLICY_NOT_FOUND",
            "message": "Active policy not found"
        }));
    }

    let (policy_type, _current_version) = current_policy.unwrap();

    // Get target version (or previous version if not specified)
    let target_version = if let Some(version) = rollback_req.target_version {
        version
    } else {
        // Get previous version
        sqlx::query_scalar::<_, i32>(
            "SELECT MAX(version_number) FROM policy_versions 
             WHERE policy_type = $1 AND version_number < (SELECT version_number FROM policy_versions WHERE id = $2)"
        )
        .bind(&policy_type)
        .bind(&policy_id)
        .fetch_optional(&data.db_pool)
        .await
        .ok()
        .flatten()
        .unwrap_or(1)
    };

    // Deactivate current policy
    let _ = sqlx::query(
        "UPDATE policy_versions SET is_active = false, deactivated_at = CURRENT_TIMESTAMP WHERE id = $1"
    )
    .bind(&policy_id)
    .execute(&data.db_pool)
    .await;

    // Activate target version
    let result = sqlx::query(
        "UPDATE policy_versions 
         SET is_active = true, activated_at = CURRENT_TIMESTAMP 
         WHERE policy_type = $1 AND version_number = $2
         RETURNING id"
    )
    .bind(&policy_type)
    .bind(target_version)
    .fetch_optional(&data.db_pool)
    .await;

    match result {
        Ok(Some(row)) => {
            let new_policy_id: Uuid = row.get(0);

            // Log rollback in history
            let _ = sqlx::query(
                "INSERT INTO policy_activation_history (policy_version_id, action, performed_by, previous_version_id, notes)
                 VALUES ($1, 'ROLLED_BACK', $2, $3, $4)"
            )
            .bind(new_policy_id)
            .bind(&claims.sub)
            .bind(Uuid::parse_str(&policy_id).ok())
            .bind(rollback_req.notes)
            .execute(&data.db_pool)
            .await;

            HttpResponse::Ok().json(serde_json::json!({
                "status": "SUCCESS",
                "message": format!("Policy rolled back to version {}", target_version),
                "new_policy_id": new_policy_id,
                "previous_policy_id": policy_id
            }))
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "TARGET_VERSION_NOT_FOUND",
            "message": format!("Version {} not found for policy type {}", target_version, policy_type)
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "ROLLBACK_FAILED",
            "message": format!("Database error: {}", e)
        }))
    }
}

/// Get policy impact analytics
#[derive(Serialize, ToSchema)]
pub struct PolicyImpactAnalytics {
    pub total_requests: i64,
    pub requests_by_country: HashMap<String, i64>,
    pub requests_by_agent: HashMap<String, AgentStats>,
    pub requests_by_endpoint: HashMap<String, i64>,
    pub risk_assessment: RiskAssessmentSummary,
}

#[derive(Serialize, ToSchema)]
pub struct AgentStats {
    pub total: i64,
    pub by_country: HashMap<String, i64>,
}

#[derive(Serialize, ToSchema)]
pub struct RiskAssessmentSummary {
    pub critical_agents: Vec<String>,
    pub partial_impact: Vec<String>,
}

#[utoipa::path(
    get,
    path = "/analytics/policy-impact",
    tag = "Policy Management",
    responses(
        (status = 200, description = "Policy impact analytics", body = PolicyImpactAnalytics),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_policy_impact_analytics(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "analytics", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let time_range = query.get("time_range").and_then(|s| s.parse::<i64>().ok()).unwrap_or(30);
    let start_time = Utc::now() - chrono::Duration::days(time_range);

    // Get compliance records
    #[derive(sqlx::FromRow)]
    struct Record {
        agent_id: String,
        action_summary: String,
        payload_hash: String,
    }

    let records: Vec<Record> = sqlx::query_as(
        "SELECT agent_id, action_summary, payload_hash 
         FROM compliance_records 
         WHERE timestamp >= $1"
    )
    .bind(start_time)
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let mut requests_by_country: HashMap<String, i64> = HashMap::new();
    let mut requests_by_agent: HashMap<String, AgentStats> = HashMap::new();
    let mut requests_by_endpoint: HashMap<String, i64> = HashMap::new();

    for record in &records {
        // Extract region
        let region = record.payload_hash
            .strip_prefix("region:")
            .map(|s| s.to_uppercase())
            .unwrap_or_else(|| "UNKNOWN".to_string());

        // Extract endpoint - need to make extract_endpoint public or use a helper
        let endpoint = if let Some(url_start) = record.action_summary.find("http") {
            let url_part = &record.action_summary[url_start..];
            let url_end = url_part
                .find(' ')
                .or_else(|| url_part.find('\n'))
                .unwrap_or(url_part.len());
            let url = &url_part[..url_end];
            
            if let Some(proto_end) = url.find("://") {
                let after_proto = &url[proto_end + 3..];
                if let Some(host_end) = after_proto.find('/') {
                    after_proto[..host_end].to_string()
                } else {
                    after_proto.to_string()
                }
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        };

        *requests_by_country.entry(region.clone()).or_insert(0) += 1;
        *requests_by_endpoint.entry(endpoint).or_insert(0) += 1;

        let agent_stats = requests_by_agent.entry(record.agent_id.clone()).or_insert_with(|| AgentStats {
            total: 0,
            by_country: HashMap::new(),
        });
        agent_stats.total += 1;
        *agent_stats.by_country.entry(region).or_insert(0) += 1;
    }

    // Identify critical agents (those with >50% US/CN/RU traffic)
    let mut critical_agents = Vec::new();
    let mut partial_impact = Vec::new();

    for (agent_id, stats) in &requests_by_agent {
        let blocked_countries: i64 = ["US", "CN", "RU"]
            .iter()
            .map(|country| stats.by_country.get(*country).copied().unwrap_or(0))
            .sum();

        let block_percentage = if stats.total > 0 {
            (blocked_countries as f64 / stats.total as f64) * 100.0
        } else {
            0.0
        };

        if block_percentage >= 50.0 {
            critical_agents.push(agent_id.clone());
        } else if block_percentage > 0.0 {
            partial_impact.push(agent_id.clone());
        }
    }

    HttpResponse::Ok().json(PolicyImpactAnalytics {
        total_requests: records.len() as i64,
        requests_by_country,
        requests_by_agent,
        requests_by_endpoint,
        risk_assessment: RiskAssessmentSummary {
            critical_agents,
            partial_impact,
        },
    })
}

// ========== ASSET REGISTRY & BUSINESS FUNCTION MAPPING ==========

/// Create or update an asset
#[derive(Deserialize, Serialize, ToSchema)]
pub struct AssetRequest {
    #[schema(example = "agent-credit-scoring-001")]
    pub asset_id: String,
    #[schema(example = "Credit Scoring AI Agent")]
    pub asset_name: String,
    #[schema(example = "AI_AGENT")]
    pub asset_type: String,
    #[schema(example = "CREDIT_SCORING")]
    pub business_function: String,
    #[schema(example = "RISK_MANAGEMENT")]
    pub department: Option<String>,
    #[schema(example = "john.doe@company.com")]
    pub owner: Option<String>,
    #[schema(example = "EU")]
    pub location: Option<String>,
    #[schema(example = "HIGH")]
    pub risk_profile: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
    #[schema(example = "agent-credit-scoring-001")]
    pub agent_ids: Option<Vec<String>>, // Agent IDs to map to this asset
}

#[derive(Serialize, ToSchema)]
pub struct AssetResponse {
    pub id: Uuid,
    pub asset_id: String,
    pub asset_name: String,
    pub business_function: String,
    pub created_at: DateTime<Utc>,
}

#[utoipa::path(
    post,
    path = "/assets",
    tag = "Asset Management",
    request_body = AssetRequest,
    responses(
        (status = 200, description = "Asset created or updated", body = AssetResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn create_or_update_asset(
    req: web::Json<AssetRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let claims = match authenticate_and_authorize(&http_req, &data.db_pool, "asset", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let asset_req = req.into_inner();
    let now = Utc::now();

    // Check if asset exists
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM assets WHERE asset_id = $1"
    )
    .bind(&asset_req.asset_id)
    .fetch_optional(&data.db_pool)
    .await
    .ok()
    .flatten();

    let asset_uuid = if let Some((id,)) = existing {
        // Update existing asset
        let _ = sqlx::query(
            "UPDATE assets SET
                asset_name = $1,
                asset_type = $2,
                business_function = $3,
                department = $4,
                owner = $5,
                location = $6,
                risk_profile = COALESCE($7, risk_profile),
                metadata = COALESCE($8, metadata),
                tags = COALESCE($9, tags),
                updated_at = $10,
                created_by = $11
             WHERE asset_id = $12"
        )
        .bind(&asset_req.asset_name)
        .bind(&asset_req.asset_type)
        .bind(&asset_req.business_function)
        .bind(&asset_req.department)
        .bind(&asset_req.owner)
        .bind(&asset_req.location)
        .bind(asset_req.risk_profile.as_deref())
        .bind(asset_req.metadata.as_ref().unwrap_or(&serde_json::json!({})))
        .bind(&asset_req.tags)
        .bind(now)
        .bind(&claims.sub)
        .bind(&asset_req.asset_id)
        .execute(&data.db_pool)
        .await;

        id
    } else {
        // Create new asset
        let new_id = Uuid::new_v4();
        let _ = sqlx::query(
            "INSERT INTO assets (
                id, asset_id, asset_name, asset_type, business_function,
                department, owner, location, risk_profile, metadata, tags,
                created_at, updated_at, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"
        )
        .bind(new_id)
        .bind(&asset_req.asset_id)
        .bind(&asset_req.asset_name)
        .bind(&asset_req.asset_type)
        .bind(&asset_req.business_function)
        .bind(&asset_req.department)
        .bind(&asset_req.owner)
        .bind(&asset_req.location)
        .bind(asset_req.risk_profile.as_deref().unwrap_or("MEDIUM"))
        .bind(asset_req.metadata.as_ref().unwrap_or(&serde_json::json!({})))
        .bind(&asset_req.tags)
        .bind(now)
        .bind(now)
        .bind(&claims.sub)
        .execute(&data.db_pool)
        .await;

        new_id
    };

    // Map agent IDs to asset if provided
    if let Some(agent_ids) = asset_req.agent_ids {
        for agent_id in agent_ids {
            let _ = sqlx::query(
                "INSERT INTO asset_agent_mapping (asset_id, agent_id, mapping_type)
                 VALUES ($1, $2, 'PRIMARY')
                 ON CONFLICT (asset_id, agent_id) DO NOTHING"
            )
            .bind(asset_uuid)
            .bind(&agent_id)
            .execute(&data.db_pool)
            .await;
        }
    }

    // Auto-enrich with TPRM data if vendors are specified in metadata
    if let Some(metadata) = &asset_req.metadata {
        if let Some(vendors) = metadata.get("vendors").and_then(|v| v.as_array()) {
            let api_key = std::env::var("VERIDION_TPRM_API_KEY").ok();
            
            // Store vendor mappings
            for vendor in vendors {
                if let Some(vendor_domain) = vendor.as_str() {
                    // Store vendor mapping
                    let _ = sqlx::query(
                        "INSERT INTO asset_vendor_mapping (asset_id, vendor_domain)
                         VALUES ($1, $2)
                         ON CONFLICT (asset_id, vendor_domain) DO NOTHING"
                    )
                    .bind(asset_uuid)
                    .bind(vendor_domain)
                    .execute(&data.db_pool)
                    .await;

                    // Fetch and store risk score (async, don't block response)
                    let api_key_clone = api_key.clone();
                    let db_pool_clone = data.db_pool.clone();
                    let vendor_domain_clone = vendor_domain.to_string();
                    tokio::spawn(async move {
                        let tprm_service = crate::integration::veridion_tprm::VeridionTPRMService::new(api_key_clone);
                        if let Ok(risk_score) = tprm_service.fetch_vendor_risk_score(&vendor_domain_clone).await {
                            let _ = tprm_service.store_vendor_risk_score(&db_pool_clone, &risk_score).await;
                        }
                    });
                }
            }
        }
    }

    HttpResponse::Ok().json(AssetResponse {
        id: asset_uuid,
        asset_id: asset_req.asset_id,
        asset_name: asset_req.asset_name,
        business_function: asset_req.business_function,
        created_at: now,
    })
}

/// Get all assets
#[derive(Serialize, ToSchema)]
pub struct AssetsListResponse {
    pub assets: Vec<AssetDetailResponse>,
}

#[derive(Serialize, ToSchema)]
pub struct AssetDetailResponse {
    pub id: Uuid,
    pub asset_id: String,
    pub asset_name: String,
    pub asset_type: String,
    pub business_function: String,
    pub department: Option<String>,
    pub owner: Option<String>,
    pub location: Option<String>,
    pub risk_profile: String,
    pub tags: Option<Vec<String>>,
    pub agent_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[utoipa::path(
    get,
    path = "/assets",
    tag = "Asset Management",
    params(
        ("business_function" = Option<String>, Query, description = "Filter by business function"),
        ("department" = Option<String>, Query, description = "Filter by department"),
        ("risk_profile" = Option<String>, Query, description = "Filter by risk profile"),
    ),
    responses(
        (status = 200, description = "List of assets", body = AssetsListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn list_assets(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "asset", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let business_function = query.get("business_function");
    let department = query.get("department");
    let risk_profile = query.get("risk_profile");

    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT a.* FROM assets a WHERE a.is_active = true"
    );

    if let Some(bf) = business_function {
        query_builder.push(" AND a.business_function = ");
        query_builder.push_bind(bf);
    }
    if let Some(dept) = department {
        query_builder.push(" AND a.department = ");
        query_builder.push_bind(dept);
    }
    if let Some(rp) = risk_profile {
        query_builder.push(" AND a.risk_profile = ");
        query_builder.push_bind(rp);
    }

    query_builder.push(" ORDER BY a.created_at DESC");

    let assets: Vec<AssetDb> = query_builder
        .build_query_as()
        .fetch_all(&data.db_pool)
        .await
        .unwrap_or_default();

    // Get agent mappings for each asset
    let mut result = Vec::new();
    for asset in assets {
        let agent_ids: Vec<String> = sqlx::query_scalar(
            "SELECT agent_id FROM asset_agent_mapping WHERE asset_id = $1"
        )
        .bind(asset.id)
        .fetch_all(&data.db_pool)
        .await
        .unwrap_or_default();

        result.push(AssetDetailResponse {
            id: asset.id,
            asset_id: asset.asset_id,
            asset_name: asset.asset_name,
            asset_type: asset.asset_type,
            business_function: asset.business_function,
            department: asset.department,
            owner: asset.owner,
            location: asset.location,
            risk_profile: asset.risk_profile,
            tags: asset.tags,
            agent_ids,
            created_at: asset.created_at,
        });
    }

    HttpResponse::Ok().json(AssetsListResponse { assets: result })
}

/// Get asset by agent_id
#[utoipa::path(
    get,
    path = "/assets/by-agent/{agent_id}",
    tag = "Asset Management",
    params(
        ("agent_id" = String, Path, description = "Agent ID to lookup")
    ),
    responses(
        (status = 200, description = "Asset found", body = AssetDetailResponse),
        (status = 404, description = "Asset not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_asset_by_agent(
    path: web::Path<String>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "asset", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let agent_id = path.into_inner();

    // Find asset by agent mapping
    let asset: Option<AssetDb> = sqlx::query_as(
        "SELECT a.* FROM assets a
         JOIN asset_agent_mapping aam ON a.id = aam.asset_id
         WHERE aam.agent_id = $1 AND a.is_active = true
         LIMIT 1"
    )
    .bind(&agent_id)
    .fetch_optional(&data.db_pool)
    .await
    .ok()
    .flatten();

    match asset {
        Some(asset) => {
            let agent_ids: Vec<String> = sqlx::query_scalar(
                "SELECT agent_id FROM asset_agent_mapping WHERE asset_id = $1"
            )
            .bind(asset.id)
            .fetch_all(&data.db_pool)
            .await
            .unwrap_or_default();

            HttpResponse::Ok().json(AssetDetailResponse {
                id: asset.id,
                asset_id: asset.asset_id,
                asset_name: asset.asset_name,
                asset_type: asset.asset_type,
                business_function: asset.business_function,
                department: asset.department,
                owner: asset.owner,
                location: asset.location,
                risk_profile: asset.risk_profile,
                tags: asset.tags,
                agent_ids,
                created_at: asset.created_at,
            })
        }
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": "ASSET_NOT_FOUND",
            "message": format!("No asset found for agent_id: {}", agent_id)
        }))
    }
}

/// Get business functions
#[utoipa::path(
    get,
    path = "/business-functions",
    tag = "Asset Management",
    responses(
        (status = 200, description = "List of business functions"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn list_business_functions(
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "asset", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let functions: Vec<BusinessFunctionDb> = sqlx::query_as(
        "SELECT * FROM business_functions ORDER BY function_name"
    )
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    HttpResponse::Ok().json(functions)
}

/// Create asset-based policy
#[derive(Deserialize, Serialize, ToSchema)]
pub struct AssetPolicyRequest {
    #[schema(example = "Credit Scoring - EU Only")]
    pub policy_name: String,
    #[schema(example = "SOVEREIGN_LOCK")]
    pub policy_type: String,
    #[schema(example = "CREDIT_SCORING")]
    pub business_function_filter: Option<String>,
    #[schema(example = "RISK_MANAGEMENT")]
    pub department_filter: Option<String>,
    #[schema(example = "EU")]
    pub location_filter: Option<String>,
    #[schema(example = "HIGH")]
    pub risk_profile_filter: Option<String>,
    pub asset_tags_filter: Option<Vec<String>>,
    pub policy_config: serde_json::Value,
    #[schema(example = 100)]
    pub priority: Option<i32>,
}

#[derive(Serialize, ToSchema)]
pub struct AssetPolicyResponse {
    pub id: Uuid,
    pub policy_name: String,
    pub created_at: DateTime<Utc>,
}

#[utoipa::path(
    post,
    path = "/asset-policies",
    tag = "Asset Management",
    request_body = AssetPolicyRequest,
    responses(
        (status = 200, description = "Asset policy created", body = AssetPolicyResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn create_asset_policy(
    req: web::Json<AssetPolicyRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let claims = match authenticate_and_authorize(&http_req, &data.db_pool, "asset", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let policy_req = req.into_inner();
    let now = Utc::now();
    let policy_id = Uuid::new_v4();

    let result = sqlx::query(
        "INSERT INTO asset_policies (
            id, policy_name, policy_type, business_function_filter,
            department_filter, location_filter, risk_profile_filter,
            asset_tags_filter, policy_config, priority, created_at, updated_at, created_by
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING id"
    )
    .bind(policy_id)
    .bind(&policy_req.policy_name)
    .bind(&policy_req.policy_type)
    .bind(&policy_req.business_function_filter)
    .bind(&policy_req.department_filter)
    .bind(&policy_req.location_filter)
    .bind(&policy_req.risk_profile_filter)
    .bind(&policy_req.asset_tags_filter)
    .bind(&policy_req.policy_config)
    .bind(policy_req.priority.unwrap_or(100))
    .bind(now)
    .bind(now)
    .bind(&claims.sub)
    .fetch_one(&data.db_pool)
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(AssetPolicyResponse {
            id: policy_id,
            policy_name: policy_req.policy_name,
            created_at: now,
        }),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": "POLICY_CREATION_FAILED",
            "message": format!("Failed to create policy: {}", e)
        }))
    }
}

/// List asset-based policies
#[utoipa::path(
    get,
    path = "/asset-policies",
    tag = "Asset Management",
    responses(
        (status = 200, description = "List of asset policies"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn list_asset_policies(
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "asset", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let policies: Vec<AssetPolicyDb> = sqlx::query_as(
        "SELECT * FROM asset_policies ORDER BY priority ASC, created_at DESC"
    )
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    HttpResponse::Ok().json(policies)
}

/// Get current system enforcement mode
#[derive(Serialize, ToSchema)]
pub struct EnforcementModeResponse {
    pub enforcement_mode: String,
    pub enabled_at: DateTime<Utc>,
    pub enabled_by: Option<String>,
    pub description: Option<String>,
}

#[utoipa::path(
    get,
    path = "/system/enforcement-mode",
    tag = "System Configuration",
    responses(
        (status = 200, description = "Current enforcement mode", body = EnforcementModeResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_enforcement_mode(
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "system", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    #[derive(sqlx::FromRow)]
    struct EnforcementModeDb {
        enforcement_mode: String,
        enabled_at: DateTime<Utc>,
        enabled_by: Option<String>,
        description: Option<String>,
    }

    let result: Option<EnforcementModeDb> = sqlx::query_as(
        "SELECT enforcement_mode, enabled_at, enabled_by, description 
         FROM system_enforcement_mode 
         ORDER BY enabled_at DESC 
         LIMIT 1"
    )
    .fetch_optional(&data.db_pool)
    .await
    .ok()
    .flatten();

    match result {
        Some(mode) => HttpResponse::Ok().json(EnforcementModeResponse {
            enforcement_mode: mode.enforcement_mode,
            enabled_at: mode.enabled_at,
            enabled_by: mode.enabled_by,
            description: mode.description,
        }),
        None => HttpResponse::Ok().json(EnforcementModeResponse {
            enforcement_mode: "ENFORCING".to_string(),
            enabled_at: Utc::now(),
            enabled_by: Some("system".to_string()),
            description: Some("Default enforcement mode".to_string()),
        })
    }
}

/// Set system enforcement mode
#[derive(Deserialize, Serialize, ToSchema)]
pub struct SetEnforcementModeRequest {
    #[schema(example = "SHADOW")]
    pub enforcement_mode: String,
    #[schema(example = "Testing new policies in shadow mode")]
    pub description: Option<String>,
    #[schema(example = "Testing new policies")]
    pub notes: Option<String>,
}

#[utoipa::path(
    post,
    path = "/system/enforcement-mode",
    tag = "System Configuration",
    request_body = SetEnforcementModeRequest,
    responses(
        (status = 200, description = "Enforcement mode updated", body = EnforcementModeResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn set_enforcement_mode(
    req: web::Json<SetEnforcementModeRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let claims = match authenticate_and_authorize(&http_req, &data.db_pool, "system", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let mode_req = req.into_inner();
    
    // Validate enforcement mode
    let valid_modes = ["SHADOW", "DRY_RUN", "ENFORCING"];
    if !valid_modes.contains(&mode_req.enforcement_mode.as_str()) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "INVALID_ENFORCEMENT_MODE",
            "message": format!("Enforcement mode must be one of: {:?}", valid_modes)
        }));
    }

    let now = Utc::now();
    let result = sqlx::query(
        "INSERT INTO system_enforcement_mode (enforcement_mode, description, enabled_by, notes)
         VALUES ($1, $2, $3, $4)
         RETURNING enforcement_mode, enabled_at, enabled_by, description"
    )
    .bind(&mode_req.enforcement_mode)
    .bind(&mode_req.description)
    .bind(&claims.sub)
    .bind(&mode_req.notes)
    .fetch_one(&data.db_pool)
    .await;

    match result {
        Ok(row) => {
            let mode: String = row.get(0);
            let enabled_at: DateTime<Utc> = row.get(1);
            let enabled_by: Option<String> = row.get(2);
            let description: Option<String> = row.get(3);
            
            log::info!("Enforcement mode changed to {} by {}", mode, claims.sub);
            
            HttpResponse::Ok().json(EnforcementModeResponse {
                enforcement_mode: mode,
                enabled_at,
                enabled_by,
                description,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "FAILED_TO_SET_MODE",
            "message": format!("Failed to set enforcement mode: {}", e)
        }))
    }
}

// ========== SHADOW MODE ANALYTICS ==========

#[derive(Serialize, ToSchema)]
pub struct ShadowModeAnalytics {
    pub total_logs: i64,
    pub would_block_count: i64,
    pub would_allow_count: i64,
    pub block_percentage: f64,
    pub top_blocked_agents: Vec<AgentShadowStats>,
    pub top_blocked_regions: Vec<RegionShadowStats>,
    pub top_policies_applied: Vec<PolicyShadowStats>,
    pub time_range: TimeRange,
    pub confidence_score: f64, // Confidence that shadow mode data is representative
}

#[derive(Serialize, ToSchema)]
pub struct AgentShadowStats {
    pub agent_id: String,
    pub would_block: i64,
    pub would_allow: i64,
    pub total: i64,
    pub block_percentage: f64,
}

#[derive(Serialize, ToSchema)]
pub struct RegionShadowStats {
    pub region: String,
    pub would_block: i64,
    pub would_allow: i64,
    pub total: i64,
    pub block_percentage: f64,
}

#[derive(Serialize, ToSchema)]
pub struct PolicyShadowStats {
    pub policy_name: String,
    pub would_block: i64,
    pub would_allow: i64,
    pub total: i64,
    pub block_percentage: f64,
}

#[derive(Serialize, ToSchema)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub days: i64,
}

/// Get shadow mode analytics
#[utoipa::path(
    get,
    path = "/analytics/shadow-mode",
    tag = "Shadow Mode",
    params(
        ("days" = Option<i64>, Query, description = "Number of days to analyze (default: 7)"),
        ("agent_id" = Option<String>, Query, description = "Filter by agent ID"),
    ),
    responses(
        (status = 200, description = "Shadow mode analytics", body = ShadowModeAnalytics),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_shadow_mode_analytics(
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "analytics", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let days = query.get("days")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(7);
    let agent_filter = query.get("agent_id");

    let end_time = Utc::now();
    let start_time = end_time - chrono::Duration::days(days);

    // Get total counts
    let total_logs: i64 = if let Some(agent_id) = agent_filter {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM shadow_mode_logs 
             WHERE timestamp >= $1 AND timestamp <= $2 AND agent_id = $3"
        )
        .bind(start_time)
        .bind(end_time)
        .bind(agent_id)
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or(0)
    } else {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM shadow_mode_logs 
             WHERE timestamp >= $1 AND timestamp <= $2"
        )
        .bind(start_time)
        .bind(end_time)
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or(0)
    };

    let would_block_count: i64 = if let Some(agent_id) = agent_filter {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM shadow_mode_logs 
             WHERE timestamp >= $1 AND timestamp <= $2 AND would_block = true AND agent_id = $3"
        )
        .bind(start_time)
        .bind(end_time)
        .bind(agent_id)
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or(0)
    } else {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM shadow_mode_logs 
             WHERE timestamp >= $1 AND timestamp <= $2 AND would_block = true"
        )
        .bind(start_time)
        .bind(end_time)
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or(0)
    };

    let would_allow_count = total_logs - would_block_count;
    let block_percentage = if total_logs > 0 {
        (would_block_count as f64 / total_logs as f64) * 100.0
    } else {
        0.0
    };

    // Get top blocked agents
    #[derive(sqlx::FromRow)]
    struct AgentStatsRow {
        agent_id: String,
        would_block: i64,
        would_allow: i64,
        total: i64,
    }

    let top_agents: Vec<AgentStatsRow> = if let Some(agent_id) = agent_filter {
        sqlx::query_as::<_, AgentStatsRow>(
            "SELECT 
                agent_id,
                COUNT(*) FILTER (WHERE would_block = true) as would_block,
                COUNT(*) FILTER (WHERE would_allow = true) as would_allow,
                COUNT(*) as total
             FROM shadow_mode_logs
             WHERE timestamp >= $1 AND timestamp <= $2 AND agent_id = $3
             GROUP BY agent_id
             ORDER BY would_block DESC
             LIMIT 10"
        )
        .bind(start_time)
        .bind(end_time)
        .bind(agent_id)
        .fetch_all(&data.db_pool)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as::<_, AgentStatsRow>(
            "SELECT 
                agent_id,
                COUNT(*) FILTER (WHERE would_block = true) as would_block,
                COUNT(*) FILTER (WHERE would_allow = true) as would_allow,
                COUNT(*) as total
             FROM shadow_mode_logs
             WHERE timestamp >= $1 AND timestamp <= $2
             GROUP BY agent_id
             ORDER BY would_block DESC
             LIMIT 10"
        )
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&data.db_pool)
        .await
        .unwrap_or_default()
    };

    let top_blocked_agents: Vec<AgentShadowStats> = top_agents.into_iter().map(|row| {
        let block_pct = if row.total > 0 {
            (row.would_block as f64 / row.total as f64) * 100.0
        } else {
            0.0
        };
        AgentShadowStats {
            agent_id: row.agent_id,
            would_block: row.would_block,
            would_allow: row.would_allow,
            total: row.total,
            block_percentage: block_pct,
        }
    }).collect();

    // Get top blocked regions
    #[derive(sqlx::FromRow)]
    struct RegionStatsRow {
        detected_country: Option<String>,
        would_block: i64,
        would_allow: i64,
        total: i64,
    }

    let top_regions: Vec<RegionStatsRow> = sqlx::query_as::<_, RegionStatsRow>(
        "SELECT 
            COALESCE(detected_country, 'UNKNOWN') as detected_country,
            COUNT(*) FILTER (WHERE would_block = true) as would_block,
            COUNT(*) FILTER (WHERE would_allow = true) as would_allow,
            COUNT(*) as total
         FROM shadow_mode_logs
         WHERE timestamp >= $1 AND timestamp <= $2
         GROUP BY detected_country
         ORDER BY would_block DESC
         LIMIT 10"
    )
    .bind(start_time)
    .bind(end_time)
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let top_blocked_regions: Vec<RegionShadowStats> = top_regions.into_iter().map(|row| {
        let block_pct = if row.total > 0 {
            (row.would_block as f64 / row.total as f64) * 100.0
        } else {
            0.0
        };
        RegionShadowStats {
            region: row.detected_country.unwrap_or_else(|| "UNKNOWN".to_string()),
            would_block: row.would_block,
            would_allow: row.would_allow,
            total: row.total,
            block_percentage: block_pct,
        }
    }).collect();

    // Get top policies applied
    #[derive(sqlx::FromRow)]
    struct PolicyStatsRow {
        policy_applied: Option<String>,
        would_block: i64,
        would_allow: i64,
        total: i64,
    }

    let top_policies: Vec<PolicyStatsRow> = sqlx::query_as::<_, PolicyStatsRow>(
        "SELECT 
            COALESCE(policy_applied, 'UNKNOWN') as policy_applied,
            COUNT(*) FILTER (WHERE would_block = true) as would_block,
            COUNT(*) FILTER (WHERE would_allow = true) as would_allow,
            COUNT(*) as total
         FROM shadow_mode_logs
         WHERE timestamp >= $1 AND timestamp <= $2
         GROUP BY policy_applied
         ORDER BY would_block DESC
         LIMIT 10"
    )
    .bind(start_time)
    .bind(end_time)
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let top_policies_applied: Vec<PolicyShadowStats> = top_policies.into_iter().map(|row| {
        let block_pct = if row.total > 0 {
            (row.would_block as f64 / row.total as f64) * 100.0
        } else {
            0.0
        };
        PolicyShadowStats {
            policy_name: row.policy_applied.unwrap_or_else(|| "UNKNOWN".to_string()),
            would_block: row.would_block,
            would_allow: row.would_allow,
            total: row.total,
            block_percentage: block_pct,
        }
    }).collect();

    // Calculate confidence score (based on sample size and time range)
    let confidence_score = if total_logs >= 1000 {
        95.0 // High confidence with large sample
    } else if total_logs >= 100 {
        85.0 // Medium-high confidence
    } else if total_logs >= 10 {
        70.0 // Medium confidence
    } else {
        50.0 // Low confidence - need more data
    };

    HttpResponse::Ok().json(ShadowModeAnalytics {
        total_logs,
        would_block_count,
        would_allow_count,
        block_percentage,
        top_blocked_agents,
        top_blocked_regions,
        top_policies_applied,
        time_range: TimeRange {
            start: start_time,
            end: end_time,
            days,
        },
        confidence_score,
    })
}

/// Configure circuit breaker for a policy
#[derive(Deserialize, Serialize, ToSchema)]
pub struct CircuitBreakerConfigRequest {
    #[schema(example = true)]
    pub enabled: bool,
    #[schema(example = 10.0)]
    pub error_threshold: Option<f64>, // Percentage: 0.0-100.0
    #[schema(example = 5)]
    pub window_minutes: Option<i32>,
    #[schema(example = 15)]
    pub cooldown_minutes: Option<i32>,
}

#[derive(Serialize, ToSchema)]
pub struct CircuitBreakerConfigResponse {
    pub policy_id: Uuid,
    pub enabled: bool,
    pub error_threshold: f64,
    pub window_minutes: i32,
    pub cooldown_minutes: i32,
    pub current_state: String,
}

#[utoipa::path(
    post,
    path = "/policies/{policy_id}/circuit-breaker/config",
    tag = "Policy Management",
    params(
        ("policy_id" = Uuid, Path, description = "Policy ID"),
    ),
    request_body = CircuitBreakerConfigRequest,
    responses(
        (status = 200, description = "Circuit breaker configured"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn configure_circuit_breaker(
    policy_id: web::Path<Uuid>,
    req: web::Json<CircuitBreakerConfigRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "policy", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let policy_id = policy_id.into_inner();
    let config = req.into_inner();
    
    // Validate threshold
    if let Some(threshold) = config.error_threshold {
        if threshold < 0.0 || threshold > 100.0 {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "INVALID_THRESHOLD",
                "message": "Error threshold must be between 0.0 and 100.0"
            }));
        }
    }
    
    let error_threshold = config.error_threshold.unwrap_or(10.0);
    let window_minutes = config.window_minutes.unwrap_or(5);
    let cooldown_minutes = config.cooldown_minutes.unwrap_or(15);
    
    let result = sqlx::query(
        "UPDATE policy_versions 
         SET circuit_breaker_enabled = $1,
             circuit_breaker_error_threshold = $2,
             circuit_breaker_window_minutes = $3,
             circuit_breaker_cooldown_minutes = $4,
             circuit_breaker_state = CASE 
                 WHEN $1 = false THEN 'CLOSED'
                 ELSE circuit_breaker_state
             END
         WHERE id = $5
         RETURNING circuit_breaker_state"
    )
    .bind(config.enabled)
    .bind(error_threshold)
    .bind(window_minutes)
    .bind(cooldown_minutes)
    .bind(policy_id)
    .fetch_optional(&data.db_pool)
    .await;
    
    match result {
        Ok(Some(row)) => {
            let current_state: String = row.get(0);
            HttpResponse::Ok().json(CircuitBreakerConfigResponse {
                policy_id,
                enabled: config.enabled,
                error_threshold,
                window_minutes,
                cooldown_minutes,
                current_state,
            })
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "POLICY_NOT_FOUND",
            "message": "Policy not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "FAILED_TO_CONFIGURE",
            "message": format!("Failed to configure circuit breaker: {}", e)
        }))
    }
}

// ========== CIRCUIT BREAKER ANALYTICS ==========

#[derive(Serialize, ToSchema)]
pub struct CircuitBreakerAnalytics {
    pub total_policies: i64,
    pub open_circuits: i64,
    pub closed_circuits: i64,
    pub half_open_circuits: i64,
    pub recent_transitions: Vec<CircuitBreakerTransition>,
    pub policies: Vec<PolicyCircuitBreakerStatus>,
}

#[derive(Serialize, ToSchema)]
pub struct CircuitBreakerTransition {
    pub policy_id: String,
    pub policy_type: String,
    pub state_transition: String,
    pub error_rate: f64,
    pub error_count: i64,
    pub total_requests: i64,
    pub triggered_by: String,
    pub notes: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct PolicyCircuitBreakerStatus {
    pub policy_id: String,
    pub policy_type: String,
    pub enabled: bool,
    pub current_state: String,
    pub error_threshold: f64,
    pub current_error_rate: f64,
    pub error_count: i64,
    pub total_requests: i64,
    pub opened_at: Option<DateTime<Utc>>,
    pub last_error_at: Option<DateTime<Utc>>,
    pub cooldown_minutes: i32,
}

/// Get circuit breaker analytics
#[utoipa::path(
    get,
    path = "/analytics/circuit-breaker",
    tag = "Circuit Breaker",
    responses(
        (status = 200, description = "Circuit breaker analytics", body = CircuitBreakerAnalytics),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_circuit_breaker_analytics(
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "analytics", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    // Get all policies with circuit breaker status
    #[derive(sqlx::FromRow)]
    struct PolicyStatusRow {
        id: uuid::Uuid,
        policy_type: String,
        circuit_breaker_enabled: bool,
        circuit_breaker_state: Option<String>,
        circuit_breaker_error_threshold: Option<f64>,
        circuit_breaker_opened_at: Option<DateTime<Utc>>,
        circuit_breaker_last_error_at: Option<DateTime<Utc>>,
        circuit_breaker_cooldown_minutes: Option<i32>,
    }

    let policies: Vec<PolicyStatusRow> = sqlx::query_as::<_, PolicyStatusRow>(
        "SELECT 
            id, policy_type,
            COALESCE(circuit_breaker_enabled, false) as circuit_breaker_enabled,
            circuit_breaker_state,
            circuit_breaker_error_threshold,
            circuit_breaker_opened_at,
            circuit_breaker_last_error_at,
            circuit_breaker_cooldown_minutes
         FROM policy_versions
         WHERE is_active = true AND circuit_breaker_enabled = true
         ORDER BY circuit_breaker_opened_at DESC NULLS LAST"
    )
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let total_policies = policies.len() as i64;
    let open_circuits = policies.iter().filter(|p| p.circuit_breaker_state.as_deref() == Some("OPEN")).count() as i64;
    let closed_circuits = policies.iter().filter(|p| p.circuit_breaker_state.as_deref() == Some("CLOSED") || p.circuit_breaker_state.is_none()).count() as i64;
    let half_open_circuits = policies.iter().filter(|p| p.circuit_breaker_state.as_deref() == Some("HALF_OPEN")).count() as i64;

    // Get recent transitions
    #[derive(sqlx::FromRow)]
    struct TransitionRow {
        policy_version_id: uuid::Uuid,
        state_transition: String,
        error_rate: f64,
        error_count: i64,
        total_requests: i64,
        triggered_by: String,
        notes: Option<String>,
        timestamp: DateTime<Utc>,
    }

    let transitions: Vec<TransitionRow> = sqlx::query_as::<_, TransitionRow>(
        "SELECT 
            cbh.policy_version_id,
            cbh.state_transition,
            cbh.error_rate,
            cbh.error_count,
            cbh.total_requests,
            cbh.triggered_by,
            cbh.notes,
            cbh.timestamp
         FROM circuit_breaker_history cbh
         JOIN policy_versions pv ON pv.id = cbh.policy_version_id
         WHERE pv.is_active = true
         ORDER BY cbh.timestamp DESC
         LIMIT 50"
    )
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let mut recent_transitions: Vec<CircuitBreakerTransition> = Vec::new();
    for row in transitions {
        let policy_type: String = sqlx::query_scalar(
            "SELECT policy_type FROM policy_versions WHERE id = $1"
        )
        .bind(row.policy_version_id)
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or_else(|_| "UNKNOWN".to_string());

        recent_transitions.push(CircuitBreakerTransition {
            policy_id: row.policy_version_id.to_string(),
            policy_type,
            state_transition: row.state_transition,
            error_rate: row.error_rate,
            error_count: row.error_count,
            total_requests: row.total_requests,
            triggered_by: row.triggered_by,
            notes: row.notes,
            timestamp: row.timestamp,
        });
    }

    // Get current error rates for each policy
    let mut policies_status: Vec<PolicyCircuitBreakerStatus> = Vec::new();
    for p in policies {
        // Get current error rate from error tracking
        let error_tracking: Option<(i64, i64, f64)> = sqlx::query_as::<_, (i64, i64, f64)>(
            "SELECT 
                error_count, total_requests, error_rate
             FROM policy_error_tracking
             WHERE policy_version_id = $1
             ORDER BY window_end DESC
             LIMIT 1"
        )
        .bind(p.id)
        .fetch_optional(&data.db_pool)
        .await
        .ok()
        .flatten();

        let (error_count, total_requests, current_error_rate) = error_tracking.unwrap_or((0, 0, 0.0));

        policies_status.push(PolicyCircuitBreakerStatus {
            policy_id: p.id.to_string(),
            policy_type: p.policy_type,
            enabled: p.circuit_breaker_enabled,
            current_state: p.circuit_breaker_state.unwrap_or_else(|| "CLOSED".to_string()),
            error_threshold: p.circuit_breaker_error_threshold.unwrap_or(10.0),
            current_error_rate,
            error_count,
            total_requests,
            opened_at: p.circuit_breaker_opened_at,
            last_error_at: p.circuit_breaker_last_error_at,
            cooldown_minutes: p.circuit_breaker_cooldown_minutes.unwrap_or(15),
        });
    }

    HttpResponse::Ok().json(CircuitBreakerAnalytics {
        total_policies,
        open_circuits,
        closed_circuits,
        half_open_circuits,
        recent_transitions,
        policies: policies_status,
    })
}

// ========== CANARY DEPLOYMENT ANALYTICS ==========

#[derive(Serialize, ToSchema)]
pub struct CanaryAnalytics {
    pub total_policies: i64,
    pub active_canaries: i64,
    pub policies: Vec<PolicyCanaryStatus>,
    pub recent_promotions: Vec<CanaryTransition>,
    pub recent_rollbacks: Vec<CanaryTransition>,
}

#[derive(Serialize, ToSchema)]
pub struct PolicyCanaryStatus {
    pub policy_id: String,
    pub policy_type: String,
    pub current_traffic_percentage: i32,
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
    pub blocked_requests: i64,
    pub success_rate: f64,
    pub auto_promote_enabled: bool,
    pub auto_rollback_enabled: bool,
    pub promotion_threshold: f64,
    pub rollback_threshold: f64,
    pub min_requests_for_promotion: i64,
    pub evaluation_window_minutes: i32,
    pub last_evaluated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct CanaryTransition {
    pub policy_id: String,
    pub policy_type: String,
    pub from_percentage: i32,
    pub to_percentage: i32,
    pub reason: String,
    pub success_rate: f64,
    pub total_requests: i64,
    pub timestamp: DateTime<Utc>,
}

/// Get canary deployment analytics
#[utoipa::path(
    get,
    path = "/analytics/canary",
    tag = "Canary Deployment",
    responses(
        (status = 200, description = "Canary deployment analytics", body = CanaryAnalytics),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_canary_analytics(
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "analytics", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    // Get all policies with canary deployment
    #[derive(sqlx::FromRow)]
    struct PolicyCanaryRow {
        id: uuid::Uuid,
        policy_type: String,
        rollout_percentage: Option<i32>,
        auto_promote_enabled: Option<bool>,
        auto_rollback_enabled: Option<bool>,
        promotion_threshold: Option<f64>,
        rollback_threshold: Option<f64>,
        min_requests_for_promotion: Option<i64>,
        evaluation_window_minutes: Option<i32>,
    }

    let policies: Vec<PolicyCanaryRow> = sqlx::query_as::<_, PolicyCanaryRow>(
        "SELECT 
            id, policy_type,
            rollout_percentage,
            auto_promote_enabled,
            auto_rollback_enabled,
            promotion_threshold,
            rollback_threshold,
            min_requests_for_promotion,
            evaluation_window_minutes
         FROM policy_versions
         WHERE is_active = true AND rollout_percentage IS NOT NULL AND rollout_percentage < 100
         ORDER BY rollout_percentage DESC"
    )
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let total_policies = policies.len() as i64;
    let active_canaries = policies.iter().filter(|p| p.rollout_percentage.unwrap_or(100) < 100).count() as i64;

    // Get canary metrics for each policy
    let mut policies_status: Vec<PolicyCanaryStatus> = Vec::new();
    for p in policies {
        let traffic_pct = p.rollout_percentage.unwrap_or(100);
        
        // Get latest canary metrics
        #[derive(sqlx::FromRow)]
        struct CanaryMetricsRow {
            total_requests: i64,
            successful_requests: i64,
            failed_requests: i64,
            blocked_requests: i64,
            success_rate: f64,
            window_end: DateTime<Utc>,
        }

        let metrics: Option<CanaryMetricsRow> = sqlx::query_as::<_, CanaryMetricsRow>(
            "SELECT 
                total_requests, successful_requests, failed_requests, blocked_requests,
                success_rate, window_end
             FROM canary_metrics
             WHERE policy_version_id = $1 AND traffic_percentage = $2
             ORDER BY window_end DESC
             LIMIT 1"
        )
        .bind(p.id)
        .bind(traffic_pct)
        .fetch_optional(&data.db_pool)
        .await
        .ok()
        .flatten();

        let (total_requests, successful_requests, failed_requests, blocked_requests, success_rate) = 
            if let Some(m) = metrics {
                (m.total_requests, m.successful_requests, m.failed_requests, m.blocked_requests, m.success_rate)
            } else {
                (0, 0, 0, 0, 100.0)
            };

        policies_status.push(PolicyCanaryStatus {
            policy_id: p.id.to_string(),
            policy_type: p.policy_type,
            current_traffic_percentage: traffic_pct,
            total_requests,
            successful_requests,
            failed_requests,
            blocked_requests,
            success_rate,
            auto_promote_enabled: p.auto_promote_enabled.unwrap_or(true),
            auto_rollback_enabled: p.auto_rollback_enabled.unwrap_or(true),
            promotion_threshold: p.promotion_threshold.unwrap_or(95.0),
            rollback_threshold: p.rollback_threshold.unwrap_or(90.0),
            min_requests_for_promotion: p.min_requests_for_promotion.unwrap_or(100),
            evaluation_window_minutes: p.evaluation_window_minutes.unwrap_or(10),
            last_evaluated_at: None, // TODO: Add last_evaluated_at to policy_versions
        });
    }

    // Get recent promotions and rollbacks from canary_history
    #[derive(sqlx::FromRow)]
    struct CanaryHistoryRow {
        policy_version_id: uuid::Uuid,
        from_percentage: i32,
        to_percentage: i32,
        reason: String,
        success_rate: f64,
        total_requests: i64,
        timestamp: DateTime<Utc>,
    }

    let promotions: Vec<CanaryHistoryRow> = sqlx::query_as::<_, CanaryHistoryRow>(
        "SELECT 
            ch.policy_version_id, ch.from_percentage, ch.to_percentage,
            ch.reason, ch.success_rate, ch.total_requests, ch.timestamp
         FROM canary_history ch
         JOIN policy_versions pv ON pv.id = ch.policy_version_id
         WHERE pv.is_active = true AND ch.to_percentage > ch.from_percentage
         ORDER BY ch.timestamp DESC
         LIMIT 20"
    )
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let rollbacks: Vec<CanaryHistoryRow> = sqlx::query_as::<_, CanaryHistoryRow>(
        "SELECT 
            ch.policy_version_id, ch.from_percentage, ch.to_percentage,
            ch.reason, ch.success_rate, ch.total_requests, ch.timestamp
         FROM canary_history ch
         JOIN policy_versions pv ON pv.id = ch.policy_version_id
         WHERE pv.is_active = true AND ch.to_percentage < ch.from_percentage
         ORDER BY ch.timestamp DESC
         LIMIT 20"
    )
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let mut recent_promotions: Vec<CanaryTransition> = Vec::new();
    for row in promotions {
        let policy_type: String = sqlx::query_scalar(
            "SELECT policy_type FROM policy_versions WHERE id = $1"
        )
        .bind(row.policy_version_id)
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or_else(|_| "UNKNOWN".to_string());

        recent_promotions.push(CanaryTransition {
            policy_id: row.policy_version_id.to_string(),
            policy_type,
            from_percentage: row.from_percentage,
            to_percentage: row.to_percentage,
            reason: row.reason,
            success_rate: row.success_rate,
            total_requests: row.total_requests,
            timestamp: row.timestamp,
        });
    }

    let mut recent_rollbacks: Vec<CanaryTransition> = Vec::new();
    for row in rollbacks {
        let policy_type: String = sqlx::query_scalar(
            "SELECT policy_type FROM policy_versions WHERE id = $1"
        )
        .bind(row.policy_version_id)
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or_else(|_| "UNKNOWN".to_string());

        recent_rollbacks.push(CanaryTransition {
            policy_id: row.policy_version_id.to_string(),
            policy_type,
            from_percentage: row.from_percentage,
            to_percentage: row.to_percentage,
            reason: row.reason,
            success_rate: row.success_rate,
            total_requests: row.total_requests,
            timestamp: row.timestamp,
        });
    }

    HttpResponse::Ok().json(CanaryAnalytics {
        total_policies,
        active_canaries,
        policies: policies_status,
        recent_promotions,
        recent_rollbacks,
    })
}

/// Pre-flight impact analysis - preview what would break before creating a policy
#[utoipa::path(
    get,
    path = "/policies/preview-impact",
    tag = "Policy Management",
    params(
        ("policy_type" = String, Query, description = "Policy type: SOVEREIGN_LOCK, AGENT_REVOCATION, etc."),
        ("time_range_days" = Option<i64>, Query, description = "Time range in days (default: 7)"),
        ("agent_filter" = Option<String>, Query, description = "Comma-separated agent IDs to filter"),
    ),
    responses(
        (status = 200, description = "Impact preview", body = SimulationResult),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn preview_policy_impact(
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "policy", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let policy_type_str = query.get("policy_type")
        .ok_or_else(|| HttpResponse::BadRequest().json(serde_json::json!({
            "error": "MISSING_POLICY_TYPE",
            "message": "policy_type parameter is required"
        })))
        .unwrap();
    
    let policy_type = match policy_type_str.as_str() {
        "SOVEREIGN_LOCK" => crate::core::policy_simulator::PolicyType::SovereignLock,
        "AGENT_REVOCATION" => crate::core::policy_simulator::PolicyType::AgentRevocation,
        "CONSENT_REQUIREMENT" => crate::core::policy_simulator::PolicyType::ConsentRequirement,
        "PROCESSING_RESTRICTION" => crate::core::policy_simulator::PolicyType::ProcessingRestriction,
        _ => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "INVALID_POLICY_TYPE",
            "message": "policy_type must be one of: SOVEREIGN_LOCK, AGENT_REVOCATION, CONSENT_REQUIREMENT, PROCESSING_RESTRICTION"
        }))
    };
    
    let time_range_days = query.get("time_range_days")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(7);
    
    let agent_filter = query.get("agent_filter")
        .map(|s| s.split(',').map(|id| id.trim().to_string()).collect());
    
    // Default policy config based on type
    let policy_config = match policy_type {
        crate::core::policy_simulator::PolicyType::SovereignLock => {
            serde_json::json!({
                "blocked_countries": ["US", "CN", "RU"]
            })
        }
        _ => serde_json::json!({})
    };
    
    let simulation_request = crate::core::policy_simulator::SimulationRequest {
        policy_type: policy_type.clone(),
        policy_config,
        time_range_days: Some(time_range_days),
        agent_filter,
        business_function_filter: query.get("business_function").map(|s| s.split(',').map(|id| id.trim().to_string()).collect()),
        location_filter: query.get("location").map(|s| s.split(',').map(|id| id.trim().to_string()).collect()),
        time_offset_days: query.get("time_offset_days").and_then(|s| s.parse::<i64>().ok()),
    };
    
    match crate::core::policy_simulator::PolicySimulator::simulate(
        &data.db_pool,
        simulation_request.clone(),
    ).await {
        Ok(result) => {
            // Get affected systems from asset registry
            let mut affected_systems: Vec<serde_json::Value> = Vec::new();
            for agent_id in &result.critical_agents {
                if let Ok(Some(asset)) = crate::core::asset_policy_engine::AssetPolicyEngine::get_asset_context_from_agent(
                    &data.db_pool,
                    agent_id,
                ).await {
                    affected_systems.push(serde_json::json!({
                        "agent_id": agent_id,
                        "asset_id": asset.asset_id,
                        "business_function": asset.business_function,
                        "location": asset.location,
                        "risk_profile": asset.risk_profile,
                        "impact_level": "CRITICAL"
                    }));
                }
            }
            for agent_id in &result.partial_impact_agents {
                if let Ok(Some(asset)) = crate::core::asset_policy_engine::AssetPolicyEngine::get_asset_context_from_agent(
                    &data.db_pool,
                    agent_id,
                ).await {
                    affected_systems.push(serde_json::json!({
                        "agent_id": agent_id,
                        "asset_id": asset.asset_id,
                        "business_function": asset.business_function,
                        "location": asset.location,
                        "risk_profile": asset.risk_profile,
                        "impact_level": "PARTIAL"
                    }));
                }
            }

            // Group by business function
            let mut business_function_impact: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            for system in &affected_systems {
                if let Some(bf) = system.get("business_function").and_then(|v| v.as_str()) {
                    *business_function_impact.entry(bf.to_string()).or_insert(0) += 1;
                }
            }

            // Estimate transaction volume impact
            // Assume each blocked request represents a transaction
            let estimated_transaction_volume_affected = result.would_block;
            let estimated_daily_transactions = if time_range_days > 0 {
                estimated_transaction_volume_affected / time_range_days
            } else {
                0
            };

            // Calculate confidence score based on sample size and time range
            let confidence_score = if result.total_requests >= 10000 {
                95.0 // High confidence with large sample
            } else if result.total_requests >= 1000 {
                85.0 // Medium-high confidence
            } else if result.total_requests >= 100 {
                70.0 // Medium confidence
            } else if result.total_requests >= 10 {
                50.0 // Low confidence
            } else {
                30.0 // Very low confidence - need more data
            };

            // Historical trend analysis (compare with previous periods)
            let historical_analysis = if time_range_days >= 7 {
                // Get data for previous period
                let prev_start = Utc::now() - chrono::Duration::days(time_range_days * 2);
                let prev_end = Utc::now() - chrono::Duration::days(time_range_days);
                
                let prev_total: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM compliance_records WHERE timestamp >= $1 AND timestamp < $2"
                )
                .bind(prev_start)
                .bind(prev_end)
                .fetch_one(&data.db_pool)
                .await
                .unwrap_or(0);

                let trend = if prev_total > 0 {
                    let change = ((result.total_requests as f64 - prev_total as f64) / prev_total as f64) * 100.0;
                    if change > 10.0 {
                        "INCREASING"
                    } else if change < -10.0 {
                        "DECREASING"
                    } else {
                        "STABLE"
                    }
                } else {
                    "NO_BASELINE"
                };

                serde_json::json!({
                    "previous_period_total": prev_total,
                    "current_period_total": result.total_requests,
                    "trend": trend,
                    "percentage_change": if prev_total > 0 {
                        ((result.total_requests as f64 - prev_total as f64) / prev_total as f64) * 100.0
                    } else {
                        0.0
                    }
                })
            } else {
                serde_json::json!({
                    "previous_period_total": null,
                    "current_period_total": result.total_requests,
                    "trend": "INSUFFICIENT_DATA",
                    "percentage_change": null
                })
            };

            // Enhanced business impact estimation
            let business_impact = serde_json::json!({
                "estimated_transaction_volume_affected": estimated_transaction_volume_affected,
                "estimated_daily_transactions_affected": estimated_daily_transactions,
                "estimated_revenue_impact": null, // Would need business metrics integration
                "critical_systems_affected": result.critical_agents.len(),
                "partial_impact_systems": result.partial_impact_agents.len(),
                "affected_business_functions": business_function_impact,
                "recommendation": match result.estimated_impact {
                    crate::core::policy_simulator::ImpactLevel::Low => "Safe to deploy - Low impact expected",
                    crate::core::policy_simulator::ImpactLevel::Medium => "Deploy with caution - Monitor closely for 24-48 hours",
                    crate::core::policy_simulator::ImpactLevel::High => "High risk - Use gradual rollout (canary deployment) and shadow mode",
                    crate::core::policy_simulator::ImpactLevel::Critical => "CRITICAL RISK - Do not deploy without mitigation plan. Consider policy adjustment or phased rollout.",
                },
                "deployment_strategy": match result.estimated_impact {
                    crate::core::policy_simulator::ImpactLevel::Low => "Direct deployment to ENFORCING mode",
                    crate::core::policy_simulator::ImpactLevel::Medium => "Start with SHADOW mode, then canary at 1%",
                    crate::core::policy_simulator::ImpactLevel::High => "SHADOW mode → 1% canary → 5% → 10% → 25% → 50% → 100%",
                    crate::core::policy_simulator::ImpactLevel::Critical => "SHADOW mode only - Review policy configuration before considering rollout",
                }
            });
            
            HttpResponse::Ok().json(serde_json::json!({
                "simulation_result": result,
                "business_impact": business_impact,
                "affected_systems": affected_systems,
                "confidence_score": confidence_score,
                "historical_analysis": historical_analysis,
                "preview_mode": true,
                "time_range_days": time_range_days
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "SIMULATION_FAILED",
            "message": format!("Failed to preview impact: {}", e)
        }))
    }
}

/// Compare two policy configurations
#[utoipa::path(
    post,
    path = "/policies/compare",
    tag = "Policy Management",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Policy comparison result"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn compare_policies(
    req: web::Json<serde_json::Value>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "policy", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let body = req.into_inner();
    
    let policy_type_a = match body.get("policy_type_a").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "MISSING_POLICY_TYPE_A",
            "message": "policy_type_a is required"
        })),
    };
    
    let policy_config_a = body.get("policy_config_a")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    
    let policy_type_b = match body.get("policy_type_b").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "MISSING_POLICY_TYPE_B",
            "message": "policy_type_b is required"
        })),
    };
    
    let policy_config_b = body.get("policy_config_b")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    
    let time_range_days = body.get("time_range_days")
        .and_then(|v| v.as_i64())
        .unwrap_or(7);

    // Parse policy types
    let policy_type_a_enum = match policy_type_a {
        "SOVEREIGN_LOCK" => crate::core::policy_simulator::PolicyType::SovereignLock,
        "AGENT_REVOCATION" => crate::core::policy_simulator::PolicyType::AgentRevocation,
        "CONSENT_REQUIREMENT" => crate::core::policy_simulator::PolicyType::ConsentRequirement,
        "PROCESSING_RESTRICTION" => crate::core::policy_simulator::PolicyType::ProcessingRestriction,
        _ => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "INVALID_POLICY_TYPE_A",
            "message": "Invalid policy_type_a"
        }))
    };

    let policy_type_b_enum = match policy_type_b {
        "SOVEREIGN_LOCK" => crate::core::policy_simulator::PolicyType::SovereignLock,
        "AGENT_REVOCATION" => crate::core::policy_simulator::PolicyType::AgentRevocation,
        "CONSENT_REQUIREMENT" => crate::core::policy_simulator::PolicyType::ConsentRequirement,
        "PROCESSING_RESTRICTION" => crate::core::policy_simulator::PolicyType::ProcessingRestriction,
        _ => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "INVALID_POLICY_TYPE_B",
            "message": "Invalid policy_type_b"
        }))
    };

    // Run both simulations
    let sim_a = crate::core::policy_simulator::PolicySimulator::simulate(
        &data.db_pool,
        crate::core::policy_simulator::SimulationRequest {
            policy_type: policy_type_a_enum.clone(),
            policy_config: policy_config_a,
            time_range_days: Some(time_range_days),
            agent_filter: None,
            business_function_filter: None,
            location_filter: None,
            time_offset_days: None,
        },
    ).await;

    let sim_b = crate::core::policy_simulator::PolicySimulator::simulate(
        &data.db_pool,
        crate::core::policy_simulator::SimulationRequest {
            policy_type: policy_type_b_enum.clone(),
            policy_config: policy_config_b,
            time_range_days: Some(time_range_days),
            agent_filter: None,
            business_function_filter: None,
            location_filter: None,
            time_offset_days: None,
        },
    ).await;

    match (sim_a, sim_b) {
        (Ok(result_a), Ok(result_b)) => {
            let block_diff = result_b.would_block - result_a.would_block;
            let block_diff_percentage = if result_a.total_requests > 0 {
                (block_diff as f64 / result_a.total_requests as f64) * 100.0
            } else {
                0.0
            };

            HttpResponse::Ok().json(serde_json::json!({
                "policy_a": {
                    "policy_type": policy_type_a,
                    "would_block": result_a.would_block,
                    "would_allow": result_a.would_allow,
                    "estimated_impact": format!("{:?}", result_a.estimated_impact),
                    "critical_agents": result_a.critical_agents.len(),
                    "partial_impact_agents": result_a.partial_impact_agents.len(),
                },
                "policy_b": {
                    "policy_type": policy_type_b,
                    "would_block": result_b.would_block,
                    "would_allow": result_b.would_allow,
                    "estimated_impact": format!("{:?}", result_b.estimated_impact),
                    "critical_agents": result_b.critical_agents.len(),
                    "partial_impact_agents": result_b.partial_impact_agents.len(),
                },
                "comparison": {
                    "block_difference": block_diff,
                    "block_difference_percentage": block_diff_percentage,
                    "more_restrictive": if block_diff > 0 { "POLICY_B" } else if block_diff < 0 { "POLICY_A" } else { "EQUAL" },
                    "recommendation": if (block_diff.abs() as f64) < (result_a.total_requests as f64 * 0.05) {
                        "Both policies have similar impact - choose based on business requirements".to_string()
                    } else if block_diff > 0 {
                        format!("Policy B would block {} more requests ({:.2}% increase). Consider if additional restrictions are necessary.", block_diff, block_diff_percentage.abs())
                    } else {
                        format!("Policy A would block {} more requests ({:.2}% increase). Policy B is less restrictive.", block_diff.abs(), block_diff_percentage.abs())
                    }
                }
            }))
        }
        (Err(e), _) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "SIMULATION_A_FAILED",
            "message": format!("Failed to simulate policy A: {}", e)
        })),
        (_, Err(e)) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "SIMULATION_B_FAILED",
            "message": format!("Failed to simulate policy B: {}", e)
        })),
    }
}

/// Get real-time policy health metrics
#[derive(Serialize, ToSchema)]
pub struct PolicyHealthResponse {
    pub policy_id: Uuid,
    pub policy_name: String,
    pub policy_type: String,
    pub is_active: bool,
    pub rollout_percentage: i32,
    pub circuit_breaker_state: Option<String>,
    pub success_rate: f64,
    pub error_rate: f64,
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
    pub blocked_requests: i64,
    pub health_status: String, // "HEALTHY", "DEGRADED", "CRITICAL"
    pub last_updated: DateTime<Utc>,
    pub avg_latency_ms: Option<f64>,
    pub p95_latency_ms: Option<f64>,
    pub p99_latency_ms: Option<f64>,
}

#[utoipa::path(
    get,
    path = "/policies/{policy_id}/health",
    tag = "Policy Management",
    params(
        ("policy_id" = Uuid, Path, description = "Policy ID"),
    ),
    responses(
        (status = 200, description = "Policy health status", body = PolicyHealthResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_policy_health(
    policy_id: web::Path<Uuid>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "policy", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let policy_id = policy_id.into_inner();
    
    #[derive(sqlx::FromRow)]
    struct PolicyDb {
        policy_name: String,
        policy_type: String,
        is_active: bool,
        rollout_percentage: i32,
        circuit_breaker_state: Option<String>,
    }
    
    let policy: Option<PolicyDb> = sqlx::query_as(
        "SELECT policy_name, policy_type, is_active, rollout_percentage, circuit_breaker_state
         FROM policy_versions
         WHERE id = $1"
    )
    .bind(policy_id)
    .fetch_optional(&data.db_pool)
    .await
    .ok()
    .flatten();
    
    match policy {
        Some(p) => {
            // Get canary metrics (last 10 minutes)
            let now = chrono::Utc::now();
            let window_start = now - chrono::Duration::minutes(10);
            
            #[derive(sqlx::FromRow)]
            struct MetricsRow {
                total_requests: Option<i64>,
                successful_requests: Option<i64>,
                failed_requests: Option<i64>,
                blocked_requests: Option<i64>,
            }
            
            let metrics: MetricsRow = sqlx::query_as(
                "SELECT 
                    SUM(total_requests) as total_requests,
                    SUM(successful_requests) as successful_requests,
                    SUM(failed_requests) as failed_requests,
                    SUM(blocked_requests) as blocked_requests
                 FROM canary_metrics
                 WHERE policy_version_id = $1
                   AND window_start >= $2"
            )
            .bind(policy_id)
            .bind(window_start)
            .fetch_one(&data.db_pool)
            .await
            .unwrap_or(MetricsRow {
                total_requests: Some(0),
                successful_requests: Some(0),
                failed_requests: Some(0),
                blocked_requests: Some(0),
            });

            // Get latency metrics from compliance_records (if available)
            #[derive(sqlx::FromRow)]
            struct LatencyRow {
                avg_latency: Option<f64>,
                p95_latency: Option<f64>,
                p99_latency: Option<f64>,
            }

            let latency: LatencyRow = sqlx::query_as(
                "SELECT 
                    AVG(EXTRACT(EPOCH FROM (completed_at - timestamp)) * 1000) as avg_latency,
                    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY EXTRACT(EPOCH FROM (completed_at - timestamp)) * 1000) as p95_latency,
                    PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY EXTRACT(EPOCH FROM (completed_at - timestamp)) * 1000) as p99_latency
                 FROM compliance_records
                 WHERE policy_version_id = $1
                   AND timestamp >= $2
                   AND completed_at IS NOT NULL"
            )
            .bind(policy_id)
            .bind(window_start)
            .fetch_one(&data.db_pool)
            .await
            .unwrap_or(LatencyRow {
                avg_latency: None,
                p95_latency: None,
                p99_latency: None,
            });

            let total = metrics.total_requests.unwrap_or(0);
            let successful = metrics.successful_requests.unwrap_or(0);
            let failed = metrics.failed_requests.unwrap_or(0);
            let blocked = metrics.blocked_requests.unwrap_or(0);
            
            let success_rate = if total > 0 {
                (successful as f64 / total as f64) * 100.0
            } else {
                100.0
            };
            
            let error_rate = if total > 0 {
                ((failed + blocked) as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            
            // Determine health status
            let health_status = if error_rate >= 10.0 || p.circuit_breaker_state.as_deref() == Some("OPEN") {
                "CRITICAL"
            } else if error_rate >= 5.0 {
                "DEGRADED"
            } else {
                "HEALTHY"
            };
            
            HttpResponse::Ok().json(PolicyHealthResponse {
                policy_id,
                policy_name: p.policy_name,
                policy_type: p.policy_type,
                is_active: p.is_active,
                rollout_percentage: p.rollout_percentage,
                circuit_breaker_state: p.circuit_breaker_state,
                success_rate,
                error_rate,
                total_requests: total,
                successful_requests: successful,
                failed_requests: failed,
                blocked_requests: blocked,
                health_status: health_status.to_string(),
                last_updated: now,
                avg_latency_ms: latency.avg_latency,
                p95_latency_ms: latency.p95_latency,
                p99_latency_ms: latency.p99_latency,
            })
        }
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": "POLICY_NOT_FOUND",
            "message": "Policy not found"
        }))
    }
}

/// Approve a policy (for multi-step approval workflow)
#[derive(Deserialize, Serialize, ToSchema)]
pub struct PolicyApprovalRequest {
    #[schema(example = "Approved after review")]
    pub notes: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct PolicyApprovalResponse {
    pub policy_id: Uuid,
    pub approver_id: String,
    pub approval_count: i64,
    pub required_count: i32,
    pub can_activate: bool,
    pub status: String,
}

#[utoipa::path(
    post,
    path = "/policies/{policy_id}/approve",
    tag = "Policy Management",
    params(
        ("policy_id" = Uuid, Path, description = "Policy ID"),
    ),
    request_body = PolicyApprovalRequest,
    responses(
        (status = 200, description = "Policy approved"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn approve_policy(
    policy_id: web::Path<Uuid>,
    req: web::Json<PolicyApprovalRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let claims = match authenticate_and_authorize(&http_req, &data.db_pool, "policy", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let policy_id = policy_id.into_inner();
    let approval_req = req.into_inner();
    
    // Check if policy exists and requires approval
    #[derive(sqlx::FromRow)]
    struct PolicyCheck {
        requires_approval: bool,
        approval_required_count: i32,
    }
    
    let policy_check: Option<PolicyCheck> = sqlx::query_as(
        "SELECT requires_approval, approval_required_count
         FROM policy_versions
         WHERE id = $1"
    )
    .bind(policy_id)
    .fetch_optional(&data.db_pool)
    .await
    .ok()
    .flatten();
    
    match policy_check {
        Some(p) => {
            if !p.requires_approval {
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "APPROVAL_NOT_REQUIRED",
                    "message": "This policy does not require approval"
                }));
            }
            
            // Check if user can approve (directly or via delegation)
            // First, get the list of required approvers
            let required_approvers: Vec<String> = sqlx::query_scalar(
                "SELECT DISTINCT approver_id 
                 FROM policy_approvals 
                 WHERE policy_version_id = $1 AND action = 'APPROVED'"
            )
            .bind(policy_id)
            .fetch_all(&data.db_pool)
            .await
            .unwrap_or_default();

            // Check if current user can approve (either directly or via delegation)
            let can_approve: Option<bool> = sqlx::query_scalar(
                "SELECT can_approve_on_behalf($1, $2)"
            )
            .bind(&claims.sub)
            .bind(&claims.sub) // For now, check direct approval
            .fetch_optional(&data.db_pool)
            .await
            .ok()
            .flatten();

            // Also check if user has delegated authority from any required approver
            let has_delegated_authority = if !required_approvers.is_empty() {
                let mut has_delegation = false;
                for approver in &required_approvers {
                    let delegated: Option<bool> = sqlx::query_scalar(
                        "SELECT can_approve_on_behalf($1, $2)"
                    )
                    .bind(&claims.sub)
                    .bind(approver)
                    .fetch_optional(&data.db_pool)
                    .await
                    .ok()
                    .flatten();
                    if delegated.unwrap_or(false) {
                        has_delegation = true;
                        break;
                    }
                }
                has_delegation
            } else {
                // If no required approvers yet, check if user has policy:write permission
                let has_permission: Option<bool> = sqlx::query_scalar(
                    "SELECT EXISTS(
                        SELECT 1 FROM user_permissions 
                        WHERE user_id = $1 AND permission = 'policy:write'
                    )"
                )
                .bind(&claims.sub)
                .fetch_optional(&data.db_pool)
                .await
                .ok()
                .flatten();
                has_permission.unwrap_or(false)
            };

            if !can_approve.unwrap_or(false) && !has_delegated_authority {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "APPROVAL_NOT_AUTHORIZED",
                    "message": "You do not have permission to approve this policy"
                }));
            }

            // Record approval (store original approver if delegated)
            let original_approver = if has_delegated_authority && !required_approvers.is_empty() {
                required_approvers.first().cloned()
            } else {
                None
            };

            let _ = sqlx::query(
                "INSERT INTO policy_approvals (policy_version_id, approver_id, action, notes)
                 VALUES ($1, $2, 'APPROVED', $3)
                 ON CONFLICT (policy_version_id, approver_id) 
                 DO UPDATE SET action = 'APPROVED', notes = $3, approved_at = CURRENT_TIMESTAMP"
            )
            .bind(policy_id)
            .bind(&claims.sub)
            .bind(&approval_req.notes)
            .execute(&data.db_pool)
            .await;
            
            // Get approval count
            let approval_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM policy_approvals 
                 WHERE policy_version_id = $1 AND action = 'APPROVED'"
            )
            .bind(policy_id)
            .fetch_one(&data.db_pool)
            .await
            .unwrap_or(0);
            
            // Check if can activate
            let can_activate: Option<bool> = sqlx::query_scalar(
                "SELECT can_activate_policy($1)"
            )
            .bind(policy_id)
            .fetch_optional(&data.db_pool)
            .await
            .ok()
            .flatten();
            
            let status = if can_activate.unwrap_or(false) {
                "APPROVED"
            } else {
                "PENDING"
            };
            
            // Update policy approval status if fully approved
            if can_activate.unwrap_or(false) {
                let _ = sqlx::query(
                    "UPDATE policy_versions 
                     SET approval_status = 'APPROVED'
                     WHERE id = $1"
                )
                .bind(policy_id)
                .execute(&data.db_pool)
                .await;

                // Send notification to policy creator that approval is complete
                let policy_creator: Option<String> = sqlx::query_scalar(
                    "SELECT created_by FROM policy_versions WHERE id = $1"
                )
                .bind(policy_id)
                .fetch_optional(&data.db_pool)
                .await
                .ok()
                .flatten();

                if let Some(creator) = policy_creator {
                    let db_pool_clone = data.db_pool.clone();
                    let policy_id_clone = policy_id;
                    let required_count = p.approval_required_count;
                    let received_count = approval_count;
                    tokio::spawn(async move {
                        let notification_service = crate::integration::notifications::NotificationService::new();
                        let request = crate::integration::notifications::NotificationRequest {
                            user_id: creator,
                            notification_type: crate::integration::notifications::NotificationType::HighRiskAiAction,
                            channel: crate::integration::notifications::NotificationChannel::Email,
                            subject: Some("Policy Approved - Ready to Activate".to_string()),
                            body: format!(
                                "Your policy has received all required approvals and is ready to be activated.\n\n\
                                Policy ID: {}\n\
                                Required Approvals: {}\n\
                                Received Approvals: {}\n\n\
                                You can now activate the policy from the dashboard.",
                                policy_id_clone, required_count, received_count
                            ),
                            language: Some("en".to_string()),
                            related_entity_type: Some("POLICY".to_string()),
                            related_entity_id: Some(policy_id_clone.to_string()),
                        };
                        let _ = notification_service.send_notification(&db_pool_clone, request).await;
                    });
                }
            } else {
                // Send notification to other approvers that approval is needed
                let existing_approvers: Vec<String> = sqlx::query_scalar(
                    "SELECT DISTINCT approver_id FROM policy_approvals WHERE policy_version_id = $1"
                )
                .bind(policy_id)
                .fetch_all(&data.db_pool)
                .await
                .unwrap_or_default();

                let remaining_approvals = p.approval_required_count - approval_count as i32;
                if remaining_approvals > 0 {
                    // Get list of potential approvers (users with policy:write permission)
                    let potential_approvers: Vec<String> = sqlx::query_scalar(
                        "SELECT DISTINCT user_id FROM user_permissions WHERE permission = 'policy:write'"
                    )
                    .fetch_all(&data.db_pool)
                    .await
                    .unwrap_or_default();

                    let approvers_set: std::collections::HashSet<String> = existing_approvers.iter()
                        .cloned()
                        .collect();

                    for approver in potential_approvers {
                        if !approvers_set.contains(&approver) && approver != claims.sub {
                            let db_pool_clone = data.db_pool.clone();
                            let policy_id_clone = policy_id;
                            let remaining = remaining_approvals;
                            let required_count = p.approval_required_count;
                            tokio::spawn(async move {
                                let notification_service = crate::integration::notifications::NotificationService::new();
                                let request = crate::integration::notifications::NotificationRequest {
                                    user_id: approver,
                                    notification_type: crate::integration::notifications::NotificationType::HighRiskAiAction,
                                    channel: crate::integration::notifications::NotificationChannel::Email,
                                    subject: Some("Policy Approval Required".to_string()),
                                    body: format!(
                                        "A policy requires your approval.\n\n\
                                        Policy ID: {}\n\
                                        Required Approvals: {}\n\
                                        Remaining Approvals Needed: {}\n\n\
                                        Please review and approve or reject the policy from the Approval Queue dashboard.",
                                        policy_id_clone, required_count, remaining
                                    ),
                                    language: Some("en".to_string()),
                                    related_entity_type: Some("POLICY".to_string()),
                                    related_entity_id: Some(policy_id_clone.to_string()),
                                };
                                let _ = notification_service.send_notification(&db_pool_clone, request).await;
                            });
                        }
                    }
                }
            }
            
            HttpResponse::Ok().json(PolicyApprovalResponse {
                policy_id,
                approver_id: claims.sub,
                approval_count,
                required_count: p.approval_required_count,
                can_activate: can_activate.unwrap_or(false),
                status: status.to_string(),
            })
        }
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": "POLICY_NOT_FOUND",
            "message": "Policy not found"
        }))
    }
}

/// Reject a policy (for multi-step approval workflow)
#[derive(Deserialize, ToSchema)]
pub struct PolicyRejectionRequest {
    #[schema(example = "Rejected due to high risk")]
    pub reason: String,
}

#[utoipa::path(
    post,
    path = "/policies/{policy_id}/reject",
    tag = "Policy Management",
    params(
        ("policy_id" = Uuid, Path, description = "Policy ID"),
    ),
    request_body = PolicyRejectionRequest,
    responses(
        (status = 200, description = "Policy rejected"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn reject_policy(
    policy_id: web::Path<Uuid>,
    req: web::Json<PolicyRejectionRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let claims = match authenticate_and_authorize(&http_req, &data.db_pool, "policy", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let policy_id = policy_id.into_inner();
    let rejection_req = req.into_inner();
    
    // Record rejection
    let _ = sqlx::query(
        "INSERT INTO policy_approvals (policy_version_id, approver_id, action, notes)
         VALUES ($1, $2, 'REJECTED', $3)
         ON CONFLICT (policy_version_id, approver_id) 
         DO UPDATE SET action = 'REJECTED', notes = $3, approved_at = CURRENT_TIMESTAMP"
    )
    .bind(policy_id)
    .bind(&claims.sub)
    .bind(&rejection_req.reason)
    .execute(&data.db_pool)
    .await;
    
    // Update policy approval status
    let _ = sqlx::query(
        "UPDATE policy_versions 
         SET approval_status = 'REJECTED', approval_notes = $1
         WHERE id = $2"
    )
    .bind(&rejection_req.reason)
    .bind(policy_id)
    .execute(&data.db_pool)
    .await;
    
    HttpResponse::Ok().json(serde_json::json!({
        "status": "REJECTED",
        "message": "Policy rejected",
        "rejected_by": claims.sub
    }))
}

// ========== VERIDION TPRM INTEGRATION ==========

/// Get vendor risk score from Veridion TPRM
#[derive(Serialize, ToSchema)]
pub struct VendorRiskScoreResponse {
    pub vendor_domain: String,
    pub vendor_name: Option<String>,
    pub risk_score: f64,
    pub risk_level: String,
    pub compliance_status: String,
    pub country_code: Option<String>,
    pub industry_sector: Option<String>,
    pub last_updated: DateTime<Utc>,
}

#[utoipa::path(
    get,
    path = "/vendors/{vendor_domain}/risk-score",
    tag = "Veridion TPRM",
    params(
        ("vendor_domain" = String, Path, description = "Vendor domain"),
    ),
    responses(
        (status = 200, description = "Vendor risk score", body = VendorRiskScoreResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_vendor_risk_score(
    vendor_domain: web::Path<String>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "tprm", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let vendor_domain = vendor_domain.into_inner();
    let tprm_service = crate::integration::veridion_tprm::VeridionTPRMService::new(
        std::env::var("VERIDION_TPRM_API_KEY").ok()
    );

    // Fetch risk score
    match tprm_service.fetch_vendor_risk_score(&vendor_domain).await {
        Ok(risk_score) => {
            // Store in database
            let _ = tprm_service.store_vendor_risk_score(&data.db_pool, &risk_score).await;

            HttpResponse::Ok().json(VendorRiskScoreResponse {
                vendor_domain: risk_score.vendor_domain,
                vendor_name: risk_score.vendor_name,
                risk_score: risk_score.risk_score,
                risk_level: risk_score.risk_level,
                compliance_status: risk_score.compliance_status,
                country_code: risk_score.country_code,
                industry_sector: risk_score.industry_sector,
                last_updated: Utc::now(),
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "TPRM_FETCH_FAILED",
            "message": format!("Failed to fetch vendor risk score: {}", e)
        }))
    }
}

/// Enrich asset with TPRM data
#[utoipa::path(
    post,
    path = "/assets/{asset_id}/enrich-tprm",
    tag = "TPRM Integration",
    responses(
        (status = 200, description = "Asset enriched with TPRM data"),
        (status = 404, description = "Asset not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn enrich_asset_tprm(
    asset_id: web::Path<Uuid>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "tprm", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let asset_id = asset_id.into_inner();
    let tprm_service = crate::integration::veridion_tprm::VeridionTPRMService::new(
        std::env::var("VERIDION_TPRM_API_KEY").ok()
    );

    match tprm_service.enrich_asset_with_tprm(&data.db_pool, asset_id).await {
        Ok(enriched_data) => {
            HttpResponse::Ok().json(serde_json::json!({
                "asset_id": asset_id,
                "vendors_enriched": enriched_data.len(),
                "vendor_risk_scores": enriched_data.iter().map(|(domain, score)| {
                    serde_json::json!({
                        "vendor_domain": domain,
                        "risk_score": score.risk_score,
                        "risk_level": score.risk_level,
                        "compliance_status": score.compliance_status
                    })
                }).collect::<Vec<_>>()
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "TPRM_ENRICHMENT_FAILED",
            "message": format!("Failed to enrich asset: {}", e)
        }))
    }
}

/// Auto-generate policies from TPRM data
#[derive(Deserialize, ToSchema)]
pub struct AutoGenerateTPRMPolicyRequest {
    #[schema(example = "asset-uuid")]
    pub asset_id: Option<Uuid>,
    #[schema(example = "openai.com")]
    pub vendor_domain: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct TPRMPolicyRecommendationsResponse {
    pub recommendations: Vec<TPRMPolicyRecommendation>,
    pub total_count: usize,
}

#[derive(Serialize, ToSchema)]
pub struct TPRMPolicyRecommendation {
    pub vendor_domain: String,
    pub recommendation_type: String,
    pub risk_reason: String,
    pub suggested_policy_config: serde_json::Value,
    pub priority: i32,
}

#[utoipa::path(
    post,
    path = "/policies/auto-generate-from-tprm",
    tag = "Veridion TPRM",
    request_body = AutoGenerateTPRMPolicyRequest,
    responses(
        (status = 200, description = "Policies generated"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn auto_generate_tprm_policies(
    req: web::Json<AutoGenerateTPRMPolicyRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "tprm", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let req_data = req.into_inner();
    let tprm_service = crate::integration::veridion_tprm::VeridionTPRMService::new(
        std::env::var("VERIDION_TPRM_API_KEY").ok()
    );

    if let Some(asset_id) = req_data.asset_id {
        match tprm_service.generate_tprm_policy_recommendations(&data.db_pool, asset_id).await {
            Ok(recommendations) => {
                // Store recommendations in database
                for rec in &recommendations {
                    let _ = sqlx::query(
                        "INSERT INTO tprm_policy_recommendations (
                            asset_id, vendor_domain, recommendation_type,
                            risk_reason, suggested_policy_config, priority
                        ) VALUES ($1, $2, $3, $4, $5, $6)"
                    )
                    .bind(asset_id)
                    .bind(&rec.vendor_domain)
                    .bind(&rec.recommendation_type)
                    .bind(&rec.risk_reason)
                    .bind(&rec.suggested_policy_config)
                    .bind(rec.priority)
                    .execute(&data.db_pool)
                    .await;
                }

                HttpResponse::Ok().json(TPRMPolicyRecommendationsResponse {
                    recommendations: recommendations.iter().map(|r| TPRMPolicyRecommendation {
                        vendor_domain: r.vendor_domain.clone(),
                        recommendation_type: r.recommendation_type.clone(),
                        risk_reason: r.risk_reason.clone(),
                        suggested_policy_config: r.suggested_policy_config.clone(),
                        priority: r.priority,
                    }).collect(),
                    total_count: recommendations.len(),
                })
            }
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "TPRM_GENERATION_FAILED",
                "message": format!("Failed to generate recommendations: {}", e)
            }))
        }
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "error": "MISSING_ASSET_ID",
            "message": "asset_id is required"
        }))
    }
}

// ========== EXECUTIVE ASSURANCE REPORTING ==========

/// Get executive compliance scorecard (Board-level reporting)
#[derive(Serialize, ToSchema)]
pub struct ExecutiveScorecardResponse {
    pub report_date: String,
    pub compliance_score: f64,
    pub risk_level: String,
    pub liability_protection_status: String,
    pub nis2_readiness: f64,
    pub dora_compliance: bool,
    pub total_assets: i32,
    pub compliant_assets: i32,
    pub non_compliant_assets: i32,
    pub critical_issues_count: i32,
    pub high_risk_issues_count: i32,
    pub last_incident_date: Option<String>,
    pub days_since_last_incident: Option<i32>,
    pub executive_summary: String,
    pub recommendations: Vec<String>,
}

#[utoipa::path(
    get,
    path = "/reports/executive-assurance",
    tag = "Executive Assurance",
    responses(
        (status = 200, description = "Executive assurance report", body = ExecutiveScorecardResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_executive_assurance(
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "reports", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let report_date = query.get("report_date")
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    match crate::core::executive_assurance::ExecutiveAssuranceService::generate_scorecard(
        &data.db_pool,
        report_date,
    ).await {
        Ok(scorecard) => {
            // Generate recommendations
            let mut recommendations = Vec::new();
            
            if scorecard.compliance_score < 90.0 {
                recommendations.push(format!(
                    "Improve compliance score from {:.1}% to 90%+ to achieve PROTECTED liability status",
                    scorecard.compliance_score
                ));
            }
            
            if scorecard.nis2_readiness < 90.0 {
                recommendations.push(format!(
                    "Increase NIS2 readiness from {:.1}% to 90%+ to meet Article 20 requirements",
                    scorecard.nis2_readiness
                ));
            }
            
            if scorecard.critical_issues_count > 0 {
                recommendations.push(format!(
                    "Address {} critical issue(s) immediately to reduce liability exposure",
                    scorecard.critical_issues_count
                ));
            }
            
            if !scorecard.dora_compliance {
                recommendations.push("Enable DORA compliance tracking to meet Article 28 requirements".to_string());
            }

            HttpResponse::Ok().json(ExecutiveScorecardResponse {
                report_date: scorecard.report_date.to_string(),
                compliance_score: scorecard.compliance_score,
                risk_level: scorecard.risk_level,
                liability_protection_status: scorecard.liability_protection_status,
                nis2_readiness: scorecard.nis2_readiness,
                dora_compliance: scorecard.dora_compliance,
                total_assets: scorecard.total_assets,
                compliant_assets: scorecard.compliant_assets,
                non_compliant_assets: scorecard.non_compliant_assets,
                critical_issues_count: scorecard.critical_issues_count,
                high_risk_issues_count: scorecard.high_risk_issues_count,
                last_incident_date: scorecard.last_incident_date.map(|dt| dt.to_rfc3339()),
                days_since_last_incident: scorecard.days_since_last_incident,
                executive_summary: scorecard.executive_summary,
                recommendations,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "SCORECARD_GENERATION_FAILED",
            "message": format!("Failed to generate scorecard: {}", e)
        }))
    }
}

/// Get compliance KPIs
#[derive(Serialize, ToSchema)]
pub struct ComplianceKPIResponse {
    pub kpi_name: String,
    pub kpi_value: f64,
    pub kpi_unit: String,
    pub kpi_category: String,
    pub target_value: Option<f64>,
    pub status: String,
}

#[utoipa::path(
    get,
    path = "/reports/compliance-kpis",
    tag = "Executive Assurance",
    responses(
        (status = 200, description = "Compliance KPIs", body = ComplianceKPIResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_compliance_kpis(
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "reports", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let category = query.get("category").map(|s| s.as_str());

    match crate::core::executive_assurance::ExecutiveAssuranceService::get_compliance_kpis(
        &data.db_pool,
        category,
    ).await {
        Ok(kpis) => {
            HttpResponse::Ok().json(kpis.iter().map(|kpi| ComplianceKPIResponse {
                kpi_name: kpi.kpi_name.clone(),
                kpi_value: kpi.kpi_value,
                kpi_unit: kpi.kpi_unit.clone(),
                kpi_category: kpi.kpi_category.clone(),
                target_value: kpi.target_value,
                status: kpi.status.clone(),
            }).collect::<Vec<_>>())
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "KPI_FETCH_FAILED",
            "message": format!("Failed to fetch KPIs: {}", e)
        }))
    }
}

// ========== POLICY HEALTH DASHBOARD ==========

#[derive(Serialize, ToSchema)]
pub struct PolicyHealthDashboard {
    pub policies: Vec<PolicyHealthSummary>,
    pub total_policies: i64,
    pub healthy_policies: i64,
    pub degraded_policies: i64,
    pub critical_policies: i64,
    pub overall_health_score: f64,
}

#[derive(Serialize, ToSchema)]
pub struct PolicyHealthSummary {
    pub policy_id: Uuid,
    pub policy_name: String,
    pub policy_type: String,
    pub health_status: String,
    pub success_rate: f64,
    pub error_rate: f64,
    pub total_requests: i64,
    pub avg_latency_ms: Option<f64>,
    pub circuit_breaker_state: Option<String>,
    pub last_updated: DateTime<Utc>,
}

/// Get policy health dashboard (all policies)
#[utoipa::path(
    get,
    path = "/analytics/policy-health",
    tag = "Policy Monitoring",
    responses(
        (status = 200, description = "Policy health dashboard", body = PolicyHealthDashboard),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_policy_health_dashboard(
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "analytics", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let now = chrono::Utc::now();
    let window_start = now - chrono::Duration::minutes(10);

    #[derive(sqlx::FromRow)]
    struct PolicyRow {
        id: Uuid,
        policy_name: String,
        policy_type: String,
        circuit_breaker_state: Option<String>,
    }

    let policies: Vec<PolicyRow> = sqlx::query_as(
        "SELECT id, policy_name, policy_type, circuit_breaker_state
         FROM policy_versions
         WHERE is_active = true
         ORDER BY policy_name"
    )
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let total_policies_count = policies.len() as i64;
    let mut policy_summaries = Vec::new();
    let mut healthy_count = 0;
    let mut degraded_count = 0;
    let mut critical_count = 0;
    let mut total_health_score = 0.0;

    for policy in policies {
        // Get metrics for this policy
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            total_requests: Option<i64>,
            successful_requests: Option<i64>,
            failed_requests: Option<i64>,
            blocked_requests: Option<i64>,
        }

        let metrics: MetricsRow = sqlx::query_as(
            "SELECT 
                SUM(total_requests) as total_requests,
                SUM(successful_requests) as successful_requests,
                SUM(failed_requests) as failed_requests,
                SUM(blocked_requests) as blocked_requests
             FROM canary_metrics
             WHERE policy_version_id = $1
               AND window_start >= $2"
        )
        .bind(policy.id)
        .bind(window_start)
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or(MetricsRow {
            total_requests: Some(0),
            successful_requests: Some(0),
            failed_requests: Some(0),
            blocked_requests: Some(0),
        });

        let total = metrics.total_requests.unwrap_or(0);
        let successful = metrics.successful_requests.unwrap_or(0);
        let failed = metrics.failed_requests.unwrap_or(0);
        let blocked = metrics.blocked_requests.unwrap_or(0);

        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            100.0
        };

        let error_rate = if total > 0 {
            ((failed + blocked) as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        // Get latency
        let avg_latency: Option<f64> = sqlx::query_scalar(
            "SELECT AVG(EXTRACT(EPOCH FROM (completed_at - timestamp)) * 1000)
             FROM compliance_records
             WHERE policy_version_id = $1
               AND timestamp >= $2
               AND completed_at IS NOT NULL"
        )
        .bind(policy.id)
        .bind(window_start)
        .fetch_optional(&data.db_pool)
        .await
        .ok()
        .flatten();

        // Determine health status
        let health_status = if error_rate >= 10.0 || policy.circuit_breaker_state.as_deref() == Some("OPEN") {
            critical_count += 1;
            "CRITICAL"
        } else if error_rate >= 5.0 {
            degraded_count += 1;
            "DEGRADED"
        } else {
            healthy_count += 1;
            "HEALTHY"
        };

        // Trigger webhook alerts for degraded/critical policies
        if health_status == "CRITICAL" || health_status == "DEGRADED" {
            let webhook_data = serde_json::json!({
                "policy_id": policy.id,
                "policy_name": policy.policy_name,
                "policy_type": policy.policy_type,
                "health_status": health_status,
                "success_rate": success_rate,
                "error_rate": error_rate,
                "total_requests": total,
                "avg_latency_ms": avg_latency,
                "circuit_breaker_state": policy.circuit_breaker_state,
            });
            
            let event_type = if health_status == "CRITICAL" {
                "policy.health.critical"
            } else {
                "policy.health.degraded"
            };
            
            trigger_webhook_event(&data.db_pool, event_type, webhook_data).await;
        }

        total_health_score += success_rate;

        policy_summaries.push(PolicyHealthSummary {
            policy_id: policy.id,
            policy_name: policy.policy_name,
            policy_type: policy.policy_type,
            health_status: health_status.to_string(),
            success_rate,
            error_rate,
            total_requests: total,
            avg_latency_ms: avg_latency,
            circuit_breaker_state: policy.circuit_breaker_state,
            last_updated: now,
        });
    }

    let overall_health_score = if !policy_summaries.is_empty() {
        total_health_score / policy_summaries.len() as f64
    } else {
        100.0
    };

    HttpResponse::Ok().json(PolicyHealthDashboard {
        policies: policy_summaries,
        total_policies: total_policies_count,
        healthy_policies: healthy_count,
        degraded_policies: degraded_count,
        critical_policies: critical_count,
        overall_health_score,
    })
}

// ========== POLICY HEALTH TRENDS ==========

#[derive(Serialize, ToSchema)]
pub struct PolicyHealthTrends {
    pub policy_id: Uuid,
    pub policy_name: String,
    pub trends: Vec<HealthTrendPoint>,
}

#[derive(Serialize, ToSchema)]
pub struct HealthTrendPoint {
    pub timestamp: DateTime<Utc>,
    pub success_rate: f64,
    pub error_rate: f64,
    pub total_requests: i64,
    pub avg_latency_ms: Option<f64>,
    pub health_status: String,
}

/// Get policy health trends (historical data)
#[utoipa::path(
    get,
    path = "/analytics/policy-health/{policy_id}/trends",
    tag = "Policy Monitoring",
    params(
        ("policy_id" = Uuid, Path, description = "Policy ID"),
        ("time_range" = Option<i64>, Query, description = "Time range in hours (default: 24)"),
    ),
    responses(
        (status = 200, description = "Policy health trends", body = PolicyHealthTrends),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_policy_health_trends(
    policy_id: web::Path<Uuid>,
    query: web::Query<HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "analytics", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let policy_id = policy_id.into_inner();
    let time_range_hours = query.get("time_range")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(24);

    let window_start = Utc::now() - chrono::Duration::hours(time_range_hours);

    // Get policy name
    let policy_name: Option<String> = sqlx::query_scalar(
        "SELECT policy_name FROM policy_versions WHERE id = $1"
    )
    .bind(policy_id)
    .fetch_optional(&data.db_pool)
    .await
    .ok()
    .flatten();

    if policy_name.is_none() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "POLICY_NOT_FOUND"
        }));
    }

    // Get hourly trends
    #[derive(sqlx::FromRow)]
    struct TrendRow {
        hour_bucket: DateTime<Utc>,
        total_requests: i64,
        successful_requests: i64,
        failed_requests: i64,
        blocked_requests: i64,
        avg_latency: Option<f64>,
    }

    let trends_data: Vec<TrendRow> = sqlx::query_as(
        "SELECT 
            date_trunc('hour', window_start) as hour_bucket,
            SUM(total_requests) as total_requests,
            SUM(successful_requests) as successful_requests,
            SUM(failed_requests) as failed_requests,
            SUM(blocked_requests) as blocked_requests,
            AVG(avg_latency_ms) as avg_latency
         FROM canary_metrics
         WHERE policy_version_id = $1
           AND window_start >= $2
         GROUP BY hour_bucket
         ORDER BY hour_bucket ASC"
    )
    .bind(policy_id)
    .bind(window_start)
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let trends: Vec<HealthTrendPoint> = trends_data.into_iter().map(|row| {
        let total = row.total_requests;
        let successful = row.successful_requests;
        let failed = row.failed_requests;
        let blocked = row.blocked_requests;

        let success_rate = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            100.0
        };

        let error_rate = if total > 0 {
            ((failed + blocked) as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let health_status = if error_rate >= 10.0 {
            "CRITICAL"
        } else if error_rate >= 5.0 {
            "DEGRADED"
        } else {
            "HEALTHY"
        };

        HealthTrendPoint {
            timestamp: row.hour_bucket,
            success_rate,
            error_rate,
            total_requests: total,
            avg_latency_ms: row.avg_latency,
            health_status: health_status.to_string(),
        }
    }).collect();

    HttpResponse::Ok().json(PolicyHealthTrends {
        policy_id,
        policy_name: policy_name.unwrap(),
        trends,
    })
}

// ========== APPROVAL QUEUE DASHBOARD ==========

#[derive(Serialize, ToSchema)]
pub struct ApprovalQueueDashboard {
    pub pending_approvals: Vec<PendingApproval>,
    pub approved_policies: Vec<ApprovedPolicy>,
    pub rejected_policies: Vec<RejectedPolicy>,
    pub total_pending: i64,
    pub total_approved: i64,
    pub total_rejected: i64,
}

#[derive(Serialize, ToSchema)]
pub struct PendingApproval {
    pub policy_id: Uuid,
    pub policy_name: String,
    pub policy_type: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub approval_count: i64,
    pub required_count: i32,
    pub remaining_approvals: i32,
    pub approvers: Vec<ApproverInfo>,
    pub policy_config: Option<serde_json::Value>,
    pub impact_analysis: Option<serde_json::Value>,
}

#[derive(Serialize, ToSchema)]
pub struct ApprovedPolicy {
    pub policy_id: Uuid,
    pub policy_name: String,
    pub approved_at: DateTime<Utc>,
    pub approvers: Vec<ApproverInfo>,
    pub can_activate: bool,
}

#[derive(Serialize, ToSchema)]
pub struct RejectedPolicy {
    pub policy_id: Uuid,
    pub policy_name: String,
    pub rejected_at: DateTime<Utc>,
    pub rejected_by: String,
    pub rejection_reason: String,
}

#[derive(Serialize, ToSchema)]
pub struct ApproverInfo {
    pub approver_id: String,
    pub action: String, // APPROVED, REJECTED, PENDING
    pub approved_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

/// Get approval queue dashboard
#[utoipa::path(
    get,
    path = "/approvals/queue",
    tag = "Policy Approvals",
    responses(
        (status = 200, description = "Approval queue dashboard", body = ApprovalQueueDashboard),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_approval_queue(
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "policy", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    // Get pending approvals
    #[derive(sqlx::FromRow)]
    struct PendingPolicyRow {
        id: Uuid,
        policy_name: String,
        policy_type: String,
        created_at: DateTime<Utc>,
        created_by: Option<String>,
        approval_required_count: i32,
        policy_config: Option<serde_json::Value>,
    }

    let pending_policies: Vec<PendingPolicyRow> = sqlx::query_as(
        "SELECT id, policy_name, policy_type, created_at, created_by, 
                approval_required_count, policy_config
         FROM policy_versions
         WHERE requires_approval = true 
           AND approval_status = 'PENDING'
         ORDER BY created_at DESC"
    )
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let mut pending_approvals = Vec::new();
    for policy in pending_policies {
        // Get approval count
        let approval_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM policy_approvals 
             WHERE policy_version_id = $1 AND action = 'APPROVED'"
        )
        .bind(policy.id)
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or(0);

        // Get approvers
        #[derive(sqlx::FromRow)]
        struct ApproverRow {
            approver_id: String,
            action: String,
            approved_at: Option<DateTime<Utc>>,
            notes: Option<String>,
        }

        let approvers_data: Vec<ApproverRow> = sqlx::query_as(
            "SELECT approver_id, action, approved_at, notes
             FROM policy_approvals
             WHERE policy_version_id = $1
             ORDER BY approved_at DESC NULLS LAST"
        )
        .bind(policy.id)
        .fetch_all(&data.db_pool)
        .await
        .unwrap_or_default();

        let approvers: Vec<ApproverInfo> = approvers_data.into_iter().map(|a| {
            ApproverInfo {
                approver_id: a.approver_id,
                action: a.action,
                approved_at: a.approved_at,
                notes: a.notes,
            }
        }).collect();

        // Get impact analysis if available
        let impact_analysis: Option<serde_json::Value> = sqlx::query_scalar(
            "SELECT impact_analysis FROM policy_impact_cache WHERE policy_version_id = $1"
        )
        .bind(policy.id)
        .fetch_optional(&data.db_pool)
        .await
        .ok()
        .flatten();

        pending_approvals.push(PendingApproval {
            policy_id: policy.id,
            policy_name: policy.policy_name,
            policy_type: policy.policy_type,
            created_at: policy.created_at,
            created_by: policy.created_by,
            approval_count,
            required_count: policy.approval_required_count,
            remaining_approvals: policy.approval_required_count - approval_count as i32,
            approvers,
            policy_config: policy.policy_config,
            impact_analysis,
        });
    }

    // Get approved policies
    #[derive(sqlx::FromRow)]
    struct ApprovedPolicyRow {
        id: Uuid,
        policy_name: String,
        approval_status: String,
    }

    let approved_policies_data: Vec<ApprovedPolicyRow> = sqlx::query_as(
        "SELECT id, policy_name, approval_status
         FROM policy_versions
         WHERE requires_approval = true 
           AND approval_status = 'APPROVED'
         ORDER BY updated_at DESC
         LIMIT 50"
    )
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let mut approved_policies = Vec::new();
    for policy in approved_policies_data {
        // Get approvers
        #[derive(sqlx::FromRow)]
        struct ApproverRow {
            approver_id: String,
            action: String,
            approved_at: Option<DateTime<Utc>>,
            notes: Option<String>,
        }

        let approvers_data: Vec<ApproverRow> = sqlx::query_as(
            "SELECT approver_id, action, approved_at, notes
             FROM policy_approvals
             WHERE policy_version_id = $1 AND action = 'APPROVED'
             ORDER BY approved_at DESC"
        )
        .bind(policy.id)
        .fetch_all(&data.db_pool)
        .await
        .unwrap_or_default();

        let approvers: Vec<ApproverInfo> = approvers_data.into_iter().map(|a| {
            ApproverInfo {
                approver_id: a.approver_id,
                action: a.action,
                approved_at: a.approved_at,
                notes: a.notes,
            }
        }).collect();

        let approved_at = approvers.first()
            .and_then(|a| a.approved_at)
            .unwrap_or_else(|| Utc::now());

        approved_policies.push(ApprovedPolicy {
            policy_id: policy.id,
            policy_name: policy.policy_name,
            approved_at,
            approvers,
            can_activate: !policy.approval_status.is_empty(),
        });
    }

    // Get rejected policies
    #[derive(sqlx::FromRow)]
    struct RejectedPolicyRow {
        id: Uuid,
        policy_name: String,
        approval_notes: Option<String>,
    }

    let rejected_policies_data: Vec<RejectedPolicyRow> = sqlx::query_as(
        "SELECT id, policy_name, approval_notes
         FROM policy_versions
         WHERE requires_approval = true 
           AND approval_status = 'REJECTED'
         ORDER BY updated_at DESC
         LIMIT 50"
    )
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let mut rejected_policies = Vec::new();
    for policy in rejected_policies_data {
        // Get rejection info
        #[derive(sqlx::FromRow)]
        struct RejectionRow {
            approver_id: String,
            approved_at: Option<DateTime<Utc>>,
            notes: Option<String>,
        }

        let rejection: Option<RejectionRow> = sqlx::query_as(
            "SELECT approver_id, approved_at, notes
             FROM policy_approvals
             WHERE policy_version_id = $1 AND action = 'REJECTED'
             ORDER BY approved_at DESC
             LIMIT 1"
        )
        .bind(policy.id)
        .fetch_optional(&data.db_pool)
        .await
        .ok()
        .flatten();

        if let Some(rej) = rejection {
            rejected_policies.push(RejectedPolicy {
                policy_id: policy.id,
                policy_name: policy.policy_name,
                rejected_at: rej.approved_at.unwrap_or_else(|| Utc::now()),
                rejected_by: rej.approver_id,
                rejection_reason: rej.notes.or(policy.approval_notes).unwrap_or_else(|| "No reason provided".to_string()),
            });
        }
    }

    let total_pending_count = pending_approvals.len() as i64;
    let total_approved_count = approved_policies.len() as i64;
    let total_rejected_count = rejected_policies.len() as i64;

    HttpResponse::Ok().json(ApprovalQueueDashboard {
        pending_approvals,
        approved_policies,
        rejected_policies,
        total_pending: total_pending_count,
        total_approved: total_approved_count,
        total_rejected: total_rejected_count,
    })
}

// ========== APPROVAL HISTORY ==========

#[derive(Serialize, ToSchema)]
pub struct ApprovalHistory {
    pub policy_id: Uuid,
    pub policy_name: String,
    pub history: Vec<ApprovalHistoryEntry>,
}

#[derive(Serialize, ToSchema)]
pub struct ApprovalHistoryEntry {
    pub approver_id: String,
    pub action: String,
    pub timestamp: DateTime<Utc>,
    pub notes: Option<String>,
}

/// Get approval history for a policy
#[utoipa::path(
    get,
    path = "/approvals/{policy_id}/history",
    tag = "Policy Approvals",
    params(
        ("policy_id" = Uuid, Path, description = "Policy ID"),
    ),
    responses(
        (status = 200, description = "Approval history", body = ApprovalHistory),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_approval_history(
    policy_id: web::Path<Uuid>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "policy", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let policy_id = policy_id.into_inner();

    // Get policy name
    let policy_name: Option<String> = sqlx::query_scalar(
        "SELECT policy_name FROM policy_versions WHERE id = $1"
    )
    .bind(policy_id)
    .fetch_optional(&data.db_pool)
    .await
    .ok()
    .flatten();

    if policy_name.is_none() {
        return HttpResponse::NotFound().json(serde_json::json!({
            "error": "POLICY_NOT_FOUND"
        }));
    }

    // Get approval history
    #[derive(sqlx::FromRow)]
    struct HistoryRow {
        approver_id: String,
        action: String,
        approved_at: Option<DateTime<Utc>>,
        notes: Option<String>,
    }

    let history_data: Vec<HistoryRow> = sqlx::query_as(
        "SELECT approver_id, action, approved_at, notes
         FROM policy_approvals
         WHERE policy_version_id = $1
         ORDER BY approved_at DESC NULLS LAST, created_at DESC"
    )
    .bind(policy_id)
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let history: Vec<ApprovalHistoryEntry> = history_data.into_iter().map(|h| {
        ApprovalHistoryEntry {
            approver_id: h.approver_id,
            action: h.action,
            timestamp: h.approved_at.unwrap_or_else(|| Utc::now()),
            notes: h.notes,
        }
    }).collect();

    HttpResponse::Ok().json(ApprovalHistory {
        policy_id,
        policy_name: policy_name.unwrap(),
        history,
    })
}

// ========== ROLLBACK HISTORY DASHBOARD ==========

#[derive(Serialize, ToSchema)]
pub struct RollbackHistoryDashboard {
    pub rollbacks: Vec<RollbackHistoryEntry>,
    pub total_rollbacks: i64,
    pub auto_rollbacks: i64,
    pub manual_rollbacks: i64,
    pub rollback_reasons: HashMap<String, i64>,
}

#[derive(Serialize, ToSchema)]
pub struct RollbackHistoryEntry {
    pub rollback_id: Uuid,
    pub policy_id: Uuid,
    pub policy_name: String,
    pub rollback_type: String, // AUTO, MANUAL
    pub from_version: Option<i32>,
    pub to_version: Option<i32>,
    pub from_percentage: Option<i32>,
    pub to_percentage: Option<i32>,
    pub success_rate: Option<f64>,
    pub error_rate: Option<f64>,
    pub reason: String,
    pub performed_by: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Get rollback history dashboard
#[utoipa::path(
    get,
    path = "/analytics/rollback-history",
    tag = "Policy Management",
    params(
        ("policy_id" = Option<Uuid>, Query, description = "Filter by policy ID"),
        ("time_range" = Option<i64>, Query, description = "Time range in days (default: 30)"),
    ),
    responses(
        (status = 200, description = "Rollback history dashboard", body = RollbackHistoryDashboard),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_rollback_history(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "analytics", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let policy_id_filter: Option<Uuid> = query.get("policy_id")
        .and_then(|s| Uuid::parse_str(s).ok());
    
    let time_range_days = query.get("time_range")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(30);

    let window_start = Utc::now() - chrono::Duration::days(time_range_days);

    // Get canary rollback history
    #[derive(sqlx::FromRow)]
    struct CanaryRollbackRow {
        id: Uuid,
        policy_version_id: Uuid,
        from_percentage: i32,
        to_percentage: i32,
        success_rate: Option<f64>,
        triggered_by: String,
        notes: Option<String>,
        created_at: DateTime<Utc>,
    }

    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT id, policy_version_id, from_percentage, to_percentage, 
                success_rate, triggered_by, notes, created_at
         FROM canary_deployment_history
         WHERE action = 'ROLLED_BACK' AND created_at >= "
    );
    query_builder.push_bind(window_start);

    if let Some(policy_id) = policy_id_filter {
        query_builder.push(" AND policy_version_id = ");
        query_builder.push_bind(policy_id);
    }

    query_builder.push(" ORDER BY created_at DESC");

    let canary_rollbacks: Vec<CanaryRollbackRow> = query_builder
        .build_query_as()
        .fetch_all(&data.db_pool)
        .await
        .unwrap_or_default();

    // Get policy version rollback history
    #[derive(sqlx::FromRow)]
    struct PolicyRollbackRow {
        id: Uuid,
        policy_version_id: Uuid,
        previous_version_id: Option<Uuid>,
        performed_by: Option<String>,
        notes: Option<String>,
        created_at: DateTime<Utc>,
    }

    let mut query_builder2 = sqlx::QueryBuilder::new(
        "SELECT id, policy_version_id, previous_version_id, performed_by, notes, created_at
         FROM policy_activation_history
         WHERE action = 'ROLLED_BACK' AND created_at >= "
    );
    query_builder2.push_bind(window_start);

    if let Some(policy_id) = policy_id_filter {
        query_builder2.push(" AND policy_version_id = ");
        query_builder2.push_bind(policy_id);
    }

    query_builder2.push(" ORDER BY created_at DESC");

    let policy_rollbacks: Vec<PolicyRollbackRow> = query_builder2
        .build_query_as()
        .fetch_all(&data.db_pool)
        .await
        .unwrap_or_default();

    let mut rollback_entries = Vec::new();
    let mut auto_count = 0;
    let mut manual_count = 0;
    let mut reason_counts: HashMap<String, i64> = HashMap::new();

    // Process canary rollbacks (auto-rollbacks)
    for rollback in canary_rollbacks {
        let policy_name: Option<String> = sqlx::query_scalar(
            "SELECT policy_name FROM policy_versions WHERE id = $1"
        )
        .bind(rollback.policy_version_id)
        .fetch_optional(&data.db_pool)
        .await
        .ok()
        .flatten();

        let reason = rollback.notes.unwrap_or_else(|| "Auto-rollback due to high failure rate".to_string());
        *reason_counts.entry(reason.clone()).or_insert(0) += 1;
        auto_count += 1;

        // Get error rate from canary metrics
        let error_rate: Option<f64> = sqlx::query_scalar(
            "SELECT (failed_requests + blocked_requests)::DECIMAL / NULLIF(total_requests, 0) * 100.0
             FROM canary_metrics
             WHERE policy_version_id = $1 AND traffic_percentage = $2
             ORDER BY window_end DESC
             LIMIT 1"
        )
        .bind(rollback.policy_version_id)
        .bind(rollback.from_percentage)
        .fetch_optional(&data.db_pool)
        .await
        .ok()
        .flatten();

        rollback_entries.push(RollbackHistoryEntry {
            rollback_id: rollback.id,
            policy_id: rollback.policy_version_id,
            policy_name: policy_name.unwrap_or_else(|| "Unknown Policy".to_string()),
            rollback_type: "AUTO".to_string(),
            from_version: None,
            to_version: None,
            from_percentage: Some(rollback.from_percentage),
            to_percentage: Some(rollback.to_percentage),
            success_rate: rollback.success_rate,
            error_rate,
            reason,
            performed_by: Some(rollback.triggered_by),
            timestamp: rollback.created_at,
        });
    }

    // Process policy version rollbacks (manual rollbacks)
    for rollback in policy_rollbacks {
        let policy_name: Option<String> = sqlx::query_scalar(
            "SELECT policy_name FROM policy_versions WHERE id = $1"
        )
        .bind(rollback.policy_version_id)
        .fetch_optional(&data.db_pool)
        .await
        .ok()
        .flatten();

        let from_version: Option<i32> = if let Some(prev_id) = rollback.previous_version_id {
            sqlx::query_scalar(
                "SELECT version_number FROM policy_versions WHERE id = $1"
            )
            .bind(prev_id)
            .fetch_optional(&data.db_pool)
            .await
            .ok()
            .flatten()
        } else {
            None
        };

        let to_version: Option<i32> = sqlx::query_scalar(
            "SELECT version_number FROM policy_versions WHERE id = $1"
        )
        .bind(rollback.policy_version_id)
        .fetch_optional(&data.db_pool)
        .await
        .ok()
        .flatten();

        let reason = rollback.notes.unwrap_or_else(|| "Manual rollback".to_string());
        *reason_counts.entry(reason.clone()).or_insert(0) += 1;
        manual_count += 1;

        rollback_entries.push(RollbackHistoryEntry {
            rollback_id: rollback.id,
            policy_id: rollback.policy_version_id,
            policy_name: policy_name.unwrap_or_else(|| "Unknown Policy".to_string()),
            rollback_type: "MANUAL".to_string(),
            from_version,
            to_version,
            from_percentage: None,
            to_percentage: None,
            success_rate: None,
            error_rate: None,
            reason,
            performed_by: rollback.performed_by,
            timestamp: rollback.created_at,
        });
    }

    // Sort by timestamp (most recent first)
    rollback_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    HttpResponse::Ok().json(RollbackHistoryDashboard {
        rollbacks: rollback_entries,
        total_rollbacks: (auto_count + manual_count) as i64,
        auto_rollbacks: auto_count,
        manual_rollbacks: manual_count,
        rollback_reasons: reason_counts,
    })
}

// ========== APPROVAL DELEGATION ==========

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateDelegationRequest {
    #[schema(example = "user-123")]
    pub delegate_id: String,
    #[schema(example = "POLICY")]
    pub resource_type: Option<String>,
    #[schema(example = "2024-12-31T23:59:59Z")]
    pub expires_at: Option<DateTime<Utc>>,
    #[schema(example = "Delegating approval authority while on vacation")]
    pub notes: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct DelegationResponse {
    pub id: Uuid,
    pub delegator_id: String,
    pub delegate_id: String,
    pub resource_type: String,
    pub active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct DelegationListResponse {
    pub delegations: Vec<DelegationResponse>,
    pub total: i64,
}

/// Create an approval delegation
#[utoipa::path(
    post,
    path = "/approvals/delegations",
    tag = "Policy Approvals",
    request_body = CreateDelegationRequest,
    responses(
        (status = 200, description = "Delegation created", body = DelegationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Invalid request"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn create_delegation(
    req: web::Json<CreateDelegationRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let claims = match authenticate_and_authorize(&http_req, &data.db_pool, "policy", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let delegation_req = req.into_inner();

    // Validate delegate exists
    let delegate_exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1::uuid OR username = $1)"
    )
    .bind(&delegation_req.delegate_id)
    .fetch_optional(&data.db_pool)
    .await
    .ok()
    .flatten();

    if !delegate_exists.unwrap_or(false) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "DELEGATE_NOT_FOUND",
            "message": "Delegate user not found"
        }));
    }

    // Check if delegate is different from delegator
    if delegation_req.delegate_id == claims.sub {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "INVALID_DELEGATE",
            "message": "Cannot delegate to yourself"
        }));
    }

    // Check if delegation already exists and is active
    let existing: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM approval_delegations
         WHERE delegator_id = $1 
           AND delegate_id = $2
           AND resource_type = COALESCE($3, 'POLICY')
           AND active = true
           AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)"
    )
    .bind(&claims.sub)
    .bind(&delegation_req.delegate_id)
    .bind(&delegation_req.resource_type)
    .fetch_optional(&data.db_pool)
    .await
    .ok()
    .flatten();

    if existing.is_some() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "DELEGATION_EXISTS",
            "message": "An active delegation already exists for this user"
        }));
    }

    // Create delegation
    let delegation_id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO approval_delegations (delegator_id, delegate_id, resource_type, expires_at, notes)
         VALUES ($1, $2, COALESCE($3, 'POLICY'), $4, $5)
         RETURNING id"
    )
    .bind(&claims.sub)
    .bind(&delegation_req.delegate_id)
    .bind(&delegation_req.resource_type)
    .bind(&delegation_req.expires_at)
    .bind(&delegation_req.notes)
    .fetch_optional(&data.db_pool)
    .await
    .ok()
    .flatten();

    match delegation_id {
        Some(id) => {
            // Get the created delegation
            #[derive(sqlx::FromRow)]
            struct DelegationRow {
                id: Uuid,
                delegator_id: String,
                delegate_id: String,
                resource_type: String,
                active: bool,
                expires_at: Option<DateTime<Utc>>,
                created_at: DateTime<Utc>,
                notes: Option<String>,
            }

            let delegation: Option<DelegationRow> = sqlx::query_as(
                "SELECT id, delegator_id, delegate_id, resource_type, active, expires_at, created_at, notes
                 FROM approval_delegations
                 WHERE id = $1"
            )
            .bind(id)
            .fetch_optional(&data.db_pool)
            .await
            .ok()
            .flatten();

            if let Some(d) = delegation {
                HttpResponse::Ok().json(DelegationResponse {
                    id: d.id,
                    delegator_id: d.delegator_id,
                    delegate_id: d.delegate_id,
                    resource_type: d.resource_type,
                    active: d.active,
                    expires_at: d.expires_at,
                    created_at: d.created_at,
                    notes: d.notes,
                })
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "DELEGATION_CREATION_FAILED"
                }))
            }
        }
        None => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "DELEGATION_CREATION_FAILED",
            "message": "Failed to create delegation"
        }))
    }
}

/// List approval delegations
#[utoipa::path(
    get,
    path = "/approvals/delegations",
    tag = "Policy Approvals",
    params(
        ("type" = Option<String>, Query, description = "Filter by type: 'sent' (delegations I created) or 'received' (delegations I received)"),
    ),
    responses(
        (status = 200, description = "List of delegations", body = DelegationListResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn list_delegations(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let claims = match authenticate_and_authorize(&http_req, &data.db_pool, "policy", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let filter_type = query.get("type").map(|s| s.as_str());

    #[derive(sqlx::FromRow)]
    struct DelegationRow {
        id: Uuid,
        delegator_id: String,
        delegate_id: String,
        resource_type: String,
        active: bool,
        expires_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        notes: Option<String>,
    }

    let delegations: Vec<DelegationRow> = match filter_type {
        Some("sent") => {
            sqlx::query_as(
                "SELECT id, delegator_id, delegate_id, resource_type, active, expires_at, created_at, notes
                 FROM approval_delegations
                 WHERE delegator_id = $1
                 ORDER BY created_at DESC"
            )
            .bind(&claims.sub)
            .fetch_all(&data.db_pool)
            .await
            .unwrap_or_default()
        }
        Some("received") => {
            sqlx::query_as(
                "SELECT id, delegator_id, delegate_id, resource_type, active, expires_at, created_at, notes
                 FROM approval_delegations
                 WHERE delegate_id = $1
                 ORDER BY created_at DESC"
            )
            .bind(&claims.sub)
            .fetch_all(&data.db_pool)
            .await
            .unwrap_or_default()
        }
        _ => {
            sqlx::query_as(
                "SELECT id, delegator_id, delegate_id, resource_type, active, expires_at, created_at, notes
                 FROM approval_delegations
                 WHERE delegator_id = $1 OR delegate_id = $1
                 ORDER BY created_at DESC"
            )
            .bind(&claims.sub)
            .fetch_all(&data.db_pool)
            .await
            .unwrap_or_default()
        }
    };

    let delegations_response: Vec<DelegationResponse> = delegations.into_iter().map(|d| {
        DelegationResponse {
            id: d.id,
            delegator_id: d.delegator_id,
            delegate_id: d.delegate_id,
            resource_type: d.resource_type,
            active: d.active,
            expires_at: d.expires_at,
            created_at: d.created_at,
            notes: d.notes,
        }
    }).collect();

    let total = delegations_response.len() as i64;
    HttpResponse::Ok().json(DelegationListResponse {
        delegations: delegations_response,
        total,
    })
}

/// Revoke an approval delegation
#[utoipa::path(
    delete,
    path = "/approvals/delegations/{delegation_id}",
    tag = "Policy Approvals",
    params(
        ("delegation_id" = Uuid, Path, description = "Delegation ID"),
    ),
    responses(
        (status = 200, description = "Delegation revoked"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Delegation not found"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn revoke_delegation(
    delegation_id: web::Path<Uuid>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let claims = match authenticate_and_authorize(&http_req, &data.db_pool, "policy", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let delegation_id = delegation_id.into_inner();

    // Check if delegation exists and user has permission to revoke it
    let delegation: Option<(String, String)> = sqlx::query_as(
        "SELECT delegator_id, delegate_id
         FROM approval_delegations
         WHERE id = $1 AND active = true"
    )
    .bind(delegation_id)
    .fetch_optional(&data.db_pool)
    .await
    .ok()
    .flatten();

    match delegation {
        Some((delegator_id, delegate_id)) => {
            // Only delegator or delegate can revoke
            if delegator_id != claims.sub && delegate_id != claims.sub {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "PERMISSION_DENIED",
                    "message": "You can only revoke delegations you created or received"
                }));
            }

            // Revoke delegation
            let rows_affected = sqlx::query(
                "UPDATE approval_delegations
                 SET active = false, revoked_at = CURRENT_TIMESTAMP
                 WHERE id = $1"
            )
            .bind(delegation_id)
            .execute(&data.db_pool)
            .await
            .map(|r| r.rows_affected())
            .unwrap_or(0);

            if rows_affected > 0 {
                HttpResponse::Ok().json(serde_json::json!({
                    "status": "SUCCESS",
                    "message": "Delegation revoked successfully"
                }))
            } else {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "DELEGATION_NOT_FOUND"
                }))
            }
        }
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": "DELEGATION_NOT_FOUND",
            "message": "Delegation not found or already revoked"
        }))
    }
}

// ========== TPRM COMPLIANCE REPORTING (DORA Article 9) ==========

#[derive(Serialize, ToSchema)]
pub struct TPRMComplianceReport {
    pub total_vendors: i64,
    pub high_risk_vendors: i64,
    pub critical_risk_vendors: i64,
    pub non_compliant_vendors: i64,
    pub vendors_by_country: HashMap<String, i64>,
    pub vendors_by_risk_level: HashMap<String, i64>,
    pub vendors: Vec<VendorComplianceInfo>,
    pub compliance_score: f64,
    pub dora_article9_compliant: bool,
}

#[derive(Serialize, ToSchema)]
pub struct VendorComplianceInfo {
    pub vendor_domain: String,
    pub vendor_name: Option<String>,
    pub risk_score: f64,
    pub risk_level: String,
    pub compliance_status: String,
    pub country_code: Option<String>,
    pub industry_sector: Option<String>,
    pub associated_assets: Vec<String>,
    pub last_assessed: Option<DateTime<Utc>>,
}

/// Get TPRM compliance report (DORA Article 9 - Third-Party Risk Register)
#[utoipa::path(
    get,
    path = "/reports/tprm-compliance",
    tag = "TPRM Compliance",
    params(
        ("risk_level" = Option<String>, Query, description = "Filter by risk level: LOW, MEDIUM, HIGH, CRITICAL"),
        ("country" = Option<String>, Query, description = "Filter by country code"),
    ),
    responses(
        (status = 200, description = "TPRM compliance report", body = TPRMComplianceReport),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_tprm_compliance_report(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "tprm", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let risk_level_filter = query.get("risk_level");
    let country_filter = query.get("country");

    // Get all vendor risk scores
    #[derive(sqlx::FromRow)]
    struct VendorRiskRow {
        vendor_domain: String,
        vendor_name: Option<String>,
        risk_score: Option<rust_decimal::Decimal>,
        risk_level: Option<String>,
        compliance_status: Option<String>,
        country_code: Option<String>,
        industry_sector: Option<String>,
        last_updated: Option<DateTime<Utc>>,
    }

    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT vendor_domain, vendor_name, risk_score, risk_level, 
                compliance_status, country_code, industry_sector, last_updated
         FROM vendor_risk_scores WHERE 1=1"
    );

    if let Some(rl) = risk_level_filter {
        query_builder.push(" AND risk_level = ");
        query_builder.push_bind(rl);
    }

    if let Some(country) = country_filter {
        query_builder.push(" AND country_code = ");
        query_builder.push_bind(country);
    }

    query_builder.push(" ORDER BY risk_score DESC NULLS LAST");

    let vendors: Vec<VendorRiskRow> = query_builder
        .build_query_as()
        .fetch_all(&data.db_pool)
        .await
        .unwrap_or_default();

    let total_vendors = vendors.len() as i64;
    let high_risk_vendors = vendors.iter().filter(|v| v.risk_level.as_deref() == Some("HIGH")).count() as i64;
    let critical_risk_vendors = vendors.iter().filter(|v| v.risk_level.as_deref() == Some("CRITICAL")).count() as i64;
    let non_compliant_vendors = vendors.iter().filter(|v| v.compliance_status.as_deref() == Some("NON_COMPLIANT")).count() as i64;

    // Group by country
    let mut vendors_by_country: HashMap<String, i64> = HashMap::new();
    for vendor in &vendors {
        if let Some(country) = &vendor.country_code {
            *vendors_by_country.entry(country.clone()).or_insert(0) += 1;
        }
    }

    // Group by risk level
    let mut vendors_by_risk_level: HashMap<String, i64> = HashMap::new();
    for vendor in &vendors {
        if let Some(risk_level) = &vendor.risk_level {
            *vendors_by_risk_level.entry(risk_level.clone()).or_insert(0) += 1;
        }
    }

    // Get associated assets for each vendor
    let mut vendor_info: Vec<VendorComplianceInfo> = Vec::new();
    for vendor in vendors {
        let asset_ids: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT a.asset_id 
             FROM assets a
             JOIN asset_vendor_mapping avm ON avm.asset_id = a.id
             WHERE avm.vendor_domain = $1 AND a.is_active = true"
        )
        .bind(&vendor.vendor_domain)
        .fetch_all(&data.db_pool)
        .await
        .unwrap_or_default();

        vendor_info.push(VendorComplianceInfo {
            vendor_domain: vendor.vendor_domain,
            vendor_name: vendor.vendor_name,
            risk_score: vendor.risk_score.map(|d| {
                use rust_decimal::prelude::ToPrimitive;
                d.to_f64().unwrap_or(0.0)
            }).unwrap_or(0.0),
            risk_level: vendor.risk_level.unwrap_or_else(|| "UNKNOWN".to_string()),
            compliance_status: vendor.compliance_status.unwrap_or_else(|| "UNKNOWN".to_string()),
            country_code: vendor.country_code,
            industry_sector: vendor.industry_sector,
            associated_assets: asset_ids,
            last_assessed: vendor.last_updated,
        });
    }

    // Calculate compliance score (0-100)
    let compliance_score = if total_vendors > 0 {
        let compliant_count = vendor_info.iter()
            .filter(|v| v.compliance_status == "COMPLIANT")
            .count() as f64;
        (compliant_count / total_vendors as f64) * 100.0
    } else {
        100.0 // No vendors = fully compliant
    };

    // DORA Article 9 compliance: Must have risk register with all third parties
    let dora_article9_compliant = total_vendors > 0 && compliance_score >= 80.0;

    HttpResponse::Ok().json(TPRMComplianceReport {
        total_vendors,
        high_risk_vendors,
        critical_risk_vendors,
        non_compliant_vendors,
        vendors_by_country,
        vendors_by_risk_level,
        vendors: vendor_info,
        compliance_score,
        dora_article9_compliant,
    })
}

/// Get vendor risk dashboard data
#[utoipa::path(
    get,
    path = "/analytics/vendor-risk",
    tag = "TPRM Analytics",
    params(
        ("risk_level" = Option<String>, Query, description = "Filter by risk level"),
        ("country" = Option<String>, Query, description = "Filter by country"),
        ("industry" = Option<String>, Query, description = "Filter by industry sector"),
    ),
    responses(
        (status = 200, description = "Vendor risk dashboard data", body = TPRMComplianceReport),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_vendor_risk_dashboard(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // Reuse TPRM compliance report endpoint for dashboard
    get_tprm_compliance_report(query, data, http_req).await
}

// ========== BUSINESS FUNCTION ANALYTICS ==========

#[derive(Serialize, ToSchema)]
pub struct BusinessFunctionDashboard {
    pub business_functions: Vec<BusinessFunctionStats>,
    pub total_assets: i64,
    pub compliant_assets: i64,
    pub non_compliant_assets: i64,
    pub compliance_by_function: HashMap<String, f64>,
}

#[derive(Serialize, ToSchema)]
pub struct BusinessFunctionStats {
    pub business_function: String,
    pub asset_count: i64,
    pub compliant_count: i64,
    pub non_compliant_count: i64,
    pub compliance_score: f64,
    pub high_risk_assets: i64,
    pub critical_assets: i64,
    pub avg_risk_score: f64,
}

/// Get business function dashboard
#[utoipa::path(
    get,
    path = "/analytics/business-functions",
    tag = "Business Analytics",
    params(
        ("business_function" = Option<String>, Query, description = "Filter by business function"),
    ),
    responses(
        (status = 200, description = "Business function dashboard", body = BusinessFunctionDashboard),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_business_function_dashboard(
    query: web::Query<HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "analytics", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let business_function_filter = query.get("business_function");

    #[derive(sqlx::FromRow)]
    struct AssetStatsRow {
        business_function: String,
        asset_count: i64,
        compliant_count: i64,
        non_compliant_count: i64,
        high_risk_count: i64,
        critical_count: i64,
        avg_risk_score: Option<rust_decimal::Decimal>,
    }

    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT 
            business_function,
            COUNT(*) as asset_count,
            COUNT(*) FILTER (WHERE risk_profile = 'LOW' OR risk_profile = 'MEDIUM') as compliant_count,
            COUNT(*) FILTER (WHERE risk_profile = 'HIGH' OR risk_profile = 'CRITICAL') as non_compliant_count,
            COUNT(*) FILTER (WHERE risk_profile = 'HIGH') as high_risk_count,
            COUNT(*) FILTER (WHERE risk_profile = 'CRITICAL') as critical_count,
            AVG(CASE 
                WHEN risk_profile = 'LOW' THEN 20.0
                WHEN risk_profile = 'MEDIUM' THEN 50.0
                WHEN risk_profile = 'HIGH' THEN 75.0
                WHEN risk_profile = 'CRITICAL' THEN 95.0
                ELSE 50.0
            END) as avg_risk_score
         FROM assets
         WHERE is_active = true"
    );

    if let Some(bf) = business_function_filter {
        query_builder.push(" AND business_function = ");
        query_builder.push_bind(bf);
    }

    query_builder.push(" GROUP BY business_function ORDER BY asset_count DESC");

    let stats: Vec<AssetStatsRow> = query_builder
        .build_query_as()
        .fetch_all(&data.db_pool)
        .await
        .unwrap_or_default();

    let total_assets: i64 = stats.iter().map(|s| s.asset_count).sum();
    let compliant_assets: i64 = stats.iter().map(|s| s.compliant_count).sum();
    let non_compliant_assets: i64 = stats.iter().map(|s| s.non_compliant_count).sum();

    let mut business_functions = Vec::new();
    let mut compliance_by_function: HashMap<String, f64> = HashMap::new();

    for stat in stats {
        let compliance_score = if stat.asset_count > 0 {
            (stat.compliant_count as f64 / stat.asset_count as f64) * 100.0
        } else {
            100.0
        };

        compliance_by_function.insert(stat.business_function.clone(), compliance_score);

        business_functions.push(BusinessFunctionStats {
            business_function: stat.business_function,
            asset_count: stat.asset_count,
            compliant_count: stat.compliant_count,
            non_compliant_count: stat.non_compliant_count,
            compliance_score,
            high_risk_assets: stat.high_risk_count,
            critical_assets: stat.critical_count,
            avg_risk_score: stat.avg_risk_score.map(|d| {
                use rust_decimal::prelude::ToPrimitive;
                d.to_f64().unwrap_or(50.0)
            }).unwrap_or(50.0),
        });
    }

    HttpResponse::Ok().json(BusinessFunctionDashboard {
        business_functions,
        total_assets,
        compliant_assets,
        non_compliant_assets,
        compliance_by_function,
    })
}

// ========== DORA COMPLIANCE REPORTING ==========

#[derive(Serialize, ToSchema)]
pub struct DORAComplianceReport {
    pub overall_score: f64,
    pub article9_compliant: bool, // ICT third-party risk register
    pub article10_compliant: bool, // Incident reporting
    pub article11_compliant: bool, // Operational resilience testing
    pub article9_score: f64,
    pub article10_score: f64,
    pub article11_score: f64,
    pub third_party_risk_register: TPRMComplianceReport,
    pub incident_reporting: DORAIncidentReporting,
    pub resilience_testing: DORAResilienceTesting,
    pub recommendations: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct DORAIncidentReporting {
    pub total_incidents: i64,
    pub incidents_reported_within_72h: i64,
    pub incidents_pending_report: i64,
    pub average_reporting_time_hours: f64,
    pub compliance_rate: f64,
    pub recent_incidents: Vec<DORAIncident>,
}

#[derive(Serialize, ToSchema)]
pub struct DORAIncident {
    pub incident_id: String,
    pub incident_type: String,
    pub detected_at: DateTime<Utc>,
    pub reported_at: Option<DateTime<Utc>>,
    pub reporting_time_hours: Option<f64>,
    pub within_72h: bool,
    pub severity: String,
}

#[derive(Serialize, ToSchema)]
pub struct DORAResilienceTesting {
    pub last_test_date: Option<DateTime<Utc>>,
    pub tests_conducted_this_year: i64,
    pub tests_passed: i64,
    pub tests_failed: i64,
    pub compliance_rate: f64,
    pub next_test_due: Option<DateTime<Utc>>,
    pub test_results: Vec<DORATestResult>,
}

#[derive(Serialize, ToSchema)]
pub struct DORATestResult {
    pub test_id: String,
    pub test_type: String,
    pub test_date: DateTime<Utc>,
    pub passed: bool,
    pub score: f64,
    pub notes: Option<String>,
}

/// Get DORA compliance report
#[utoipa::path(
    get,
    path = "/reports/dora-compliance",
    tag = "DORA Compliance",
    responses(
        (status = 200, description = "DORA compliance report", body = DORAComplianceReport),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_dora_compliance_report(
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "reports", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    // Get TPRM compliance (Article 9) - Query directly
    #[derive(sqlx::FromRow)]
    struct VendorRiskRow {
        vendor_domain: String,
        vendor_name: Option<String>,
        risk_score: Option<rust_decimal::Decimal>,
        risk_level: Option<String>,
        compliance_status: Option<String>,
        country_code: Option<String>,
        industry_sector: Option<String>,
        last_updated: Option<DateTime<Utc>>,
    }

    let vendors: Vec<VendorRiskRow> = sqlx::query_as(
        "SELECT vendor_domain, vendor_name, risk_score, risk_level, 
                compliance_status, country_code, industry_sector, last_updated
         FROM vendor_risk_scores
         ORDER BY risk_score DESC NULLS LAST"
    )
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let total_vendors = vendors.len() as i64;
    let compliant_vendors = vendors.iter()
        .filter(|v| v.compliance_status.as_deref() == Some("COMPLIANT"))
        .count() as i64;

    let article9_score = if total_vendors > 0 {
        (compliant_vendors as f64 / total_vendors as f64) * 100.0
    } else {
        100.0
    };

    let article9_compliant = total_vendors > 0 && article9_score >= 80.0;

    // Get incident reporting data (Article 10)
    #[derive(sqlx::FromRow)]
    struct IncidentRow {
        breach_id: String,
        breach_type: String,
        detected_at: DateTime<Utc>,
        reported_to_authority_at: Option<DateTime<Utc>>,
        severity: String,
    }

    let incidents: Vec<IncidentRow> = sqlx::query_as(
        "SELECT breach_id, breach_type, detected_at, reported_to_authority_at, severity
         FROM data_breaches
         WHERE detected_at >= CURRENT_TIMESTAMP - INTERVAL '1 year'
         ORDER BY detected_at DESC"
    )
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let total_incidents = incidents.len() as i64;
    let mut incidents_reported_within_72h = 0;
    let mut total_reporting_time = 0.0;
    let mut reported_count = 0;

    let mut dora_incidents = Vec::new();
    for incident in incidents {
        let reporting_time = if let Some(reported_at) = incident.reported_to_authority_at {
            let duration = reported_at.signed_duration_since(incident.detected_at);
            let hours = duration.num_hours() as f64;
            total_reporting_time += hours;
            reported_count += 1;
            if hours <= 72.0 {
                incidents_reported_within_72h += 1;
            }
            Some(hours)
        } else {
            None
        };

        dora_incidents.push(DORAIncident {
            incident_id: incident.breach_id,
            incident_type: incident.breach_type,
            detected_at: incident.detected_at,
            reported_at: incident.reported_to_authority_at,
            reporting_time_hours: reporting_time,
            within_72h: reporting_time.map(|h| h <= 72.0).unwrap_or(false),
            severity: incident.severity,
        });
    }

    let average_reporting_time = if reported_count > 0 {
        total_reporting_time / reported_count as f64
    } else {
        0.0
    };

    let article10_compliance_rate = if total_incidents > 0 {
        (incidents_reported_within_72h as f64 / total_incidents as f64) * 100.0
    } else {
        100.0
    };

    let article10_score = article10_compliance_rate;
    let article10_compliant = article10_compliance_rate >= 100.0;

    // Get resilience testing data (Article 11)
    #[derive(sqlx::FromRow)]
    struct TestRow {
        test_id: String,
        test_type: String,
        test_date: DateTime<Utc>,
        passed: bool,
        score: Option<rust_decimal::Decimal>,
        notes: Option<String>,
    }

    let tests: Vec<TestRow> = sqlx::query_as(
        "SELECT test_id, test_type, test_date, passed, score, notes
         FROM resilience_tests
         WHERE test_date >= CURRENT_TIMESTAMP - INTERVAL '1 year'
         ORDER BY test_date DESC"
    )
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let tests_this_year = tests.len() as i64;
    let tests_passed = tests.iter().filter(|t| t.passed).count() as i64;
    let tests_failed = tests_this_year - tests_passed;

    let last_test = tests.first();
    let last_test_date = last_test.map(|t| t.test_date);

    // Calculate next test due (should be within 1 year)
    let next_test_due = last_test_date.map(|d| d + chrono::Duration::days(365));

    let article11_compliance_rate = if tests_this_year > 0 {
        (tests_passed as f64 / tests_this_year as f64) * 100.0
    } else {
        0.0 // No tests = non-compliant
    };

    let article11_score = if tests_this_year >= 1 {
        article11_compliance_rate
    } else {
        0.0
    };

    let article11_compliant = tests_this_year >= 1 && article11_compliance_rate >= 80.0;

    let test_results: Vec<DORATestResult> = tests.into_iter().map(|t| {
        DORATestResult {
            test_id: t.test_id,
            test_type: t.test_type,
            test_date: t.test_date,
            passed: t.passed,
            score: t.score.map(|d| {
                use rust_decimal::prelude::ToPrimitive;
                d.to_f64().unwrap_or(0.0)
            }).unwrap_or(if t.passed { 100.0 } else { 0.0 }),
            notes: t.notes,
        }
    }).collect();

    // Calculate overall DORA compliance score
    let overall_score = (article9_score + article10_score + article11_score) / 3.0;

    // Generate recommendations
    let mut recommendations = Vec::new();
    if !article9_compliant {
        recommendations.push(format!(
            "Improve third-party risk register compliance from {:.1}% to 80%+ (DORA Article 9)",
            article9_score
        ));
    }
    if !article10_compliant {
        recommendations.push(format!(
            "Ensure all incidents are reported within 72 hours. Current compliance: {:.1}% (DORA Article 10)",
            article10_compliance_rate
        ));
    }
    if !article11_compliant {
        if tests_this_year == 0 {
            recommendations.push("Conduct operational resilience testing (DORA Article 11)".to_string());
        } else {
            recommendations.push(format!(
                "Improve resilience testing pass rate from {:.1}% to 80%+ (DORA Article 11)",
                article11_compliance_rate
            ));
        }
    }

    // Build TPRM report for response
    let vendors_by_country: HashMap<String, i64> = vendors.iter()
        .filter_map(|v| v.country_code.as_ref().map(|c| (c.clone(), 1)))
        .fold(HashMap::new(), |mut acc, (country, _)| {
            *acc.entry(country).or_insert(0) += 1;
            acc
        });

    let vendors_by_risk_level: HashMap<String, i64> = vendors.iter()
        .filter_map(|v| v.risk_level.as_ref().map(|r| (r.clone(), 1)))
        .fold(HashMap::new(), |mut acc, (level, _)| {
            *acc.entry(level).or_insert(0) += 1;
            acc
        });

    let vendor_info: Vec<VendorComplianceInfo> = vendors.into_iter().map(|v| {
        VendorComplianceInfo {
            vendor_domain: v.vendor_domain,
            vendor_name: v.vendor_name,
            risk_score: v.risk_score.map(|d| {
                use rust_decimal::prelude::ToPrimitive;
                d.to_f64().unwrap_or(0.0)
            }).unwrap_or(0.0),
            risk_level: v.risk_level.unwrap_or_else(|| "UNKNOWN".to_string()),
            compliance_status: v.compliance_status.unwrap_or_else(|| "UNKNOWN".to_string()),
            country_code: v.country_code,
            industry_sector: v.industry_sector,
            associated_assets: Vec::new(), // Simplified for DORA report
            last_assessed: v.last_updated,
        }
    }).collect();

    let tprm_report = TPRMComplianceReport {
        total_vendors,
        high_risk_vendors: vendors_by_risk_level.get("HIGH").copied().unwrap_or(0),
        critical_risk_vendors: vendors_by_risk_level.get("CRITICAL").copied().unwrap_or(0),
        non_compliant_vendors: vendors_by_risk_level.get("NON_COMPLIANT").copied().unwrap_or(0),
        vendors_by_country,
        vendors_by_risk_level,
        vendors: vendor_info,
        compliance_score: article9_score,
        dora_article9_compliant: article9_compliant,
    };

    HttpResponse::Ok().json(DORAComplianceReport {
        overall_score,
        article9_compliant,
        article10_compliant,
        article11_compliant,
        article9_score,
        article10_score,
        article11_score,
        third_party_risk_register: tprm_report,
        incident_reporting: DORAIncidentReporting {
            total_incidents,
            incidents_reported_within_72h,
            incidents_pending_report: total_incidents - incidents_reported_within_72h,
            average_reporting_time_hours: average_reporting_time,
            compliance_rate: article10_compliance_rate,
            recent_incidents: dora_incidents,
        },
        resilience_testing: DORAResilienceTesting {
            last_test_date,
            tests_conducted_this_year: tests_this_year,
            tests_passed,
            tests_failed,
            compliance_rate: article11_compliance_rate,
            next_test_due,
            test_results,
        },
        recommendations,
    })
}

// ========== NIS2 COMPLIANCE REPORTING ==========

#[derive(Serialize, ToSchema)]
pub struct NIS2ComplianceReport {
    pub overall_score: f64,
    pub article20_compliant: bool, // Management body accountability
    pub article21_compliant: bool, // Baseline cybersecurity measures
    pub article23_compliant: bool, // Incident reporting
    pub article20_score: f64,
    pub article21_score: f64,
    pub article23_score: f64,
    pub management_accountability: NIS2ManagementAccountability,
    pub baseline_measures: NIS2BaselineMeasures,
    pub incident_reporting: NIS2IncidentReporting,
    pub liability_protection_status: String, // PROTECTED, AT_RISK, EXPOSED
    pub recommendations: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct NIS2ManagementAccountability {
    pub management_body_identified: bool,
    pub accountability_framework_established: bool,
    pub training_completed: bool,
    pub compliance_score: f64,
    pub last_assessment_date: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct NIS2BaselineMeasures {
    pub measures_implemented: Vec<NIS2Measure>,
    pub total_measures: i32,
    pub implemented_count: i32,
    pub compliance_rate: f64,
}

#[derive(Serialize, ToSchema)]
pub struct NIS2Measure {
    pub measure_id: String,
    pub measure_name: String,
    pub article_reference: String,
    pub implemented: bool,
    pub evidence: Option<String>,
    pub last_verified: Option<DateTime<Utc>>,
}

#[derive(Serialize, ToSchema)]
pub struct NIS2IncidentReporting {
    pub total_incidents: i64,
    pub incidents_reported: i64,
    pub incidents_pending: i64,
    pub average_reporting_time_hours: f64,
    pub compliance_rate: f64,
    pub early_warning_indicators: Vec<String>,
}

/// Get NIS2 compliance report
#[utoipa::path(
    get,
    path = "/reports/nis2-compliance",
    tag = "NIS2 Compliance",
    responses(
        (status = 200, description = "NIS2 compliance report", body = NIS2ComplianceReport),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_nis2_compliance_report(
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "reports", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    // Article 20: Management Body Accountability
    // Check if management accountability framework exists
    let management_accountability = {
        // Check for executive assurance scorecard (indicates management engagement)
        let has_executive_dashboard = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = 'executive_scorecards')"
        )
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or(false);

        // Check for compliance KPIs (indicates accountability framework)
        let has_kpis = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM compliance_kpis WHERE is_active = true"
        )
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or(0);

        let management_body_identified = has_executive_dashboard || has_kpis > 0;
        let accountability_framework_established = has_executive_dashboard;
        
        // Check for training records
        let training_completed = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM training_records WHERE training_type = 'NIS2_MANAGEMENT' AND completed_at IS NOT NULL"
        )
        .fetch_one(&data.db_pool)
        .await
        .unwrap_or(0) > 0;

        let compliance_score = {
            let mut score = 0.0;
            if management_body_identified { score += 33.3; }
            if accountability_framework_established { score += 33.3; }
            if training_completed { score += 33.4; }
            score
        };

        let last_assessment = sqlx::query_scalar::<_, Option<DateTime<Utc>>>(
            "SELECT MAX(created_at) FROM executive_scorecards"
        )
        .fetch_one(&data.db_pool)
        .await
        .ok()
        .flatten();

        NIS2ManagementAccountability {
            management_body_identified,
            accountability_framework_established,
            training_completed,
            compliance_score,
            last_assessment_date: last_assessment,
        }
    };

    let article20_score = management_accountability.compliance_score;
    let article20_compliant = article20_score >= 80.0;

    // Article 21: Baseline Cybersecurity Measures
    // NIS2 requires 10 minimum cybersecurity measures
    let baseline_measures_list = vec![
        ("POLICY", "Security policies and procedures", "Article 21.1"),
        ("RISK_ASSESSMENT", "Risk analysis and information system security policies", "Article 21.2"),
        ("INCIDENT_HANDLING", "Incident handling", "Article 21.3"),
        ("BUSINESS_CONTINUITY", "Business continuity and crisis management", "Article 21.4"),
        ("SUPPLY_CHAIN", "Supply chain security", "Article 21.5"),
        ("BASIC_HYGIENE", "Security in network and information systems", "Article 21.6"),
        ("PERSONNEL", "Policies and procedures regarding the security of network and information systems", "Article 21.7"),
        ("ENCRYPTION", "Use of cryptography and encryption", "Article 21.8"),
        ("ACCESS_CONTROL", "Access control and asset management", "Article 21.9"),
        ("MONITORING", "Continuous monitoring and vulnerability management", "Article 21.10"),
    ];

    let mut measures_implemented = Vec::new();
    for (measure_id, measure_name, article_ref) in baseline_measures_list {
        // Check if measure is implemented based on existing features
        let implemented = match measure_id {
            "POLICY" => {
                // Check for active policies
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM asset_policies WHERE is_active = true"
                )
                .fetch_one(&data.db_pool)
                .await
                .unwrap_or(0) > 0
            },
            "RISK_ASSESSMENT" => {
                // Check for risk assessments
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM risk_assessments WHERE assessed_at > CURRENT_TIMESTAMP - INTERVAL '1 year'"
                )
                .fetch_one(&data.db_pool)
                .await
                .unwrap_or(0) > 0
            },
            "INCIDENT_HANDLING" => {
                // Check for incident handling (data breaches)
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM data_breaches"
                )
                .fetch_one(&data.db_pool)
                .await
                .unwrap_or(0) > 0
            },
            "BUSINESS_CONTINUITY" => {
                // Check for resilience tests
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM resilience_tests"
                )
                .fetch_one(&data.db_pool)
                .await
                .unwrap_or(0) > 0
            },
            "SUPPLY_CHAIN" => {
                // Check for TPRM (vendor risk management)
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM vendor_risk_scores"
                )
                .fetch_one(&data.db_pool)
                .await
                .unwrap_or(0) > 0
            },
            "BASIC_HYGIENE" => {
                // Check for compliance records (indicates security monitoring)
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM compliance_records WHERE timestamp > CURRENT_TIMESTAMP - INTERVAL '30 days'"
                )
                .fetch_one(&data.db_pool)
                .await
                .unwrap_or(0) > 0
            },
            "PERSONNEL" => {
                // Check for user management
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM users"
                )
                .fetch_one(&data.db_pool)
                .await
                .unwrap_or(0) > 0
            },
            "ENCRYPTION" => {
                // Check for crypto-shredder (indicates encryption)
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM crypto_shredder_keys"
                )
                .fetch_one(&data.db_pool)
                .await
                .unwrap_or(0) > 0
            },
            "ACCESS_CONTROL" => {
                // Check for API keys (indicates access control)
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM api_keys WHERE is_active = true"
                )
                .fetch_one(&data.db_pool)
                .await
                .unwrap_or(0) > 0
            },
            "MONITORING" => {
                // Check for monitoring (compliance records, webhooks)
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM webhooks WHERE is_active = true"
                )
                .fetch_one(&data.db_pool)
                .await
                .unwrap_or(0) > 0
            },
            _ => false,
        };

        measures_implemented.push(NIS2Measure {
            measure_id: measure_id.to_string(),
            measure_name: measure_name.to_string(),
            article_reference: article_ref.to_string(),
            implemented,
            evidence: if implemented {
                Some(format!("Verified through {} feature", measure_id))
            } else {
                None
            },
            last_verified: if implemented {
                Some(Utc::now())
            } else {
                None
            },
        });
    }

    let implemented_count = measures_implemented.iter().filter(|m| m.implemented).count() as i32;
    let total_measures = measures_implemented.len() as i32;
    let article21_score = (implemented_count as f64 / total_measures as f64) * 100.0;
    let article21_compliant = article21_score >= 80.0;

    // Article 23: Incident Reporting
    // Get incident reporting data (similar to DORA Article 10)
    #[derive(sqlx::FromRow)]
    struct IncidentRow {
        breach_id: String,
        detected_at: DateTime<Utc>,
        reported_to_authority_at: Option<DateTime<Utc>>,
    }

    let incidents: Vec<IncidentRow> = sqlx::query_as(
        "SELECT breach_id, detected_at, reported_to_authority_at
         FROM data_breaches
         WHERE detected_at >= CURRENT_TIMESTAMP - INTERVAL '1 year'
         ORDER BY detected_at DESC"
    )
    .fetch_all(&data.db_pool)
    .await
    .unwrap_or_default();

    let total_incidents = incidents.len() as i64;
    let incidents_reported = incidents.iter().filter(|i| i.reported_to_authority_at.is_some()).count() as i64;
    let incidents_pending = total_incidents - incidents_reported;

    let mut total_reporting_time = 0.0;
    let mut reported_count = 0;
    for incident in &incidents {
        if let Some(reported_at) = incident.reported_to_authority_at {
            let duration = reported_at.signed_duration_since(incident.detected_at);
            total_reporting_time += duration.num_hours() as f64;
            reported_count += 1;
        }
    }

    let average_reporting_time = if reported_count > 0 {
        total_reporting_time / reported_count as f64
    } else {
        0.0
    };

    let article23_compliance_rate = if total_incidents > 0 {
        (incidents_reported as f64 / total_incidents as f64) * 100.0
    } else {
        100.0
    };

    let article23_score = article23_compliance_rate;
    let article23_compliant = article23_compliance_rate >= 100.0;

    // Early warning indicators
    let mut early_warning_indicators = Vec::new();
    if total_incidents > 0 && incidents_pending > 0 {
        early_warning_indicators.push(format!("{} incident(s) pending report", incidents_pending));
    }
    if average_reporting_time > 24.0 {
        early_warning_indicators.push(format!("Average reporting time is {:.1} hours (target: <24h)", average_reporting_time));
    }
    if article21_score < 80.0 {
        early_warning_indicators.push(format!("Only {:.0}% of baseline measures implemented", article21_score));
    }

    // Calculate overall NIS2 compliance score
    let overall_score = (article20_score + article21_score + article23_score) / 3.0;

    // Determine liability protection status
    let liability_protection_status = if overall_score >= 90.0 {
        "PROTECTED"
    } else if overall_score >= 70.0 {
        "AT_RISK"
    } else {
        "EXPOSED"
    }.to_string();

    // Generate recommendations
    let mut recommendations = Vec::new();
    if !article20_compliant {
        recommendations.push(format!(
            "Establish management accountability framework. Current score: {:.1}% (NIS2 Article 20)",
            article20_score
        ));
    }
    if !article21_compliant {
        recommendations.push(format!(
            "Implement remaining baseline cybersecurity measures. {}/{} measures implemented (NIS2 Article 21)",
            implemented_count, total_measures
        ));
    }
    if !article23_compliant {
        recommendations.push(format!(
            "Ensure all incidents are reported. Current compliance: {:.1}% (NIS2 Article 23)",
            article23_compliance_rate
        ));
    }
    if overall_score < 90.0 {
        recommendations.push(format!(
            "Improve overall NIS2 compliance from {:.1}% to 90%+ to achieve PROTECTED liability status",
            overall_score
        ));
    }

    HttpResponse::Ok().json(NIS2ComplianceReport {
        overall_score,
        article20_compliant,
        article21_compliant,
        article23_compliant,
        article20_score,
        article21_score,
        article23_score,
        management_accountability,
        baseline_measures: NIS2BaselineMeasures {
            measures_implemented,
            total_measures,
            implemented_count,
            compliance_rate: article21_score,
        },
        incident_reporting: NIS2IncidentReporting {
            total_incidents,
            incidents_reported,
            incidents_pending,
            average_reporting_time_hours: average_reporting_time,
            compliance_rate: article23_compliance_rate,
            early_warning_indicators,
        },
        liability_protection_status,
        recommendations,
    })
}

// ========== AI EXPLAINABILITY & OBSERVABILITY ==========

/// Get AI decision explanation
#[derive(Serialize, ToSchema)]
pub struct AIDecisionExplanationResponse {
    pub decision_id: String,
    pub seal_id: String,
    pub agent_id: String,
    pub decision_type: String,
    pub decision_outcome: String,
    pub explanation_text: String,
    pub feature_importance: Option<serde_json::Value>,
    pub decision_path: Option<serde_json::Value>,
    pub confidence_score: Option<f64>,
    pub created_at: String,
}

#[utoipa::path(
    get,
    path = "/models/{model_id}/explanations/{decision_id}",
    tag = "AI Explainability",
    params(
        ("model_id" = String, Path, description = "Model ID"),
        ("decision_id" = String, Path, description = "Decision ID"),
    ),
    responses(
        (status = 200, description = "Decision explanation", body = AIDecisionExplanationResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_decision_explanation(
    path: web::Path<(String, String)>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "explainability", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let (_model_id, decision_id) = path.into_inner();

    #[derive(sqlx::FromRow)]
    struct ExplanationRow {
        decision_id: String,
        seal_id: String,
        agent_id: String,
        decision_type: String,
        decision_outcome: String,
        explanation_text: String,
        feature_importance: Option<serde_json::Value>,
        decision_path: Option<serde_json::Value>,
        confidence_score: Option<f64>,
        created_at: DateTime<Utc>,
    }

    let explanation: Option<ExplanationRow> = sqlx::query_as(
        "SELECT decision_id, seal_id, agent_id, decision_type, decision_outcome,
                explanation_text, feature_importance, decision_path, confidence_score, created_at
         FROM ai_decision_explanations
         WHERE decision_id = $1"
    )
    .bind(&decision_id)
    .fetch_optional(&data.db_pool)
    .await
    .ok()
    .flatten();

    match explanation {
        Some(exp) => HttpResponse::Ok().json(AIDecisionExplanationResponse {
            decision_id: exp.decision_id,
            seal_id: exp.seal_id,
            agent_id: exp.agent_id,
            decision_type: exp.decision_type,
            decision_outcome: exp.decision_outcome,
            explanation_text: exp.explanation_text,
            feature_importance: exp.feature_importance,
            decision_path: exp.decision_path,
            confidence_score: exp.confidence_score,
            created_at: exp.created_at.to_rfc3339(),
        }),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": "DECISION_NOT_FOUND",
            "message": "Decision explanation not found"
        }))
    }
}

/// Get feature importance for a model
#[derive(Serialize, ToSchema)]
pub struct FeatureImportanceResponse {
    pub feature_name: String,
    pub importance_score: f64,
    pub importance_rank: Option<i32>,
    pub feature_type: Option<String>,
}

#[utoipa::path(
    get,
    path = "/models/{model_id}/feature-importance",
    tag = "AI Explainability",
    params(
        ("model_id" = String, Path, description = "Model ID"),
    ),
    responses(
        (status = 200, description = "Feature importance", body = FeatureImportanceResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_feature_importance(
    model_id: web::Path<String>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "explainability", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let model_id = model_id.into_inner();

    match crate::core::ai_explainability::AIExplainabilityService::get_feature_importance(
        &data.db_pool,
        &model_id,
    ).await {
        Ok(features) => {
            HttpResponse::Ok().json(features.iter().map(|f| FeatureImportanceResponse {
                feature_name: f.feature_name.clone(),
                importance_score: f.importance_score,
                importance_rank: f.importance_rank,
                feature_type: f.feature_type.clone(),
            }).collect::<Vec<_>>())
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "FEATURE_IMPORTANCE_FETCH_FAILED",
            "message": format!("Failed to fetch feature importance: {}", e)
        }))
    }
}

/// Detect model drift
#[derive(Serialize, ToSchema)]
pub struct ModelDriftResponse {
    pub model_id: String,
    pub drift_type: String,
    pub drift_score: f64,
    pub drift_severity: String,
    pub affected_features: Vec<String>,
    pub baseline_date: Option<String>,
    pub detected_at: String,
}

#[utoipa::path(
    get,
    path = "/models/{model_id}/drift",
    tag = "AI Explainability",
    params(
        ("model_id" = String, Path, description = "Model ID"),
    ),
    responses(
        (status = 200, description = "Model drift information", body = ModelDriftResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_model_drift(
    model_id: web::Path<String>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "explainability", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let model_id = model_id.into_inner();

    match crate::core::ai_explainability::AIExplainabilityService::detect_model_drift(
        &data.db_pool,
        &model_id,
    ).await {
        Ok(Some(drift)) => HttpResponse::Ok().json(ModelDriftResponse {
            model_id: drift.model_id,
            drift_type: drift.drift_type,
            drift_score: drift.drift_score,
            drift_severity: drift.drift_severity,
            affected_features: drift.affected_features,
            baseline_date: drift.baseline_date.map(|dt| dt.to_rfc3339()),
            detected_at: drift.detected_at.to_rfc3339(),
        }),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "status": "NO_DRIFT_DETECTED",
            "message": "No model drift detected in the last 7 days"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "DRIFT_DETECTION_FAILED",
            "message": format!("Failed to detect drift: {}", e)
        }))
    }
}

// ========== CONFIGURATION DRIFT DETECTION ==========

/// Create configuration baseline
#[derive(Deserialize, ToSchema)]
pub struct CreateBaselineRequest {
    pub baseline_name: String,
    pub baseline_type: String,
    pub baseline_config: serde_json::Value,
    pub is_golden_image: bool,
    pub description: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct BaselineResponse {
    pub id: String,
    pub baseline_name: String,
    pub baseline_type: String,
    pub is_golden_image: bool,
    pub created_at: String,
}

#[utoipa::path(
    post,
    path = "/configuration/baselines",
    tag = "Configuration Drift",
    request_body = CreateBaselineRequest,
    responses(
        (status = 200, description = "Baseline created", body = BaselineResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn create_baseline(
    req: web::Json<CreateBaselineRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let claims = match authenticate_and_authorize(&http_req, &data.db_pool, "configuration", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let req_data = req.into_inner();

    match crate::core::configuration_drift::ConfigurationDriftService::create_baseline(
        &data.db_pool,
        &req_data.baseline_name,
        &req_data.baseline_type,
        &req_data.baseline_config,
        req_data.is_golden_image,
        Some(&claims.sub),
        req_data.description.as_deref(),
    ).await {
        Ok(baseline_id) => {
            HttpResponse::Ok().json(BaselineResponse {
                id: baseline_id.to_string(),
                baseline_name: req_data.baseline_name,
                baseline_type: req_data.baseline_type,
                is_golden_image: req_data.is_golden_image,
                created_at: Utc::now().to_rfc3339(),
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "BASELINE_CREATION_FAILED",
            "message": format!("Failed to create baseline: {}", e)
        }))
    }
}

/// Detect configuration drift
#[derive(Deserialize, ToSchema)]
pub struct DetectDriftRequest {
    pub current_config: serde_json::Value,
}

#[derive(Serialize, ToSchema)]
pub struct DriftDetectionResponse {
    pub drifts: Vec<DriftResponse>,
    pub total_count: usize,
}

#[derive(Serialize, ToSchema)]
pub struct DriftResponse {
    pub id: String,
    pub baseline_id: String,
    pub drift_type: String,
    pub drift_severity: String,
    pub changed_path: String,
    pub detected_at: String,
    pub auto_remediated: bool,
}

#[utoipa::path(
    post,
    path = "/configuration/baselines/{baseline_id}/detect-drift",
    tag = "Configuration Drift",
    params(
        ("baseline_id" = Uuid, Path, description = "Baseline ID"),
    ),
    request_body = DetectDriftRequest,
    responses(
        (status = 200, description = "Drift detected"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn detect_configuration_drift(
    baseline_id: web::Path<Uuid>,
    req: web::Json<DetectDriftRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "configuration", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let baseline_id = baseline_id.into_inner();
    let req_data = req.into_inner();

    match crate::core::configuration_drift::ConfigurationDriftService::detect_drift(
        &data.db_pool,
        baseline_id,
        &req_data.current_config,
    ).await {
        Ok(drifts) => {
            HttpResponse::Ok().json(DriftDetectionResponse {
                drifts: drifts.iter().map(|d| DriftResponse {
                    id: d.id.to_string(),
                    baseline_id: d.baseline_id.to_string(),
                    drift_type: d.drift_type.clone(),
                    drift_severity: d.drift_severity.clone(),
                    changed_path: d.changed_path.clone(),
                    detected_at: d.detected_at.to_rfc3339(),
                    auto_remediated: d.auto_remediated,
                }).collect(),
                total_count: drifts.len(),
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "DRIFT_DETECTION_FAILED",
            "message": format!("Failed to detect drift: {}", e)
        }))
    }
}

/// Get configuration drifts
#[utoipa::path(
    get,
    path = "/configuration/baselines/{baseline_id}/drifts",
    tag = "Configuration Drift",
    params(
        ("acknowledged" = Option<bool>, Query, description = "Filter by acknowledged status"),
    ),
    responses(
        (status = 200, description = "Configuration drifts"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_configuration_drifts(
    baseline_id: web::Path<Uuid>,
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "configuration", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let baseline_id = baseline_id.into_inner();
    let acknowledged = query.get("acknowledged").and_then(|s| s.parse::<bool>().ok());

    match crate::core::configuration_drift::ConfigurationDriftService::get_drifts(
        &data.db_pool,
        baseline_id,
        acknowledged,
    ).await {
        Ok(drifts) => {
            HttpResponse::Ok().json(drifts.iter().map(|d| serde_json::json!({
                "id": d.id,
                "baseline_id": d.baseline_id,
                "drift_type": d.drift_type,
                "drift_severity": d.drift_severity,
                "changed_path": d.changed_path,
                "old_value": d.old_value,
                "new_value": d.new_value,
                "detected_at": d.detected_at.to_rfc3339(),
                "auto_remediated": d.auto_remediated,
                "acknowledged": d.acknowledged,
            })).collect::<Vec<_>>())
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "DRIFT_FETCH_FAILED",
            "message": format!("Failed to fetch drifts: {}", e)
        }))
    }
}

// ========== MULTI-CLOUD NATIVE INTEGRATIONS ==========

/// Register cloud provider
#[derive(Deserialize, ToSchema)]
pub struct RegisterCloudProviderRequest {
    pub provider: String, // AWS, AZURE, GCP
    pub account_id: String,
    pub region: Option<String>,
    pub credentials_encrypted: String,
}

#[derive(Serialize, ToSchema)]
pub struct CloudProviderResponse {
    pub id: String,
    pub provider: String,
    pub account_id: String,
    pub region: Option<String>,
    pub is_active: bool,
}

#[derive(Serialize, ToSchema)]
pub struct CloudSyncResponse {
    pub sync_id: String,
    pub provider: String,
    pub account_id: String,
    pub status: String,
    pub started_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct CloudComplianceSummaryResponse {
    pub provider: String,
    pub total_resources: i32,
    pub compliant_resources: i32,
    pub non_compliant_resources: i32,
    pub compliance_percentage: f64,
}

#[utoipa::path(
    post,
    path = "/cloud/providers",
    tag = "Multi-Cloud",
    request_body = RegisterCloudProviderRequest,
    responses(
        (status = 200, description = "Provider registered", body = CloudProviderResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn register_cloud_provider(
    req: web::Json<RegisterCloudProviderRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let claims = match authenticate_and_authorize(&http_req, &data.db_pool, "cloud", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let req_data = req.into_inner();

    match crate::integration::multi_cloud::MultiCloudService::register_provider(
        &data.db_pool,
        &req_data.provider,
        &req_data.account_id,
        req_data.region.as_deref(),
        &req_data.credentials_encrypted,
        Some(&claims.sub),
    ).await {
        Ok(_config_id) => {
            HttpResponse::Ok().json(CloudProviderResponse {
                id: Uuid::new_v4().to_string(),
                provider: req_data.provider,
                account_id: req_data.account_id,
                region: req_data.region,
                is_active: true,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "PROVIDER_REGISTRATION_FAILED",
            "message": format!("Failed to register provider: {}", e)
        }))
    }
}

/// Sync cloud compliance
#[utoipa::path(
    post,
    path = "/cloud/providers/{provider}/sync",
    tag = "Multi-Cloud",
    params(
        ("account_id" = String, Query, description = "Cloud account ID"),
    ),
    responses(
        (status = 200, description = "Sync started", body = CloudSyncResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn sync_cloud_compliance(
    provider: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "cloud", "write").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let provider = provider.into_inner();
    let account_id = match query.get("account_id") {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "MISSING_ACCOUNT_ID",
                "message": "account_id is required"
            }));
        }
    };

    match crate::integration::multi_cloud::MultiCloudService::sync_cloud_compliance(
        &data.db_pool,
        &provider,
        account_id,
    ).await {
        Ok(sync_id) => {
            HttpResponse::Ok().json(CloudSyncResponse {
                sync_id,
                provider,
                account_id: account_id.to_string(),
                status: "PENDING".to_string(),
                started_at: Utc::now().to_rfc3339(),
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "SYNC_FAILED",
            "message": format!("Failed to sync: {}", e)
        }))
    }
}

/// Get cloud compliance summary
#[utoipa::path(
    get,
    path = "/cloud/providers/{provider}/compliance",
    tag = "Multi-Cloud",
    responses(
        (status = 200, description = "Compliance summary", body = CloudComplianceSummaryResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_cloud_compliance_summary(
    provider: web::Path<String>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    // AUTHENTICATION & AUTHORIZATION
    let _claims = match authenticate_and_authorize(&http_req, &data.db_pool, "cloud", "read").await {
        Ok(c) => c,
        Err(resp) => return resp,
    };

    let provider = provider.into_inner();

    match crate::integration::multi_cloud::MultiCloudService::get_compliance_summary(
        &data.db_pool,
        &provider,
    ).await {
        Ok(summary) => {
            HttpResponse::Ok().json(CloudComplianceSummaryResponse {
                provider: summary.provider,
                total_resources: summary.total_resources,
                compliant_resources: summary.compliant_resources,
                non_compliant_resources: summary.non_compliant_resources,
                compliance_percentage: summary.compliance_percentage,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "SUMMARY_FETCH_FAILED",
            "message": format!("Failed to fetch summary: {}", e)
        }))
    }
}
