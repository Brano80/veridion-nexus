// DORA Lite: Simplified DORA compliance for Startups/SMEs (Fáza 1)
// Following principle of proportionality - simplified requirements
// This is NOT the full Enterprise DORA (see routes.rs for Enterprise DORA)

use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use sqlx::FromRow;
use crate::api_state::AppState;
use crate::security::{AuthService, extract_claims, RbacService, require_permission, Claims};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ========== DORA LITE INCIDENT LOG (Article 10 Simplified) ==========

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DORALiteIncident {
    pub id: Uuid,
    pub incident_type: String,
    pub description: String,
    pub detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub severity: String,
    pub status: String,
    pub impact_description: Option<String>,
    pub mitigation_steps: Option<String>,
    pub reported_to_authority: bool,
    pub reported_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateDORALiteIncidentRequest {
    pub incident_type: String,
    pub description: String,
    pub severity: String,
    pub impact_description: Option<String>,
    pub mitigation_steps: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct DORALiteIncidentList {
    pub incidents: Vec<DORALiteIncident>,
    pub total: i64,
    pub open_incidents: i64,
    pub resolved_incidents: i64,
}

/// Create a DORA Lite incident (Article 10 simplified)
#[utoipa::path(
    post,
    path = "/dora-lite/incidents",
    tag = "DORA Lite",
    request_body = CreateDORALiteIncidentRequest,
    responses(
        (status = 200, description = "Incident created", body = DORALiteIncident),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn create_dora_lite_incident(
    body: web::Json<CreateDORALiteIncidentRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let auth_service = match AuthService::new() {
        Ok(service) => service,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to initialize auth service: {}", e)
        })),
    };
    let claims = match extract_claims(&http_req, &auth_service) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let rbac = RbacService::new(data.db_pool.clone());
    if let Err(resp) = require_permission(&http_req, &rbac, &claims, "dora_lite", "write").await {
        return resp;
    }

    #[derive(FromRow)]
    struct IncidentRow {
        id: Uuid,
        incident_type: String,
        description: String,
        detected_at: DateTime<Utc>,
        resolved_at: Option<DateTime<Utc>>,
        severity: String,
        status: String,
        impact_description: Option<String>,
        mitigation_steps: Option<String>,
        reported_to_authority: bool,
        reported_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    let incident = sqlx::query_as::<_, IncidentRow>(
        r#"
        INSERT INTO dora_lite_incidents (
            incident_type, description, severity, impact_description, mitigation_steps, status
        )
        VALUES ($1, $2, $3, $4, $5, 'OPEN')
        RETURNING
            id, incident_type, description, detected_at, resolved_at, severity, status,
            impact_description, mitigation_steps, reported_to_authority, reported_at,
            created_at, updated_at
        "#
    )
    .bind(&body.incident_type)
    .bind(&body.description)
    .bind(&body.severity)
    .bind(&body.impact_description)
    .bind(&body.mitigation_steps)
    .fetch_one(&data.db_pool)
    .await;

    match incident {
        Ok(inc) => {
            let dora_incident = DORALiteIncident {
                id: inc.id,
                incident_type: inc.incident_type,
                description: inc.description,
                detected_at: inc.detected_at,
                resolved_at: inc.resolved_at,
                severity: inc.severity,
                status: inc.status,
                impact_description: inc.impact_description,
                mitigation_steps: inc.mitigation_steps,
                reported_to_authority: inc.reported_to_authority,
                reported_at: inc.reported_at,
                created_at: inc.created_at,
                updated_at: inc.updated_at,
            };
            HttpResponse::Ok().json(dora_incident)
        }
        Err(e) => {
            eprintln!("Error creating DORA Lite incident: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create incident"
            }))
        }
    }
}

