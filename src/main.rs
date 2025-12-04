use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use chrono::Local;
use uuid::Uuid;

// Dáta, ktoré držíme v pamäti (Logy)
struct AppState {
    logs: Mutex<Vec<ComplianceRecord>>,
}

#[derive(Serialize, Clone)]
struct ComplianceRecord {
    timestamp: String,
    action_summary: String,
    seal_id: String,
    status: String,
    tx_id: String,
}

// Čo nám posiela Python/MCP?
#[derive(Deserialize)]
struct LogRequest {
    agent_id: String,
    action: String,
    payload: String,
    target_region: Option<String>, // TOTO JE KĽÚČOVÉ
}

#[derive(Serialize)]
struct LogResponse {
    status: String,
    seal_id: String,
    tx_id: String,
}

// 1. ENDPOINT: LOG ACTION
async fn log_action(req: web::Json<LogRequest>, data: web::Data<AppState>) -> impl Responder {
    
    // --- TU JE TÁ "DRÁMA" ---
    // Ak je región US, zablokujeme to!
    let is_violation = req.target_region.as_deref() == Some("US");
    
    let status = if is_violation { 
        "BLOCKED (SOVEREIGNTY)" 
    } else { 
        "COMPLIANT" 
    };

    let seal_id = if is_violation {
        "N/A".to_string()
    } else {
        Uuid::new_v4().to_string()
    };

    let tx_id = Uuid::new_v4().to_string();

    // Uložíme do pamäte pre Dashboard
    let new_record = ComplianceRecord {
        timestamp: Local::now().format("%H:%M:%S").to_string(),
        action_summary: format!("{}: {}", req.agent_id, req.action),
        seal_id: seal_id.clone(),
        status: status.to_string(),
        tx_id: tx_id.clone(),
    };

    let mut logs = data.logs.lock().unwrap();
    logs.insert(0, new_record); // Pridáme na začiatok zoznamu

    // Vrátime odpoveď Agentovi
    if is_violation {
        println!("🛑 BLOCKED request to US from {}", req.agent_id);
        HttpResponse::Forbidden().json(LogResponse {
            status: status.to_string(),
            seal_id: "BLOCKED".to_string(),
            tx_id: "0000".to_string(),
        })
    } else {
        println!("✅ SEALED request from {}", req.agent_id);
        HttpResponse::Ok().json(LogResponse {
            status: status.to_string(),
            seal_id,
            tx_id,
        })
    }
}

// 2. ENDPOINT: GET LOGS (Pre Dashboard)
async fn get_logs(data: web::Data<AppState>) -> impl Responder {
    let logs = data.logs.lock().unwrap();
    HttpResponse::Ok().json(&*logs)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        logs: Mutex::new(Vec::new()),
    });

    println!("🚀 Veridion Nexus Backend running on port 8080");

    HttpServer::new(move || {
        let cors = Cors::permissive(); // Povolíme všetko pre demo

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .route("/log_action", web::post().to(log_action))
            .route("/logs", web::get().to(get_logs))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
