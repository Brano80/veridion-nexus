use actix_web::{web, App, HttpResponse, HttpServer, Responder, middleware};
use actix_web::middleware::Compress;
use actix_cors::Cors;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use security::{SecurityHeaders, RateLimit, RateLimitConfig};

mod api_state;
mod routes;
mod compliance_models;
mod database;
mod models;
mod background_worker;
mod security;
mod module_service;
mod deployment;
mod services;

// Core Runtime Compliance Engine (mandatory)
mod core;
// Operational Modules (optional)
mod modules;
// Integration Layer (SDKs, Webhooks, API)
mod integration;

use routes::*;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::log_action,
        routes::get_logs,
        routes::shred_data,
        routes::download_report,
        routes::revoke_access,
        routes::data_subject_access,
        routes::data_subject_export,
        routes::data_subject_rectify,
        routes::request_processing_restriction,
        routes::lift_processing_restriction,
        routes::get_processing_restrictions,
        routes::request_processing_objection,
        routes::withdraw_processing_objection,
        routes::reject_processing_objection,
        routes::get_processing_objections,
        routes::request_human_review,
        routes::appeal_automated_decision,
        routes::get_automated_decisions,
        routes::require_human_oversight,
        routes::approve_action,
        routes::reject_action,
        routes::get_risk_assessment,
        routes::get_all_risks,
        routes::report_breach,
        routes::get_breaches,
        routes::grant_consent,
        routes::withdraw_consent,
        routes::get_user_consents,
        routes::create_dpia,
        routes::update_dpia,
        routes::get_all_dpias,
        routes::create_retention_policy,
        routes::assign_retention_policy,
        routes::get_retention_status,
        routes::get_all_retention_policies,
        routes::execute_retention_deletions,
        routes::create_monitoring_event,
        routes::update_event_resolution,
        routes::get_all_monitoring_events,
        routes::get_system_health,
        routes::export_ai_bom,
        routes::register_ai_system,
        routes::register_webhook,
        routes::list_webhooks,
        routes::update_webhook,
        routes::delete_webhook,
        routes::get_webhook_deliveries,
        routes::api_keys::create_api_key,
        routes::api_keys::list_api_keys,
        routes::api_keys::get_api_key,
        routes::api_keys::revoke_api_key,
        routes::modules::list_modules,
        routes::modules::enable_module,
        routes::modules::disable_module,
        routes::modules::get_module_status,
        routes::modules::get_modules_by_regulation,
        routes::modules::get_module_config_schema,
        routes::modules::get_company_module_config,
        routes::modules::set_company_module_config,
        routes::gdpr_article_12::list_privacy_notices,
        routes::gdpr_article_12::create_privacy_notice,
        routes::gdpr_article_12::get_privacy_notice,
        routes::gdpr_article_12::update_privacy_notice,
        routes::gdpr_article_12::publish_privacy_notice,
        routes::gdpr_article_12::get_privacy_notice_templates,
        routes::proxy_request,
        routes::simulate_policy,
        routes::rollback_policy,
        routes::get_policy_impact_analytics,
        routes::get_shadow_mode_analytics,
        routes::get_circuit_breaker_analytics,
        routes::get_canary_analytics,
        routes::get_tprm_compliance_report,
        routes::get_dora_compliance_report,
        routes::get_nis2_compliance_report,
        routes::get_vendor_risk_dashboard,
        routes::get_policy_health_dashboard,
        routes::get_policy_health_trends,
        routes::get_approval_queue,
        routes::get_approval_history,
        routes::get_rollback_history,
        routes::create_delegation,
        routes::list_delegations,
        routes::revoke_delegation,
        routes::get_business_function_dashboard,
        routes::create_or_update_asset,
        routes::list_assets,
        routes::get_asset_by_agent,
        routes::list_business_functions,
        routes::create_asset_policy,
        routes::list_asset_policies,
        routes::get_enforcement_mode,
        routes::set_enforcement_mode,
        routes::configure_circuit_breaker,
        routes::configure_canary,
        routes::preview_policy_impact,
        routes::get_policy_health,
        routes::approve_policy,
        routes::reject_policy,
        routes::get_vendor_risk_score,
        routes::enrich_asset_tprm,
        routes::auto_generate_tprm_policies,
        routes::get_executive_assurance,
        routes::get_compliance_kpis,
        routes::get_decision_explanation,
        routes::get_feature_importance,
        routes::get_model_drift,
        routes::create_baseline,
        routes::detect_configuration_drift,
        routes::get_configuration_drifts,
        routes::register_cloud_provider,
        routes::sync_cloud_compliance,
        routes::get_cloud_compliance_summary,
        routes::wizard::create_company_profile,
        routes::wizard::get_company_profile,
        routes::wizard::recommend_modules,
        routes::wizard::calculate_price,
        routes::wizard::start_trial,
        routes::wizard::get_subscription,
        routes::wizard::upgrade_subscription,
    ),
    components(schemas(
        routes::LogRequest,
        routes::LogResponse,
        routes::ShredRequest,
        crate::integration::proxy::ProxyRequest,
        compliance_models::RiskAssessment,
        compliance_models::HumanOversightRequest,
        compliance_models::HumanOversightResponse,
        compliance_models::DataSubjectAccessRequest,
        compliance_models::DataSubjectAccessResponse,
        compliance_models::DataSubjectRecord,
        compliance_models::DataSubjectRectificationRequest,
        compliance_models::ProcessingRestrictionRequest,
        compliance_models::ProcessingRestrictionResponse,
        compliance_models::LiftRestrictionRequest,
        compliance_models::RestrictionsResponse,
        compliance_models::ProcessingObjectionRequest,
        compliance_models::ProcessingObjectionResponse,
        compliance_models::WithdrawObjectionRequest,
        compliance_models::RejectObjectionRequest,
        compliance_models::ObjectionsResponse,
        compliance_models::AutomatedDecisionResponse,
        compliance_models::RequestReviewRequest,
        compliance_models::RequestReviewResponse,
        compliance_models::AppealDecisionRequest,
        compliance_models::AutomatedDecisionsResponse,
        compliance_models::DataBreachReport,
        compliance_models::DataBreachResponse,
        compliance_models::ConsentRequest,
        compliance_models::ConsentResponse,
        compliance_models::WithdrawConsentRequest,
        compliance_models::UserConsentsResponse,
        compliance_models::DpiaRequest,
        compliance_models::DpiaResponse,
        compliance_models::UpdateDpiaRequest,
        compliance_models::DpiasResponse,
        compliance_models::RetentionPolicyRequest,
        compliance_models::RetentionPolicyResponse,
        compliance_models::AssignRetentionRequest,
        compliance_models::RetentionStatusResponse,
        compliance_models::RetentionPoliciesResponse,
        routes::auth::LoginRequest,
        routes::auth::LoginResponse,
        routes::auth::RegisterRequest,
        routes::auth::RegisterResponse,
        routes::auth::UserResponse,
        routes::api_keys::CreateApiKeyRequest,
        routes::api_keys::CreateApiKeyResponse,
        routes::api_keys::ApiKeyInfoResponse,
        routes::api_keys::ApiKeysListResponse,
        routes::modules::ModuleInfo,
        routes::modules::ModulesListResponse,
        routes::modules::EnableModuleRequest,
        routes::modules::ModuleResponse,
        routes::RollbackRequest,
        routes::PolicyImpactAnalytics,
        routes::AgentStats,
        routes::RiskAssessmentSummary,
        routes::ShadowModeAnalytics,
        routes::AgentShadowStats,
        routes::RegionShadowStats,
        routes::PolicyShadowStats,
        routes::TimeRange,
        routes::CircuitBreakerAnalytics,
        routes::CircuitBreakerTransition,
        routes::PolicyCircuitBreakerStatus,
        routes::CanaryAnalytics,
        routes::PolicyCanaryStatus,
        routes::CanaryTransition,
        routes::TPRMComplianceReport,
        routes::VendorComplianceInfo,
        routes::BusinessFunctionDashboard,
        routes::BusinessFunctionStats,
        routes::DORAComplianceReport,
        routes::DORAIncidentReporting,
        routes::DORAIncident,
        routes::DORAResilienceTesting,
        routes::DORATestResult,
        routes::NIS2ComplianceReport,
        routes::NIS2ManagementAccountability,
        routes::NIS2BaselineMeasures,
        routes::NIS2Measure,
        routes::NIS2IncidentReporting,
        routes::PolicyHealthDashboard,
        routes::PolicyHealthSummary,
        routes::PolicyHealthTrends,
        routes::HealthTrendPoint,
        routes::ApprovalQueueDashboard,
        routes::PendingApproval,
        routes::ApprovedPolicy,
        routes::RejectedPolicy,
        routes::ApproverInfo,
        routes::ApprovalHistory,
        routes::ApprovalHistoryEntry,
        routes::RollbackHistoryDashboard,
        routes::RollbackHistoryEntry,
        routes::CreateDelegationRequest,
        routes::DelegationResponse,
        routes::DelegationListResponse,
        crate::core::policy_simulator::SimulationRequest,
        crate::core::policy_simulator::SimulationResult,
        routes::AssetRequest,
        routes::AssetResponse,
        routes::AssetsListResponse,
        routes::AssetDetailResponse,
        routes::AssetPolicyRequest,
        routes::AssetPolicyResponse,
        routes::EnforcementModeResponse,
        routes::SetEnforcementModeRequest,
        routes::CircuitBreakerConfigRequest,
        routes::CircuitBreakerConfigResponse,
        routes::CanaryConfigRequest,
        routes::CanaryConfigResponse,
        routes::PolicyHealthResponse,
        routes::PolicyApprovalRequest,
        routes::PolicyApprovalResponse,
        routes::PolicyRejectionRequest,
        routes::VendorRiskScoreResponse,
        routes::AutoGenerateTPRMPolicyRequest,
        routes::TPRMPolicyRecommendationsResponse,
        routes::TPRMPolicyRecommendation,
        routes::ExecutiveScorecardResponse,
        routes::ComplianceKPIResponse,
        routes::AIDecisionExplanationResponse,
        routes::FeatureImportanceResponse,
        routes::ModelDriftResponse,
        routes::CreateBaselineRequest,
        routes::BaselineResponse,
        routes::DetectDriftRequest,
        routes::DriftDetectionResponse,
        routes::DriftResponse,
        routes::RegisterCloudProviderRequest,
        routes::CloudProviderResponse,
        routes::CloudSyncResponse,
        routes::CloudComplianceSummaryResponse,
        routes::wizard::CompanyProfileResponse,
        routes::wizard::RecommendModulesRequest,
        routes::wizard::CalculatePriceRequest,
        routes::wizard::SubscriptionResponse,
        routes::wizard::UpgradeRequest,
        crate::services::wizard_service::ModuleRecommendationResponse,
        crate::services::wizard_service::RecommendedModule,
        crate::services::wizard_service::PricingBreakdown,
        routes::gdpr_article_12::PrivacyNoticeResponse,
        routes::gdpr_article_12::PrivacyNoticesListResponse,
    ))
)]
struct ApiDoc;