/// Get DORA Lite incidents
#[utoipa::path(
    get,
    path = "/dora-lite/incidents",
    tag = "DORA Lite",
    params(
        ("status" = Option<String>, Query, description = "Filter by status"),
        ("severity" = Option<String>, Query, description = "Filter by severity"),
        ("limit" = Option<i64>, Query, description = "Limit results"),
    ),
    responses(
        (status = 200, description = "List of incidents", body = DORALiteIncidentList),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_dora_lite_incidents(
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let auth_service = match AuthService::new() {
        Ok(service) => service,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to initialize auth service: {}", e)
        })),
    };
    let claims = match extract_claims(&http_req, &auth_service) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let rbac = RbacService::new(data.db_pool.clone());
    if let Err(resp) = require_permission(&http_req, &rbac, &claims, "dora_lite", "read").await {
        return resp;
    }

    let status_filter = query.get("status");
    let severity_filter = query.get("severity");
    let limit = query.get("limit")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(100);

    let mut query_str = "SELECT id, incident_type, description, detected_at, resolved_at, severity, status, impact_description, mitigation_steps, reported_to_authority, reported_at, created_at, updated_at FROM dora_lite_incidents WHERE 1=1".to_string();
    let mut params: Vec<String> = vec![];

    if let Some(status) = status_filter {
        query_str.push_str(" AND status = $");
        params.push(status.clone());
        query_str.push_str(&params.len().to_string());
    }

    if let Some(severity) = severity_filter {
        query_str.push_str(" AND severity = $");
        params.push(severity.clone());
        query_str.push_str(&params.len().to_string());
    }

    query_str.push_str(" ORDER BY detected_at DESC LIMIT $");
    params.push(limit.to_string());
    query_str.push_str(&params.len().to_string());

    #[derive(FromRow)]
    struct IncidentRow {
        id: Uuid,
        incident_type: String,
        description: String,
        detected_at: DateTime<Utc>,
        resolved_at: Option<DateTime<Utc>>,
        severity: String,
        status: String,
        impact_description: Option<String>,
        mitigation_steps: Option<String>,
        reported_to_authority: bool,
        reported_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    // Simplified query execution - use static query for now
    let incidents_result = sqlx::query_as::<_, IncidentRow>(
        "SELECT id, incident_type, description, detected_at, resolved_at, severity, status, impact_description, mitigation_steps, reported_to_authority, reported_at, created_at, updated_at FROM dora_lite_incidents ORDER BY detected_at DESC LIMIT $1"
    )
    .bind(limit)
    .fetch_all(&data.db_pool)
    .await;

    let incidents: Vec<DORALiteIncident> = match incidents_result {
        Ok(incs) => incs.into_iter().map(|inc| DORALiteIncident {
            id: inc.id,
            incident_type: inc.incident_type,
            description: inc.description,
            detected_at: inc.detected_at,
            resolved_at: inc.resolved_at,
            severity: inc.severity,
            status: inc.status,
            impact_description: inc.impact_description,
            mitigation_steps: inc.mitigation_steps,
            reported_to_authority: inc.reported_to_authority,
            reported_at: inc.reported_at,
            created_at: inc.created_at,
            updated_at: inc.updated_at,
        }).collect(),
        Err(e) => {
            eprintln!("Error fetching DORA Lite incidents: {}", e);
            vec![]
        }
    };

    let total = incidents.len() as i64;
    let open_incidents = incidents.iter()
        .filter(|i| i.status == "OPEN" || i.status == "IN_PROGRESS")
        .count() as i64;
    let resolved_incidents = incidents.iter()
        .filter(|i| i.status == "RESOLVED" || i.status == "CLOSED")
        .count() as i64;

    HttpResponse::Ok().json(DORALiteIncidentList {
        incidents,
        total,
        open_incidents,
        resolved_incidents,
    })
}

