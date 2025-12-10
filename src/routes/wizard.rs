// Wizard API Routes: Company profiles, module recommendations, pricing, and trial management

use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::api_state::AppState;
use crate::services::wizard_service::{WizardService, CreateCompanyProfileRequest, StartTrialRequest};
use crate::security::{AuthService, extract_claims};
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CompanyProfileResponse {
    pub id: Uuid,
    pub company_name: String,
    pub industry: String,
    pub company_size: String,
    pub country: String,
    pub regulatory_requirements: Vec<String>,
    pub ai_use_cases: Vec<String>,
    pub deployment_preference: String,
    pub estimated_ai_systems: i32,
    pub wizard_completed: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RecommendModulesRequest {
    pub industry: String,
    pub regulatory_requirements: Vec<String>,
    pub ai_use_cases: Vec<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CalculatePriceRequest {
    pub selected_modules: Vec<String>,
    pub num_systems: i32,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SubscriptionResponse {
    pub id: Uuid,
    pub company_id: Uuid,
    pub subscription_type: String,
    pub status: String,
    pub trial_start_date: Option<String>,
    pub trial_end_date: Option<String>,
    pub days_remaining: Option<i32>,
    pub monthly_price: Option<f64>,
    pub annual_price: Option<f64>,
}

/// Create or update company profile
#[utoipa::path(
    post,
    path = "/wizard/company-profile",
    request_body = CreateCompanyProfileRequest,
    responses((status = 200, body = CompanyProfileResponse))
)]
pub async fn create_company_profile(
    body: web::Json<CreateCompanyProfileRequest>,
    http_req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Responder {
    // Optional authentication - wizard can be used without auth for initial setup
    let auth_service = match AuthService::new() {
        Ok(service) => service,
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to initialize auth service"
            }));
        }
    };
    
    let user_id = if let Ok(claims) = extract_claims(&http_req, &auth_service) {
        uuid::Uuid::parse_str(&claims.sub).ok()
    } else {
        None // Allow unauthenticated access for wizard
    };

    let wizard_service = WizardService::new(data.db_pool.clone());
    
    match wizard_service.create_or_update_company_profile(body.into_inner(), user_id).await {
        Ok(profile) => {
            HttpResponse::Ok().json(CompanyProfileResponse {
                id: profile.id,
                company_name: profile.company_name,
                industry: profile.industry,
                company_size: profile.company_size,
                country: profile.country,
                regulatory_requirements: profile.regulatory_requirements,
                ai_use_cases: profile.ai_use_cases,
                deployment_preference: profile.deployment_preference,
                estimated_ai_systems: profile.estimated_ai_systems,
                wizard_completed: profile.wizard_completed,
            })
        }
        Err(e) => {
            eprintln!("Error creating company profile: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create company profile"
            }))
        }
    }
}

/// Get company profile by ID
#[utoipa::path(
    get,
    path = "/wizard/company-profile/{company_id}",
    responses((status = 200, body = CompanyProfileResponse))
)]
pub async fn get_company_profile(
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
) -> impl Responder {
    let company_id = path.into_inner();
    let wizard_service = WizardService::new(data.db_pool.clone());
    
    match wizard_service.get_company_profile(company_id).await {
        Ok(Some(profile)) => {
            HttpResponse::Ok().json(CompanyProfileResponse {
                id: profile.id,
                company_name: profile.company_name,
                industry: profile.industry,
                company_size: profile.company_size,
                country: profile.country,
                regulatory_requirements: profile.regulatory_requirements,
                ai_use_cases: profile.ai_use_cases,
                deployment_preference: profile.deployment_preference,
                estimated_ai_systems: profile.estimated_ai_systems,
                wizard_completed: profile.wizard_completed,
            })
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Company profile not found"
            }))
        }
        Err(e) => {
            eprintln!("Error getting company profile: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get company profile"
            }))
        }
    }
}

/// Get recommended modules based on company profile
#[utoipa::path(
    post,
    path = "/wizard/recommend-modules",
    request_body = RecommendModulesRequest,
    responses((status = 200, body = crate::services::wizard_service::ModuleRecommendationResponse))
)]
pub async fn recommend_modules(
    body: web::Json<RecommendModulesRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let wizard_service = WizardService::new(data.db_pool.clone());
    
    match wizard_service.get_recommended_modules(
        &body.industry,
        &body.regulatory_requirements,
        &body.ai_use_cases,
    ).await {
        Ok(recommendations) => {
            HttpResponse::Ok().json(recommendations)
        }
        Err(e) => {
            eprintln!("Error getting module recommendations: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get module recommendations"
            }))
        }
    }
}

