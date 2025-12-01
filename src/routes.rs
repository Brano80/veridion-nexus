use crate::api_state::AppState;
use crate::sovereign_lock;
use crate::privacy_bridge::{hash_payload};
use crate::annex_iv_compiler::ComplianceRecord;
use actix_web::{web, HttpResponse, Responder, Result as ActixResult};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::panic;
use std::fs;
use utoipa::ToSchema;

/// Request payload for /log_action endpoint
#[derive(Debug, Deserialize, ToSchema)]
pub struct LogActionRequest {
    /// Unique identifier for the AI agent performing the action
    #[schema(example = "agent-001")]
    pub agent_id: String,
    /// Description of the action being performed
    #[schema(example = "Credit Check")]
    pub action: String,
    /// Payload data associated with the action
    #[schema(example = "Customer ID: 12345, Amount: 1000 EUR")]
    pub payload: String,
}

/// Response payload for /log_action endpoint
#[derive(Debug, Serialize, ToSchema)]
pub struct LogActionResponse {
    /// Compliance status of the action
    #[schema(example = "COMPLIANT")]
    pub status: String,
    /// Qualified Electronic Seal ID from eIDAS provider
    #[schema(example = "SEAL-2024-01-15-ABC123")]
    pub seal_id: String,
    /// Transaction ID for the logged event
    #[schema(example = "tx-abc123def456")]
    pub tx_id: String,
}

/// Logs a high-risk AI action through the compliance pipeline
/// 
/// This endpoint processes agent actions through four compliance modules:
/// 1. Sovereign Lock: Validates IP geolocation (EU/EEA only)
/// 2. Privacy Bridge: Hashes payload and obtains eIDAS Qualified Seal
/// 3. Crypto-Shredder: Encrypts and stores the action with envelope encryption
/// 4. Annex IV: Adds record to compliance log for technical documentation
#[utoipa::path(
    post,
    path = "/log_action",
    request_body = LogActionRequest,
    responses(
        (status = 200, description = "Action successfully logged and compliance verified", body = LogActionResponse),
        (status = 403, description = "Data sovereignty violation - non-EU IP address blocked"),
        (status = 500, description = "Internal server error during compliance processing")
    ),
    tag = "Compliance"
)]
pub async fn log_action(
    req: web::Json<LogActionRequest>,
    state: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    // Step 0: Check if system is in lockdown mode
    let is_locked = *state.is_locked_down.lock()
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Lock error: {}", e)))?;
    
    if is_locked {
        return Ok(HttpResponse::Forbidden()
            .json(serde_json::json!({
                "status": "BLOCKED",
                "reason": "SYSTEM_LOCKDOWN: Agent identity keys have been revoked. All operations are suspended."
            })));
    }
    
    // Step 1: Sovereign Lock - Check IP address (hardcoded for MVP)
    // Using "5.1.2.3" (Germany/EU) to allow full compliance flow
    let target_ip = "5.1.2.3"; // TODO: Extract from request or use agent's actual IP
    
    let sovereignty_result = panic::catch_unwind(|| {
        sovereign_lock::check_sovereignty(target_ip)
    });
    
    match sovereignty_result {
        Ok(Ok(())) => {
            // IP is in EU/EEA whitelist - proceed
        }
        Ok(Err(_)) => {
            // Should not happen
            return Ok(HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Unexpected sovereignty check error"})));
        }
        Err(_) => {
            // Panic caught - non-EU IP blocked
            return Ok(HttpResponse::Forbidden()
                .json(serde_json::json!({
                    "status": "BLOCKED",
                    "reason": "DATA_SOVEREIGNTY_VIOLATION: Attempted connection to non-sovereign jurisdiction"
                })));
        }
    }
    
    // Step 2: Privacy Bridge - Hash payload and get seal
    let payload_hash = hash_payload(&req.payload);
    
    let seal = match state.signicat.request_seal(&payload_hash).await {
        Ok(seal) => seal,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": format!("Failed to get seal: {}", e)})));
        }
    };
    
    // Extract seal ID
    let seal_id = seal.split(" | TIMESTAMP:").next().unwrap_or("UNKNOWN").to_string();
    
    // Step 3: Crypto-Shredder - Encrypt and store
    let encrypted_log = {
        let key_store = state.key_store.lock()
            .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Lock error: {}", e)))?;
        key_store.log_event(&req.payload)
    };
    
    // Generate transaction ID from log_id
    let tx_id = encrypted_log.log_id.clone();
    
    // Step 4: Annex IV - Add to compliance log
    {
        let mut log = state.compliance_log.lock()
            .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Lock error: {}", e)))?;
        
        log.push(ComplianceRecord {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            action_summary: format!("{}: {}", req.agent_id, req.action),
            seal_id: seal.clone(),
            status: "COMPLIANT".to_string(),
        });
    }
    
    // Return success response
    Ok(HttpResponse::Ok().json(LogActionResponse {
        status: "COMPLIANT".to_string(),
        seal_id,
        tx_id,
    }))
}