// ========== DORA LITE VENDOR LIST (Article 9 Simplified) ==========

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DORALiteVendor {
    pub id: Uuid,
    pub vendor_name: String,
    pub vendor_type: String,
    pub service_description: Option<String>,
    pub country_code: Option<String>,
    pub contact_email: Option<String>,
    pub sla_uptime_percentage: Option<rust_decimal::Decimal>,
    pub last_reviewed_at: Option<DateTime<Utc>>,
    pub risk_level: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateDORALiteVendorRequest {
    pub vendor_name: String,
    pub vendor_type: String,
    pub service_description: Option<String>,
    pub country_code: Option<String>,
    pub contact_email: Option<String>,
    pub sla_uptime_percentage: Option<f64>,
    pub risk_level: Option<String>,
    pub notes: Option<String>,
}

/// Create a DORA Lite vendor (Article 9 simplified)
#[utoipa::path(
    post,
    path = "/dora-lite/vendors",
    tag = "DORA Lite",
    request_body = CreateDORALiteVendorRequest,
    responses(
        (status = 200, description = "Vendor created", body = DORALiteVendor),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn create_dora_lite_vendor(
    body: web::Json<CreateDORALiteVendorRequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let auth_service = match AuthService::new() {
        Ok(service) => service,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to initialize auth service: {}", e)
        })),
    };
    let claims = match extract_claims(&http_req, &auth_service) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let rbac = RbacService::new(data.db_pool.clone());
    if let Err(resp) = require_permission(&http_req, &rbac, &claims, "dora_lite", "write").await {
        return resp;
    }

    let sla_uptime = body.sla_uptime_percentage.map(|v| rust_decimal::Decimal::from_f64_retain(v).unwrap_or(rust_decimal::Decimal::ZERO));

    #[derive(FromRow)]
    struct VendorRow {
        id: Uuid,
        vendor_name: String,
        vendor_type: String,
        service_description: Option<String>,
        country_code: Option<String>,
        contact_email: Option<String>,
        sla_uptime_percentage: Option<rust_decimal::Decimal>,
        last_reviewed_at: Option<DateTime<Utc>>,
        risk_level: String,
        notes: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    let vendor = sqlx::query_as::<_, VendorRow>(
        r#"
        INSERT INTO dora_lite_vendors (
            vendor_name, vendor_type, service_description, country_code, contact_email,
            sla_uptime_percentage, risk_level, notes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (vendor_name, vendor_type) DO UPDATE SET
            service_description = EXCLUDED.service_description,
            country_code = EXCLUDED.country_code,
            contact_email = EXCLUDED.contact_email,
            sla_uptime_percentage = EXCLUDED.sla_uptime_percentage,
            risk_level = EXCLUDED.risk_level,
            notes = EXCLUDED.notes,
            updated_at = CURRENT_TIMESTAMP
        RETURNING
            id, vendor_name, vendor_type, service_description, country_code, contact_email,
            sla_uptime_percentage, last_reviewed_at, risk_level, notes, created_at, updated_at
        "#
    )
    .bind(&body.vendor_name)
    .bind(&body.vendor_type)
    .bind(&body.service_description)
    .bind(&body.country_code)
    .bind(&body.contact_email)
    .bind(sla_uptime)
    .bind(body.risk_level.as_deref().unwrap_or("UNKNOWN"))
    .bind(&body.notes)
    .fetch_one(&data.db_pool)
    .await;

    match vendor {
        Ok(v) => {
            let dora_vendor = DORALiteVendor {
                id: v.id,
                vendor_name: v.vendor_name,
                vendor_type: v.vendor_type,
                service_description: v.service_description,
                country_code: v.country_code,
                contact_email: v.contact_email,
                sla_uptime_percentage: v.sla_uptime_percentage,
                last_reviewed_at: v.last_reviewed_at,
                risk_level: v.risk_level,
                notes: v.notes,
                created_at: v.created_at,
                updated_at: v.updated_at,
            };
            HttpResponse::Ok().json(dora_vendor)
        }
        Err(e) => {
            eprintln!("Error creating DORA Lite vendor: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create vendor"
            }))
        }
    }
}