/// Calculate pricing based on selected modules and number of systems
#[utoipa::path(
    post,
    path = "/wizard/calculate-price",
    request_body = CalculatePriceRequest,
    responses((status = 200, body = crate::services::wizard_service::PricingBreakdown))
)]
pub async fn calculate_price(
    body: web::Json<CalculatePriceRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let wizard_service = WizardService::new(data.db_pool.clone());
    
    match wizard_service.calculate_pricing(&body.selected_modules, body.num_systems).await {
        Ok(pricing) => {
            HttpResponse::Ok().json(pricing)
        }
        Err(e) => {
            eprintln!("Error calculating price: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to calculate price"
            }))
        }
    }
}

/// Start free trial (3 months in Shadow Mode)
#[utoipa::path(
    post,
    path = "/wizard/start-trial",
    request_body = StartTrialRequest,
    responses((status = 200, body = SubscriptionResponse))
)]
pub async fn start_trial(
    body: web::Json<StartTrialRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let wizard_service = WizardService::new(data.db_pool.clone());
    
    match wizard_service.start_trial(body.into_inner()).await {
        Ok(subscription) => {
            // Calculate days remaining
            let days_remaining = subscription.trial_end_date
                .map(|end| {
                    let now = chrono::Utc::now();
                    if end > now {
                        (end - now).num_days() as i32
                    } else {
                        0
                    }
                });

            // Set enforcement mode to SHADOW for trial
            let _ = sqlx::query(
                r#"
                INSERT INTO system_enforcement_mode (enforcement_mode, description, enabled_by)
                VALUES ('SHADOW', 'Trial period - Shadow Mode enabled', NULL)
                ON CONFLICT (id) DO UPDATE SET
                    enforcement_mode = 'SHADOW',
                    description = 'Trial period - Shadow Mode enabled',
                    enabled_at = CURRENT_TIMESTAMP
                "#
            )
            .execute(&data.db_pool)
            .await;

            HttpResponse::Ok().json(SubscriptionResponse {
                id: subscription.id,
                company_id: subscription.company_id,
                subscription_type: subscription.subscription_type,
                status: subscription.status,
                trial_start_date: subscription.trial_start_date.map(|d| d.to_rfc3339()),
                trial_end_date: subscription.trial_end_date.map(|d| d.to_rfc3339()),
                days_remaining,
                monthly_price: subscription.monthly_price.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
                annual_price: subscription.annual_price.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            })
        }
        Err(e) => {
            eprintln!("Error starting trial: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to start trial"
            }))
        }
    }
}

/// Get current subscription for a company
#[utoipa::path(
    get,
    path = "/wizard/subscription/{company_id}",
    responses((status = 200, body = SubscriptionResponse))
)]
pub async fn get_subscription(
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
) -> impl Responder {
    let company_id = path.into_inner();
    let wizard_service = WizardService::new(data.db_pool.clone());
    
    match wizard_service.get_current_subscription(company_id).await {
        Ok(Some(subscription)) => {
            let days_remaining = subscription.trial_end_date
                .map(|end| {
                    let now = chrono::Utc::now();
                    if end > now {
                        (end - now).num_days() as i32
                    } else {
                        0
                    }
                });

            HttpResponse::Ok().json(SubscriptionResponse {
                id: subscription.id,
                company_id: subscription.company_id,
                subscription_type: subscription.subscription_type,
                status: subscription.status,
                trial_start_date: subscription.trial_start_date.map(|d| d.to_rfc3339()),
                trial_end_date: subscription.trial_end_date.map(|d| d.to_rfc3339()),
                days_remaining,
                monthly_price: subscription.monthly_price.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
                annual_price: subscription.annual_price.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            })
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "No active subscription found"
            }))
        }
        Err(e) => {
            eprintln!("Error getting subscription: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get subscription"
            }))
        }
    }
}

/// Upgrade from trial to paid subscription
#[utoipa::path(
    post,
    path = "/wizard/upgrade",
    request_body = UpgradeRequest,
    responses((status = 200, body = SubscriptionResponse))
)]
pub async fn upgrade_subscription(
    body: web::Json<UpgradeRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let wizard_service = WizardService::new(data.db_pool.clone());
    
    match wizard_service.upgrade_to_paid(
        body.company_id,
        &body.subscription_type,
        &body.billing_cycle,
    ).await {
        Ok(subscription) => {
            HttpResponse::Ok().json(SubscriptionResponse {
                id: subscription.id,
                company_id: subscription.company_id,
                subscription_type: subscription.subscription_type,
                status: subscription.status,
                trial_start_date: subscription.trial_start_date.map(|d| d.to_rfc3339()),
                trial_end_date: subscription.trial_end_date.map(|d| d.to_rfc3339()),
                days_remaining: None,
                monthly_price: subscription.monthly_price.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
                annual_price: subscription.annual_price.map(|d| d.to_string().parse::<f64>().unwrap_or(0.0)),
            })
        }
        Err(e) => {
            eprintln!("Error upgrading subscription: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to upgrade subscription"
            }))
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UpgradeRequest {
    pub company_id: Uuid,
    pub subscription_type: String,
    pub billing_cycle: String,
}