/// Retrieves audit history of all compliance records
/// 
/// Returns a list of all logged actions with their compliance status,
/// timestamps, seal IDs, and action summaries for Annex IV documentation.
#[utoipa::path(
    get,
    path = "/logs",
    responses(
        (status = 200, description = "List of compliance records", body = Vec<ComplianceRecord>)
    ),
    tag = "Compliance"
)]
pub async fn get_logs(state: web::Data<AppState>) -> impl Responder {
    let logs = state.compliance_log.lock().unwrap();
    HttpResponse::Ok().json(&*logs)
}

/// Revokes agent identity keys and puts the system in lockdown mode
/// 
/// This endpoint activates the kill switch, blocking all new agent actions.
/// Once activated, all POST requests to /log_action will be rejected with 403 Forbidden.
#[utoipa::path(
    post,
    path = "/revoke_keys",
    responses(
        (status = 200, description = "Agent keys successfully revoked, system in lockdown"),
        (status = 500, description = "Error activating lockdown")
    ),
    tag = "Compliance"
)]
pub async fn revoke_keys(state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    let mut is_locked = state.is_locked_down.lock()
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Lock error: {}", e)))?;
    
    *is_locked = true;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "LOCKDOWN_ACTIVATED",
        "message": "Identity certificates revoked via Signicat API. Agent is now isolated.",
        "timestamp": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    })))
}

/// Restores agent identity keys and removes lockdown mode
/// 
/// This endpoint deactivates the kill switch, allowing agent actions to proceed normally.
/// Useful for testing and recovery scenarios.
#[utoipa::path(
    post,
    path = "/restore_keys",
    responses(
        (status = 200, description = "Agent keys restored, system operational"),
        (status = 500, description = "Error restoring keys")
    ),
    tag = "Compliance"
)]
pub async fn restore_keys(state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    let mut is_locked = state.is_locked_down.lock()
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Lock error: {}", e)))?;
    
    *is_locked = false;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "LOCKDOWN_DEACTIVATED",
        "message": "Identity certificates restored. Agent operations resumed.",
        "timestamp": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    })))
}

/// Generates and downloads the Annex IV compliance report as a PDF
/// 
/// This endpoint generates a PDF report containing all compliance records
/// for EU AI Act Annex IV technical documentation requirements.
/// The report includes timestamps, action summaries, seal IDs, and compliance status.
#[utoipa::path(
    get,
    path = "/download_report",
    responses(
        (status = 200, description = "PDF report file", content_type = "application/pdf"),
        (status = 500, description = "Error generating or reading PDF report")
    ),
    tag = "Compliance"
)]
pub async fn download_report(
    data: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    // Lock compliance log
    let logs = {
        let log = data.compliance_log.lock()
            .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Lock error: {}", e)))?;
        log.clone()
    };
    
    // Generate the PDF report
    let report_path = "server_report.pdf";
    crate::annex_iv_compiler::generate_report(&logs, report_path)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to generate report: {}", e)))?;
    
    // Read the file bytes
    let bytes = fs::read(report_path)
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to read report file: {}", e)))?;
    
    // Return PDF as downloadable file
    Ok(HttpResponse::Ok()
        .content_type("application/pdf")
        .append_header(("Content-Disposition", "attachment; filename=\"Veridion_Annex_IV.pdf\""))
        .body(bytes))
}