/// Get DORA Lite vendors
#[utoipa::path(
    get,
    path = "/dora-lite/vendors",
    tag = "DORA Lite",
    params(
        ("vendor_type" = Option<String>, Query, description = "Filter by vendor type"),
        ("risk_level" = Option<String>, Query, description = "Filter by risk level"),
    ),
    responses(
        (status = 200, description = "List of vendors", body = Vec<DORALiteVendor>),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_dora_lite_vendors(
    query: web::Query<std::collections::HashMap<String, String>>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let auth_service = match AuthService::new() {
        Ok(service) => service,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to initialize auth service: {}", e)
        })),
    };
    let claims = match extract_claims(&http_req, &auth_service) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let rbac = RbacService::new(data.db_pool.clone());
    if let Err(resp) = require_permission(&http_req, &rbac, &claims, "dora_lite", "read").await {
        return resp;
    }

    let vendor_type_filter = query.get("vendor_type");
    let risk_level_filter = query.get("risk_level");

    let mut query_str = "SELECT id, vendor_name, vendor_type, service_description, country_code, contact_email, sla_uptime_percentage, last_reviewed_at, risk_level, notes, created_at, updated_at FROM dora_lite_vendors WHERE 1=1".to_string();
    let mut params: Vec<String> = vec![];

    if let Some(vt) = vendor_type_filter {
        query_str.push_str(" AND vendor_type = $");
        params.push(vt.clone());
        query_str.push_str(&params.len().to_string());
    }

    if let Some(rl) = risk_level_filter {
        query_str.push_str(" AND risk_level = $");
        params.push(rl.clone());
        query_str.push_str(&params.len().to_string());
    }

    #[derive(FromRow)]
    struct VendorRow {
        id: Uuid,
        vendor_name: String,
        vendor_type: String,
        service_description: Option<String>,
        country_code: Option<String>,
        contact_email: Option<String>,
        sla_uptime_percentage: Option<rust_decimal::Decimal>,
        last_reviewed_at: Option<DateTime<Utc>>,
        risk_level: String,
        notes: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    let vendors_result = sqlx::query_as::<_, VendorRow>(
        "SELECT id, vendor_name, vendor_type, service_description, country_code, contact_email, sla_uptime_percentage, last_reviewed_at, risk_level, notes, created_at, updated_at FROM dora_lite_vendors ORDER BY vendor_name"
    )
    .fetch_all(&data.db_pool)
    .await;

    match vendors_result {
        Ok(vendors) => {
            let dora_vendors: Vec<DORALiteVendor> = vendors.into_iter().map(|v| DORALiteVendor {
                id: v.id,
                vendor_name: v.vendor_name,
                vendor_type: v.vendor_type,
                service_description: v.service_description,
                country_code: v.country_code,
                contact_email: v.contact_email,
                sla_uptime_percentage: v.sla_uptime_percentage,
                last_reviewed_at: v.last_reviewed_at,
                risk_level: v.risk_level,
                notes: v.notes,
                created_at: v.created_at,
                updated_at: v.updated_at,
            }).collect();
            HttpResponse::Ok().json(dora_vendors)
        }
        Err(e) => {
            eprintln!("Error fetching DORA Lite vendors: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch vendors"
            }))
        }
    }
}