/// Health check endpoint
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "veridion-nexus",
        "version": "1.0.0"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    println!("🚀 Veridion Nexus API starting on port 8080");
    println!("📚 Swagger UI available at: http://localhost:8080/swagger-ui/");

    // Get database URL from environment (required, no default)
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set. See .env.example for configuration.");

    println!("📊 Connecting to database...");
    let app_state = match api_state::AppState::new(&database_url).await {
        Ok(state) => {
            println!("✅ Database connected successfully");
            state
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to database: {}", e);
            eprintln!("💡 Make sure PostgreSQL is running and DATABASE_URL is set correctly");
            std::process::exit(1);
        }
    };

    let app_state = web::Data::new(app_state);

    // Start background workers
    let db_pool_for_worker = app_state.db_pool.clone();
    let worker = background_worker::BackgroundWorker::new(db_pool_for_worker.clone());
    tokio::spawn(async move {
        worker.process_webhook_deliveries().await;
    });
    
    let db_pool_for_retention = app_state.db_pool.clone();
    let worker2 = background_worker::BackgroundWorker::new(db_pool_for_retention.clone());
    tokio::spawn(async move {
        worker2.process_retention_deletions().await;
    });
    
    let db_pool_for_views = app_state.db_pool.clone();
    let worker3 = background_worker::BackgroundWorker::new(db_pool_for_views);
    tokio::spawn(async move {
        worker3.refresh_materialized_views().await;
    });

    // Initialize security services
    let rate_limiter = RateLimit::new(RateLimitConfig {
        requests_per_minute: 100,
        window_seconds: 60,
    });

    HttpServer::new(move || {
        // CORS configuration - SECURITY: Never allow * in production
        let allowed_origins = std::env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| {
                // In production, this should be set explicitly
                if std::env::var("RUST_ENV").unwrap_or_default() == "production" {
                    panic!("ALLOWED_ORIGINS must be set in production environment");
                }
                "*".to_string() // Only allow * in development
            });
        
        let cors = if allowed_origins == "*" {
            // Development mode only - allow any origin
            // SECURITY WARNING: This should never be used in production
            if std::env::var("RUST_ENV").unwrap_or_default() == "production" {
                panic!("CORS wildcard (*) is not allowed in production. Set ALLOWED_ORIGINS to specific origins.");
            }
            Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600)
        } else {
            // Production mode - specific origins only
            let origins: Vec<&str> = allowed_origins.split(',').map(|s| s.trim()).collect();
            if origins.is_empty() {
                panic!("ALLOWED_ORIGINS must contain at least one origin");
            }
            let mut cors_builder = Cors::default()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600);
            
            for origin in origins {
                cors_builder = cors_builder.allowed_origin(origin);
            }
            cors_builder
        };

        App::new()
            .app_data(app_state.clone())
            // SECURITY: Set request payload size limit (10MB) to prevent DoS
            .app_data(web::JsonConfig::default().limit(10_485_760))
            // Security middleware (order matters!)
            .wrap(cors)
            .wrap(SecurityHeaders)
            .wrap(rate_limiter.clone())
            // Response compression (Priority 3: Performance Optimization)
            .wrap(Compress::default())
            // Enable request logging
            .wrap(middleware::Logger::default())
            .route("/health", web::get().to(health))
            .service(
                web::scope("/api/v1")
                    // Auth endpoints (public)
                    .service(web::resource("/auth/login").route(web::post().to(routes::auth::login)))
                    .service(web::resource("/auth/register").route(web::post().to(routes::auth::register)))
                    .service(web::resource("/auth/me").route(web::get().to(routes::auth::get_me)))
                    // Existing endpoints
                    .service(web::resource("/log_action").route(web::post().to(log_action)))
                    .service(web::resource("/logs").route(web::get().to(get_logs)))
                    .service(web::resource("/shred_data").route(web::post().to(shred_data)))
                    .service(web::resource("/download_report").route(web::get().to(download_report)))
                    .service(web::resource("/revoke_access").route(web::post().to(revoke_access)))
                    // Priority 1: Data Subject Rights
                    .service(web::resource("/data_subject/{user_id}/access").route(web::get().to(data_subject_access)))
                    .service(web::resource("/data_subject/{user_id}/export").route(web::get().to(data_subject_export)))
                    .service(web::resource("/data_subject/{user_id}/rectify").route(web::put().to(data_subject_rectify)))
                    // GDPR Article 18: Processing Restrictions
                    .service(web::resource("/data_subject/{user_id}/restrict").route(web::post().to(request_processing_restriction)))
                    .service(web::resource("/data_subject/{user_id}/lift_restriction").route(web::post().to(lift_processing_restriction)))
                    .service(web::resource("/data_subject/{user_id}/restrictions").route(web::get().to(get_processing_restrictions)))
                    // GDPR Article 21: Processing Objections
                    .service(web::resource("/data_subject/{user_id}/object").route(web::post().to(request_processing_objection)))
                    .service(web::resource("/data_subject/{user_id}/withdraw_objection").route(web::post().to(withdraw_processing_objection)))
                    .service(web::resource("/data_subject/{user_id}/reject_objection").route(web::post().to(reject_processing_objection)))
                    .service(web::resource("/data_subject/{user_id}/objections").route(web::get().to(get_processing_objections)))
                    // GDPR Article 22: Automated Decision-Making
                    .service(web::resource("/data_subject/{user_id}/request_review").route(web::post().to(request_human_review)))
                    .service(web::resource("/data_subject/{user_id}/appeal_decision").route(web::post().to(appeal_automated_decision)))
                    .service(web::resource("/data_subject/{user_id}/automated_decisions").route(web::get().to(get_automated_decisions)))
                    // Priority 1: Human Oversight
                    .service(web::resource("/action/{seal_id}/require_approval").route(web::post().to(require_human_oversight)))
                    .service(web::resource("/action/{seal_id}/approve").route(web::post().to(approve_action)))
                    .service(web::resource("/action/{seal_id}/reject").route(web::post().to(reject_action)))
                    // Priority 1: Risk Assessment
                    .service(web::resource("/risk_assessment/{seal_id}").route(web::get().to(get_risk_assessment)))
                    .service(web::resource("/risks").route(web::get().to(get_all_risks)))
                    // Priority 1: Data Breach
                    .service(web::resource("/breach_report").route(web::post().to(report_breach)))
                    .service(web::resource("/breaches").route(web::get().to(get_breaches)))
                    // Priority 2: Consent Management
                    .service(web::resource("/consent").route(web::post().to(grant_consent)))
                    .service(web::resource("/consent/withdraw").route(web::post().to(withdraw_consent)))
                    .service(web::resource("/consent/{user_id}").route(web::get().to(get_user_consents)))
                    // Priority 2: DPIA Tracking
                    .service(web::resource("/dpia").route(web::post().to(create_dpia)))
                    .service(web::resource("/dpia/{dpia_id}").route(web::put().to(update_dpia)))
                    .service(web::resource("/dpias").route(web::get().to(get_all_dpias)))
                    // Priority 2: Retention Period Automation
                    .service(web::resource("/retention/policy").route(web::post().to(create_retention_policy)))
                    .service(web::resource("/retention/assign").route(web::post().to(assign_retention_policy)))
                    .service(web::resource("/retention/status/{record_type}/{record_id}").route(web::get().to(get_retention_status)))
                    .service(web::resource("/retention/policies").route(web::get().to(get_all_retention_policies)))
                    .service(web::resource("/retention/execute_deletions").route(web::post().to(execute_retention_deletions)))
                    // Priority 2: Post-Market Monitoring
                    .service(web::resource("/monitoring/event").route(web::post().to(create_monitoring_event)))
                    .service(web::resource("/monitoring/event/{event_id}").route(web::put().to(update_event_resolution)))
                    .service(web::resource("/monitoring/events").route(web::get().to(get_all_monitoring_events)))
                    .service(web::resource("/monitoring/health/{system_id}").route(web::get().to(get_system_health)))
                    // AI-BOM Export (CycloneDX)
                    .service(web::resource("/ai_bom/{system_id}").route(web::get().to(export_ai_bom)))
                    .service(web::resource("/ai_bom/inventory").route(web::post().to(register_ai_system)))
                    // Webhook Support
                    .service(web::resource("/webhooks").route(web::post().to(register_webhook)).route(web::get().to(list_webhooks)))
                    .service(web::resource("/webhooks/{id}").route(web::put().to(update_webhook)).route(web::delete().to(delete_webhook)))
                    .service(web::resource("/webhooks/{id}/deliveries").route(web::get().to(get_webhook_deliveries)))
                    // API Key Management
                    .service(web::resource("/api_keys").route(web::post().to(routes::api_keys::create_api_key)).route(web::get().to(routes::api_keys::list_api_keys)))
                    .service(web::resource("/api_keys/{id}").route(web::get().to(routes::api_keys::get_api_key)).route(web::delete().to(routes::api_keys::revoke_api_key)))
                    // Module Management
                    .service(web::resource("/modules").route(web::get().to(routes::modules::list_modules)))
                    .service(web::resource("/modules/{name}/enable").route(web::post().to(routes::modules::enable_module)))
                    .service(web::resource("/modules/{name}/disable").route(web::post().to(routes::modules::disable_module)))
                    .service(web::resource("/modules/{name}/status").route(web::get().to(routes::modules::get_module_status)))
                    // Enhanced Module System
                    .service(web::resource("/modules/by-regulation/{regulation}").route(web::get().to(routes::modules::get_modules_by_regulation)))
                    .service(web::resource("/modules/{name}/config-schema").route(web::get().to(routes::modules::get_module_config_schema)))
                    .service(web::resource("/companies/{company_id}/modules/{module_name}/config").route(web::get().to(routes::modules::get_company_module_config)).route(web::post().to(routes::modules::set_company_module_config)))
                    // Phase 1: GDPR Article 12 - Transparent Information
                    .service(web::resource("/modules/gdpr-article-12/notices").route(web::get().to(routes::gdpr_article_12::list_privacy_notices)).route(web::post().to(routes::gdpr_article_12::create_privacy_notice)))
                    .service(web::resource("/modules/gdpr-article-12/notices/{id}").route(web::get().to(routes::gdpr_article_12::get_privacy_notice)).route(web::put().to(routes::gdpr_article_12::update_privacy_notice)))
                    .service(web::resource("/modules/gdpr-article-12/notices/{id}/publish").route(web::post().to(routes::gdpr_article_12::publish_privacy_notice)))
                    .service(web::resource("/modules/gdpr-article-12/templates").route(web::get().to(routes::gdpr_article_12::get_privacy_notice_templates)))
                    // Priority 2: User Notification Preferences (EU AI Act Article 13)
                    .service(web::resource("/user/{user_id}/notification_preferences").route(web::post().to(routes::set_notification_preferences)).route(web::get().to(routes::get_notification_preferences)))
                    .service(web::resource("/user/{user_id}/notifications").route(web::get().to(routes::get_user_notifications)))
                    // Priority 3: GDPR Article 30 - Records of Processing Activities
                    .service(web::resource("/processing_records").route(web::get().to(routes::get_processing_records)))
                    // Priority 3: EU AI Act Article 8 - Conformity Assessment
                    .service(web::resource("/conformity_assessments").route(web::post().to(routes::create_conformity_assessment)).route(web::get().to(routes::get_conformity_assessments)))
                    // Priority 3: EU AI Act Article 11 - Data Governance Extension
                    .service(web::resource("/data_quality/metrics").route(web::post().to(routes::record_data_quality_metric)))
                    .service(web::resource("/data_quality/bias").route(web::post().to(routes::record_data_bias)))
                    .service(web::resource("/data_quality/lineage").route(web::post().to(routes::record_data_lineage)))
                    .service(web::resource("/data_quality/report/{seal_id}").route(web::get().to(routes::get_data_quality_report)))
                    .service(web::resource("/modules/{name}/status").route(web::get().to(routes::modules::get_module_status)))
                    
                    // Wizard endpoints
                    .service(web::resource("/wizard/company-profile").route(web::post().to(routes::wizard::create_company_profile)))
                    .service(web::resource("/wizard/company-profile/{company_id}").route(web::get().to(routes::wizard::get_company_profile)))
                    .service(web::resource("/wizard/recommend-modules").route(web::post().to(routes::wizard::recommend_modules)))
                    .service(web::resource("/wizard/calculate-price").route(web::post().to(routes::wizard::calculate_price)))
                    .service(web::resource("/wizard/start-trial").route(web::post().to(routes::wizard::start_trial)))
                    .service(web::resource("/wizard/subscription/{company_id}").route(web::get().to(routes::wizard::get_subscription)))
                    .service(web::resource("/wizard/upgrade").route(web::post().to(routes::wizard::upgrade_subscription)))
                    // Proxy Mode - Network-level compliance enforcement
                    .service(web::resource("/proxy").route(web::post().to(routes::proxy_request)))
                    // Policy Management - Operational Safety
                    .service(web::resource("/policies/simulate").route(web::post().to(routes::simulate_policy)))
                    .service(web::resource("/policies/preview-impact").route(web::get().to(routes::preview_policy_impact)))
                    .service(web::resource("/policies/compare").route(web::post().to(routes::compare_policies)))
                    .service(web::resource("/policies/{policy_id}/rollback").route(web::post().to(routes::rollback_policy)))
                    .service(web::resource("/policies/{policy_id}/health").route(web::get().to(routes::get_policy_health)))
                    .service(web::resource("/policies/{policy_id}/approve").route(web::post().to(routes::approve_policy)))
                    .service(web::resource("/policies/{policy_id}/reject").route(web::post().to(routes::reject_policy)))
                    .service(web::resource("/analytics/policy-impact").route(web::get().to(routes::get_policy_impact_analytics)))
                    .service(web::resource("/analytics/shadow-mode").route(web::get().to(routes::get_shadow_mode_analytics)))
                    .service(web::resource("/analytics/circuit-breaker").route(web::get().to(routes::get_circuit_breaker_analytics)))
                    .service(web::resource("/analytics/canary").route(web::get().to(routes::get_canary_analytics)))
                    .service(web::resource("/analytics/vendor-risk").route(web::get().to(routes::get_vendor_risk_dashboard)))
                    .service(web::resource("/analytics/business-functions").route(web::get().to(routes::get_business_function_dashboard)))
                    .service(web::resource("/analytics/policy-health").route(web::get().to(routes::get_policy_health_dashboard)))
                    .service(web::resource("/analytics/policy-health/{policy_id}/trends").route(web::get().to(routes::get_policy_health_trends)))
                    .service(web::resource("/approvals/queue").route(web::get().to(routes::get_approval_queue)))
                    .service(web::resource("/approvals/{policy_id}/history").route(web::get().to(routes::get_approval_history)))
                    .service(web::resource("/approvals/delegations").route(web::post().to(routes::create_delegation)).route(web::get().to(routes::list_delegations)))
                    .service(web::resource("/approvals/delegations/{delegation_id}").route(web::delete().to(routes::revoke_delegation)))
                    .service(web::resource("/analytics/rollback-history").route(web::get().to(routes::get_rollback_history)))
                    // Veridion TPRM Integration
                    .service(web::resource("/vendors/{vendor_domain}/risk-score").route(web::get().to(routes::get_vendor_risk_score)))
                    .service(web::resource("/assets/{asset_id}/enrich-tprm").route(web::post().to(routes::enrich_asset_tprm)))
                    .service(web::resource("/policies/auto-generate-from-tprm").route(web::post().to(routes::auto_generate_tprm_policies)))
                    // Executive Assurance Reporting
                    .service(web::resource("/reports/executive-assurance").route(web::get().to(routes::get_executive_assurance)))
                    .service(web::resource("/reports/compliance-kpis").route(web::get().to(routes::get_compliance_kpis)))
                    .service(web::resource("/reports/tprm-compliance").route(web::get().to(routes::get_tprm_compliance_report)))
                    .service(web::resource("/reports/dora-compliance").route(web::get().to(routes::get_dora_compliance_report)))
                    .service(web::resource("/reports/nis2-compliance").route(web::get().to(routes::get_nis2_compliance_report)))
                    // AI Explainability & Observability
                    .service(web::resource("/models/{model_id}/explanations/{decision_id}").route(web::get().to(routes::get_decision_explanation)))
                    .service(web::resource("/models/{model_id}/feature-importance").route(web::get().to(routes::get_feature_importance)))
                    .service(web::resource("/models/{model_id}/drift").route(web::get().to(routes::get_model_drift)))
                    // Configuration Drift Detection
                    .service(web::resource("/configuration/baselines").route(web::post().to(routes::create_baseline)))
                    .service(web::resource("/configuration/baselines/{baseline_id}/detect-drift").route(web::post().to(routes::detect_configuration_drift)))
                    .service(web::resource("/configuration/baselines/{baseline_id}/drifts").route(web::get().to(routes::get_configuration_drifts)))
                    // Multi-Cloud Native Integrations
                    .service(web::resource("/cloud/providers").route(web::post().to(routes::register_cloud_provider)))
                    .service(web::resource("/cloud/providers/{provider}/sync").route(web::post().to(routes::sync_cloud_compliance)))
                    .service(web::resource("/cloud/providers/{provider}/compliance").route(web::get().to(routes::get_cloud_compliance_summary)))
                    // Asset Management
                    .service(web::resource("/assets").route(web::post().to(routes::create_or_update_asset)).route(web::get().to(routes::list_assets)))
                    .service(web::resource("/assets/by-agent/{agent_id}").route(web::get().to(routes::get_asset_by_agent)))
                    .service(web::resource("/business-functions").route(web::get().to(routes::list_business_functions)))
                    .service(web::resource("/asset-policies").route(web::post().to(routes::create_asset_policy)).route(web::get().to(routes::list_asset_policies)))
                    // System Configuration - Shadow Mode
                    .service(web::resource("/system/enforcement-mode").route(web::get().to(routes::get_enforcement_mode)).route(web::post().to(routes::set_enforcement_mode)))
                    // Circuit Breaker Configuration
                    .service(web::resource("/policies/{policy_id}/circuit-breaker/config").route(web::post().to(routes::configure_circuit_breaker)))
                    // Canary Deployment Configuration
                    .service(web::resource("/policies/{policy_id}/canary-config").route(web::post().to(routes::configure_canary)))
            )
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", ApiDoc::openapi())
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
