// Library crate for Veridion Nexus
// Exports modules for use in integration tests

pub mod api_state;
pub mod routes;
pub mod compliance_models;
pub mod database;
pub mod models;
pub mod security;
pub mod module_service;
pub mod background_worker;
pub mod deployment;
pub mod services;

pub use services::wizard_service;
pub use services::legislative_service;

// Core Runtime Compliance Engine (mandatory)
pub mod core;
// Operational Modules (optional)
pub mod modules;
// Integration Layer (SDKs, Webhooks, API)
pub mod integration;

#[cfg(test)]
pub mod test_helpers;

pub use api_state::AppState;
pub use routes::*;

// Export health endpoint
use actix_web::{HttpResponse, Responder};

pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "veridion-nexus",
        "version": "1.0.0"
    }))
}