// ========== DORA LITE SLA MONITORING (Article 11 Simplified) ==========

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DORALiteSLAMonitoring {
    pub id: Uuid,
    pub service_name: String,
    pub service_type: String,
    pub sla_target_uptime: rust_decimal::Decimal,
    pub actual_uptime: Option<rust_decimal::Decimal>,
    pub monitoring_period_start: DateTime<Utc>,
    pub monitoring_period_end: DateTime<Utc>,
    pub downtime_minutes: i32,
    pub incidents_count: i32,
    pub sla_met: bool,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateDORALiteSLARequest {
    pub service_name: String,
    pub service_type: String,
    pub sla_target_uptime: f64,
    pub monitoring_period_start: DateTime<Utc>,
    pub monitoring_period_end: DateTime<Utc>,
    pub notes: Option<String>,
}

/// Create DORA Lite SLA monitoring (Article 11 simplified)
#[utoipa::path(
    post,
    path = "/dora-lite/sla-monitoring",
    tag = "DORA Lite",
    request_body = CreateDORALiteSLARequest,
    responses(
        (status = 200, description = "SLA monitoring created", body = DORALiteSLAMonitoring),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn create_dora_lite_sla_monitoring(
    body: web::Json<CreateDORALiteSLARequest>,
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let auth_service = match AuthService::new() {
        Ok(service) => service,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to initialize auth service: {}", e)
        })),
    };
    let claims = match extract_claims(&http_req, &auth_service) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let rbac = RbacService::new(data.db_pool.clone());
    if let Err(resp) = require_permission(&http_req, &rbac, &claims, "dora_lite", "write").await {
        return resp;
    }

    let sla_target = rust_decimal::Decimal::from_f64_retain(body.sla_target_uptime)
        .unwrap_or(rust_decimal::Decimal::ZERO);

    #[derive(FromRow)]
    struct SLARow {
        id: Uuid,
        service_name: String,
        service_type: String,
        sla_target_uptime: rust_decimal::Decimal,
        actual_uptime: Option<rust_decimal::Decimal>,
        monitoring_period_start: DateTime<Utc>,
        monitoring_period_end: DateTime<Utc>,
        downtime_minutes: i32,
        incidents_count: i32,
        sla_met: bool,
        notes: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    let sla = sqlx::query_as::<_, SLARow>(
        r#"
        INSERT INTO dora_lite_sla_monitoring (
            service_name, service_type, sla_target_uptime, monitoring_period_start,
            monitoring_period_end, notes
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id, service_name, service_type, sla_target_uptime, actual_uptime,
            monitoring_period_start, monitoring_period_end, downtime_minutes,
            incidents_count, sla_met, notes, created_at, updated_at
        "#
    )
    .bind(&body.service_name)
    .bind(&body.service_type)
    .bind(sla_target)
    .bind(body.monitoring_period_start)
    .bind(body.monitoring_period_end)
    .bind(&body.notes)
    .fetch_one(&data.db_pool)
    .await;

    match sla {
        Ok(s) => {
            let dora_sla = DORALiteSLAMonitoring {
                id: s.id,
                service_name: s.service_name,
                service_type: s.service_type,
                sla_target_uptime: s.sla_target_uptime,
                actual_uptime: s.actual_uptime,
                monitoring_period_start: s.monitoring_period_start,
                monitoring_period_end: s.monitoring_period_end,
                downtime_minutes: s.downtime_minutes,
                incidents_count: s.incidents_count,
                sla_met: s.sla_met,
                notes: s.notes,
                created_at: s.created_at,
                updated_at: s.updated_at,
            };
            HttpResponse::Ok().json(dora_sla)
        }
        Err(e) => {
            eprintln!("Error creating DORA Lite SLA monitoring: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create SLA monitoring"
            }))
        }
    }
}

/// Get DORA Lite SLA monitoring
#[utoipa::path(
    get,
    path = "/dora-lite/sla-monitoring",
    tag = "DORA Lite",
    responses(
        (status = 200, description = "List of SLA monitoring", body = Vec<DORALiteSLAMonitoring>),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_dora_lite_sla_monitoring(
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let auth_service = match AuthService::new() {
        Ok(service) => service,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to initialize auth service: {}", e)
        })),
    };
    let claims = match extract_claims(&http_req, &auth_service) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let rbac = RbacService::new(data.db_pool.clone());
    if let Err(resp) = require_permission(&http_req, &rbac, &claims, "dora_lite", "read").await {
        return resp;
    }

    #[derive(FromRow)]
    struct SLARow {
        id: Uuid,
        service_name: String,
        service_type: String,
        sla_target_uptime: rust_decimal::Decimal,
        actual_uptime: Option<rust_decimal::Decimal>,
        monitoring_period_start: DateTime<Utc>,
        monitoring_period_end: DateTime<Utc>,
        downtime_minutes: i32,
        incidents_count: i32,
        sla_met: bool,
        notes: Option<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    let sla_result = sqlx::query_as::<_, SLARow>(
        r#"
        SELECT id, service_name, service_type, sla_target_uptime, actual_uptime,
               monitoring_period_start, monitoring_period_end, downtime_minutes,
               incidents_count, sla_met, notes, created_at, updated_at
        FROM dora_lite_sla_monitoring
        WHERE monitoring_period_end >= CURRENT_TIMESTAMP
        ORDER BY service_name
        "#
    )
    .fetch_all(&data.db_pool)
    .await;

    match sla_result {
        Ok(slas) => {
            let dora_slas: Vec<DORALiteSLAMonitoring> = slas.into_iter().map(|s| DORALiteSLAMonitoring {
                id: s.id,
                service_name: s.service_name,
                service_type: s.service_type,
                sla_target_uptime: s.sla_target_uptime,
                actual_uptime: s.actual_uptime,
                monitoring_period_start: s.monitoring_period_start,
                monitoring_period_end: s.monitoring_period_end,
                downtime_minutes: s.downtime_minutes,
                incidents_count: s.incidents_count,
                sla_met: s.sla_met,
                notes: s.notes,
                created_at: s.created_at,
                updated_at: s.updated_at,
            }).collect();
            HttpResponse::Ok().json(dora_slas)
        }
        Err(e) => {
            eprintln!("Error fetching DORA Lite SLA monitoring: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch SLA monitoring"
            }))
        }
    }
}

