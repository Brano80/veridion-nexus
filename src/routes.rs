use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::api_state::AppState;
use crate::annex_iv_compiler::ComplianceRecord;
use utoipa::ToSchema;
use actix_web::http::header::ContentDisposition;
use std::fs;
use chrono::Local;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LogRequest {
    pub agent_id: String,
    pub action: String,
    pub payload: String,
    pub target_region: Option<String>, // Nové: Simulácia cieľovej krajiny
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LogResponse {
    pub status: String,
    pub seal_id: String,
    pub tx_id: String,
}

// Definujeme, čo nám Frontend pošle (ID riadku)
#[derive(Deserialize, ToSchema)]
pub struct ShredRequest {
    pub seal_id: String,
}

// 1. LOG ACTION (S logikou pre BLOCKING)

#[utoipa::path(
    post,
    path = "/log_action",
    request_body = LogRequest,
    responses((status = 200, body = LogResponse), (status = 403, body = LogResponse))
)]
pub async fn log_action(
    req: web::Json<LogRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    
    // A. KONTROLA KILL SWITCHU
    {
        let revoked = data.is_locked_down.lock().unwrap();
        if *revoked {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "status": "SYSTEM_LOCKDOWN",
                "reason": "Agent Identity Revoked"
            }));
        }
    }

    // B. SOVEREIGN LOCK (Simulácia)
    // Ak agent posiela dáta do "US", systém to zablokuje
    let is_violation = req.target_region.as_deref() == Some("US");
    
    let status = if is_violation { "BLOCKED (SOVEREIGNTY)" } else { "COMPLIANT" };

    // C. PRIVACY BRIDGE (Signicat)
    let log_hash = crate::privacy_bridge::hash_payload(&req.payload);
    let seal_result = data.signicat.request_seal(&log_hash).await;
    let seal_id = seal_result.unwrap_or_else(|e| format!("ERROR: {}", e));

    // D. CRYPTO-SHREDDER
    let encrypted_log = data.key_store.lock().unwrap().log_event(&req.payload);

    // E. ANNEX IV COMPILER
    let record = ComplianceRecord {
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        action_summary: format!("{}: {}", req.agent_id, req.action),
        seal_id: seal_id.clone(),
        status: status.to_string(),
    };

    data.compliance_log.lock().unwrap().push(record);

    println!("Log: {} | Status: {}", req.action, status);

    if is_violation {
        // Vrátime 403, aby agent vedel, že neprešiel
        HttpResponse::Forbidden().json(LogResponse {
            status: status.to_string(),
            seal_id: "N/A (Connection Refused)".to_string(),
            tx_id: "0000".to_string(),
        })
    } else {
        HttpResponse::Ok().json(LogResponse {
            status: status.to_string(),
            seal_id,
            tx_id: encrypted_log.log_id,
        })
    }
}

// 2. GET LOGS

#[utoipa::path(get, path = "/logs")]
pub async fn get_logs(data: web::Data<AppState>) -> impl Responder {
    let logs = data.compliance_log.lock().unwrap();
    HttpResponse::Ok().json(&*logs)
}

// 3. CRYPTO-SHREDDER (Vymazanie dát)

// POST /shred_data (Teraz cieli na konkrétny riadok)
#[utoipa::path(
    post, 
    path = "/shred_data", 
    request_body = ShredRequest
)]
pub async fn shred_data(
    req: web::Json<ShredRequest>, // Prijmeme JSON s ID
    data: web::Data<AppState>
) -> impl Responder {
    let mut logs = data.compliance_log.lock().unwrap();
    
    // Nájdeme riadok s daným Seal ID a upravíme ho
    if let Some(record) = logs.iter_mut().find(|r| r.seal_id == req.seal_id) {
        record.action_summary = "[GDPR PURGED] Data Cryptographically Erased".to_string();
        record.status = "ERASED (Art. 17)".to_string();
        
        // V reálnom svete by sme tu volali key_store.shred_key(tx_id)
        println!("🗑️ Shredded record: {}", req.seal_id);
        
        return HttpResponse::Ok().json(serde_json::json!({"status": "SUCCESS"}));
    }

    HttpResponse::NotFound().json(serde_json::json!({"status": "NOT_FOUND"}))
}

// ... (Ostatné endpointy download_report a revoke_access ostávajú rovnaké, len ich tam nechajte alebo doplňte z minula) ...
// Pre istotu tu pridávam download_report a revoke_access aby to bolo kompletné:

#[utoipa::path(get, path = "/download_report")]
pub async fn download_report(data: web::Data<AppState>) -> impl Responder {
    let logs = data.compliance_log.lock().unwrap();
    let _ = crate::annex_iv_compiler::generate_report(&logs, "server_report.pdf");
    if let Ok(bytes) = fs::read("server_report.pdf") {
        HttpResponse::Ok().content_type("application/pdf")
            .append_header(ContentDisposition::attachment("Veridion_Annex_IV.pdf")).body(bytes)
    } else { HttpResponse::InternalServerError().finish() }
}

#[utoipa::path(post, path = "/revoke_access")]
pub async fn revoke_access(data: web::Data<AppState>) -> impl Responder {
    let mut revoked = data.is_locked_down.lock().unwrap();
    *revoked = true;
    HttpResponse::Ok().json(serde_json::json!({"status": "SUCCESS"}))
}