// ========== DORA LITE COMPLIANCE STATUS ==========

#[derive(Serialize, ToSchema)]
pub struct DORALiteComplianceStatus {
    pub compliance_score: f64,
    pub article9_compliant: bool,
    pub article10_compliant: bool,
    pub article11_compliant: bool,
    pub vendor_count: i64,
    pub incident_count: i64,
    pub sla_count: i64,
    pub recommendations: Vec<String>,
}

/// Get DORA Lite compliance status
#[utoipa::path(
    get,
    path = "/dora-lite/compliance-status",
    tag = "DORA Lite",
    responses(
        (status = 200, description = "DORA Lite compliance status", body = DORALiteComplianceStatus),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_dora_lite_compliance_status(
    data: web::Data<AppState>,
    http_req: HttpRequest,
) -> impl Responder {
    let auth_service = match AuthService::new() {
        Ok(service) => service,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to initialize auth service: {}", e)
        })),
    };
    let claims = match extract_claims(&http_req, &auth_service) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let rbac = RbacService::new(data.db_pool.clone());
    if let Err(resp) = require_permission(&http_req, &rbac, &claims, "dora_lite", "read").await {
        return resp;
    }

    // Get vendor count
    let vendor_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM dora_lite_vendors"
    )
    .fetch_one(&data.db_pool)
    .await
    .unwrap_or(0);

    // Get incident count (last year)
    let incident_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM dora_lite_incidents WHERE detected_at >= CURRENT_TIMESTAMP - INTERVAL '1 year'"
    )
    .fetch_one(&data.db_pool)
    .await
    .unwrap_or(0);

    // Get active SLA count
    let sla_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM dora_lite_sla_monitoring WHERE monitoring_period_end >= CURRENT_TIMESTAMP"
    )
    .fetch_one(&data.db_pool)
    .await
    .unwrap_or(0);

    // Calculate compliance
    let article9_compliant = vendor_count > 0;
    let article10_compliant = true; // Incident log table exists = compliant
    let article11_compliant = sla_count > 0;

    let compliance_score = {
        let mut score = 0.0;
        if article9_compliant { score += 33.33; }
        if article10_compliant { score += 33.33; }
        if article11_compliant { score += 33.34; }
        score
    };

    let mut recommendations = Vec::new();
    if !article9_compliant {
        recommendations.push("Register at least one vendor (cloud provider, AI provider, etc.) to meet Article 9 requirements".to_string());
    }
    if !article11_compliant {
        recommendations.push("Set up SLA monitoring for at least one service to meet Article 11 requirements".to_string());
    }
    if vendor_count == 0 && sla_count == 0 {
        recommendations.push("Start by registering your cloud provider and setting up basic SLA monitoring".to_string());
    }

    HttpResponse::Ok().json(DORALiteComplianceStatus {
        compliance_score,
        article9_compliant,
        article10_compliant,
        article11_compliant,
        vendor_count,
        incident_count,
        sla_count,
        recommendations,
    })
}

