use actix_web::{test, web, App};
use veridion_nexus::{AppState, health};
use veridion_nexus::routes::{
    log_action, get_logs, shred_data, data_subject_access, data_subject_export, 
    data_subject_rectify, approve_action, reject_action, 
    get_risk_assessment, get_all_risks, report_breach, get_breaches, 
    LogRequest, ShredRequest
};
use veridion_nexus::compliance_models::{
    DataSubjectRectificationRequest, HumanOversightResponse, DataBreachReport
};

// Helper to create test AppState with database
async fn create_test_state() -> AppState {
    std::env::set_var("VERIDION_MASTER_KEY", "test_master_key_for_testing_only_32_bytes_long");
    
    // Use test database or fallback to main database
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://veridion:veridion_password@localhost:5432/veridion_nexus".to_string());
    
    AppState::new(&database_url).await
        .expect("Failed to create test AppState. Make sure PostgreSQL is running.")
}

#[actix_web::test]
async fn test_health_endpoint() {
    let app = test::init_service(
        App::new().route("/health", web::get().to(health))
    ).await;
    
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "healthy");
    assert_eq!(body["service"], "veridion-nexus");
}

#[actix_web::test]
async fn test_log_action_basic() {
    let app_state = web::Data::new(create_test_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .service(web::scope("/api/v1")
                .service(web::resource("/log_action").route(web::post().to(log_action)))
            )
    ).await;
    
    let req_body = LogRequest {
        agent_id: "test-agent".to_string(),
        action: "Test Action".to_string(),
        payload: "test data".to_string(),
        target_region: Some("EU".to_string()),
        user_notified: None,
        notification_timestamp: None,
        user_id: None,
        requires_human_oversight: None,
    };
    
    let req = test::TestRequest::post()
        .uri("/api/v1/log_action")
        .set_json(&req_body)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "COMPLIANT");
    assert!(body["seal_id"].as_str().is_some());
}

#[actix_web::test]
async fn test_sovereign_lock_blocks_us() {
    let app_state = web::Data::new(create_test_state().await);
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .service(web::scope("/api/v1")
                .service(web::resource("/log_action").route(web::post().to(log_action)))
            )
    ).await;
    
    let req_body = LogRequest {
        agent_id: "test-agent".to_string(),
        action: "Test Action".to_string(),
        payload: "test data".to_string(),
        target_region: Some("US".to_string()),
        user_notified: None,
        notification_timestamp: None,
        user_id: None,
        requires_human_oversight: None,
    };
    
    let req = test::TestRequest::post()
        .uri("/api/v1/log_action")
        .set_json(&req_body)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "BLOCKED (SOVEREIGNTY)");
}

#[actix_web::test]
async fn test_data_subject_access() {
    
    
    let app_state = web::Data::new(create_test_state().await);
    
    // Najprv vytvoríme nejaké dáta
    let log_req = LogRequest {
        agent_id: "test-agent".to_string(),
        action: "Test Action".to_string(),
        payload: "test data".to_string(),
        target_region: Some("EU".to_string()),
        user_notified: None,
        notification_timestamp: None,
        user_id: Some("user-123".to_string()),
        requires_human_oversight: None,
    };
    
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .service(web::scope("/api/v1")
                .service(web::resource("/log_action").route(web::post().to(log_action)))
                .service(web::resource("/data_subject/{user_id}/access").route(web::get().to(data_subject_access)))
            )
    ).await;
    
    // Vytvoríme log entry
    let req = test::TestRequest::post()
        .uri("/api/v1/log_action")
        .set_json(&log_req)
        .to_request();
    let _ = test::call_service(&app, req).await;
    
    // Teraz testujeme data subject access
    let req = test::TestRequest::get()
        .uri("/api/v1/data_subject/user-123/access")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["format"], "json");
    assert!(body["records"].is_array());
}

#[actix_web::test]
async fn test_data_subject_export() {
    
    
    let app_state = web::Data::new(create_test_state().await);
    
    let log_req = LogRequest {
        agent_id: "test-agent".to_string(),
        action: "Test Action".to_string(),
        payload: "test data".to_string(),
        target_region: Some("EU".to_string()),
        user_notified: None,
        notification_timestamp: None,
        user_id: Some("user-456".to_string()),
        requires_human_oversight: None,
    };
    
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .service(web::scope("/api/v1")
                .service(web::resource("/log_action").route(web::post().to(log_action)))
                .service(web::resource("/data_subject/{user_id}/export").route(web::get().to(data_subject_export)))
            )
    ).await;
    
    // Vytvoríme log entry
    let req = test::TestRequest::post()
        .uri("/api/v1/log_action")
        .set_json(&log_req)
        .to_request();
    let _ = test::call_service(&app, req).await;
    
    // Testujeme export
    let req = test::TestRequest::get()
        .uri("/api/v1/data_subject/user-456/export")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_data_subject_rectify() {
    
    
    let app_state = web::Data::new(create_test_state().await);
    
    // Vytvoríme log entry
    let log_req = LogRequest {
        agent_id: "test-agent".to_string(),
        action: "Test Action".to_string(),
        payload: "test data".to_string(),
        target_region: Some("EU".to_string()),
        user_notified: None,
        notification_timestamp: None,
        user_id: Some("user-789".to_string()),
        requires_human_oversight: None,
    };
    
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .service(web::scope("/api/v1")
                .service(web::resource("/log_action").route(web::post().to(log_action)))
                .service(web::resource("/data_subject/{user_id}/rectify").route(web::put().to(data_subject_rectify)))
            )
    ).await;
    
    // Vytvoríme log entry
    let req = test::TestRequest::post()
        .uri("/api/v1/log_action")
        .set_json(&log_req)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    let seal_id = body["seal_id"].as_str().unwrap();
    
    // Testujeme rectification
    let rectify_req = DataSubjectRectificationRequest {
        user_id: "user-789".to_string(),
        seal_id: seal_id.to_string(),
        corrected_data: "Corrected action description".to_string(),
    };
    
    let req = test::TestRequest::put()
        .uri("/api/v1/data_subject/user-789/rectify")
        .set_json(&rectify_req)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_human_oversight_approve() {
    
    
    let app_state = web::Data::new(create_test_state().await);
    
    // Vytvoríme akciu s human oversight
    let log_req = LogRequest {
        agent_id: "test-agent".to_string(),
        action: "High Risk Action".to_string(),
        payload: "sensitive data".to_string(),
        target_region: Some("EU".to_string()),
        user_notified: None,
        notification_timestamp: None,
        user_id: None,
        requires_human_oversight: Some(true),
    };
    
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .service(web::scope("/api/v1")
                .service(web::resource("/log_action").route(web::post().to(log_action)))
                .service(web::resource("/action/{seal_id}/approve").route(web::post().to(approve_action)))
            )
    ).await;
    
    // Vytvoríme log entry
    let req = test::TestRequest::post()
        .uri("/api/v1/log_action")
        .set_json(&log_req)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    let seal_id = body["seal_id"].as_str().unwrap();
    assert_eq!(body["human_oversight_status"], "PENDING");
    
    // Teraz schválime akciu
    let approve_req = HumanOversightResponse {
        status: "APPROVED".to_string(),
        reviewer_id: Some("reviewer-001".to_string()),
        decided_at: "2024-01-15 14:35:00".to_string(),
        comments: Some("Approved".to_string()),
    };
    
    // Escape seal_id for URI
    let seal_id_escaped = urlencoding::encode(seal_id);
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/action/{}/approve", seal_id_escaped))
        .set_json(&approve_req)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_human_oversight_reject() {
    
    
    let app_state = web::Data::new(create_test_state().await);
    
    let log_req = LogRequest {
        agent_id: "test-agent".to_string(),
        action: "High Risk Action".to_string(),
        payload: "sensitive data".to_string(),
        target_region: Some("EU".to_string()),
        user_notified: None,
        notification_timestamp: None,
        user_id: None,
        requires_human_oversight: Some(true),
    };
    
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .service(web::scope("/api/v1")
                .service(web::resource("/log_action").route(web::post().to(log_action)))
                .service(web::resource("/action/{seal_id}/reject").route(web::post().to(reject_action)))
            )
    ).await;
    
    // Vytvoríme log entry
    let req = test::TestRequest::post()
        .uri("/api/v1/log_action")
        .set_json(&log_req)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    let seal_id = body["seal_id"].as_str().unwrap();
    
    // Teraz zamietneme akciu
    let reject_req = HumanOversightResponse {
        status: "REJECTED".to_string(),
        reviewer_id: Some("reviewer-001".to_string()),
        decided_at: "2024-01-15 14:35:00".to_string(),
        comments: Some("Rejected - too risky".to_string()),
    };
    
    // Escape seal_id for URI
    let seal_id_escaped = urlencoding::encode(seal_id);
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/action/{}/reject", seal_id_escaped))
        .set_json(&reject_req)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_risk_assessment() {
    
    
    let app_state = web::Data::new(create_test_state().await);
    
    let log_req = LogRequest {
        agent_id: "test-agent".to_string(),
        action: "Credit Check".to_string(),
        payload: "credit data".to_string(),
        target_region: Some("EU".to_string()),
        user_notified: None,
        notification_timestamp: None,
        user_id: None,
        requires_human_oversight: None,
    };
    
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .service(web::scope("/api/v1")
                .service(web::resource("/log_action").route(web::post().to(log_action)))
                .service(web::resource("/risk_assessment/{seal_id}").route(web::get().to(get_risk_assessment)))
            )
    ).await;
    
    // Vytvoríme log entry
    let req = test::TestRequest::post()
        .uri("/api/v1/log_action")
        .set_json(&log_req)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    let seal_id = body["seal_id"].as_str().unwrap();
    let risk_level = body["risk_level"].as_str().unwrap();
    
    // Overíme, že risk level je HIGH pre credit check
    assert_eq!(risk_level, "HIGH");
    
    // Testujeme risk assessment endpoint
    let seal_id_escaped = urlencoding::encode(seal_id);
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/risk_assessment/{}", seal_id_escaped))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let risk_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(risk_body["risk_level"], "HIGH");
    assert!(risk_body["risk_factors"].is_array());
}

#[actix_web::test]
async fn test_get_all_risks() {
    
    
    let app_state = web::Data::new(create_test_state().await);
    
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .service(web::scope("/api/v1")
                .service(web::resource("/log_action").route(web::post().to(log_action)))
                .service(web::resource("/risks").route(web::get().to(get_all_risks)))
            )
    ).await;
    
    // Vytvoríme niekoľko log entries
    for i in 0..3 {
        let log_req = LogRequest {
            agent_id: format!("test-agent-{}", i),
            action: "Test Action".to_string(),
            payload: "test data".to_string(),
            target_region: Some("EU".to_string()),
            user_notified: None,
            notification_timestamp: None,
            user_id: None,
            requires_human_oversight: None,
        };
        
        let req = test::TestRequest::post()
            .uri("/api/v1/log_action")
            .set_json(&log_req)
            .to_request();
        let _ = test::call_service(&app, req).await;
    }
    
    // Získame všetky risks
    let req = test::TestRequest::get()
        .uri("/api/v1/risks")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let risks: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert!(risks.len() >= 3);
}

#[actix_web::test]
async fn test_data_breach_reporting() {
    
    
    let app_state = web::Data::new(create_test_state().await);
    
    let breach_req = DataBreachReport {
        description: "Unauthorized access detected".to_string(),
        breach_type: "UNAUTHORIZED_ACCESS".to_string(),
        affected_users: vec!["user-123".to_string(), "user-456".to_string()],
        detected_at: "2024-01-15 14:30:00".to_string(),
        affected_records_count: Some(42),
    };
    
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .service(web::scope("/api/v1")
                .service(web::resource("/breach_report").route(web::post().to(report_breach)))
                .service(web::resource("/breaches").route(web::get().to(get_breaches)))
            )
    ).await;
    
    // Nahlásime breach
    let req = test::TestRequest::post()
        .uri("/api/v1/breach_report")
        .set_json(&breach_req)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "REPORTED");
    assert!(body["breach_id"].as_str().is_some());
    
    // Získame všetky breaches
    let req = test::TestRequest::get()
        .uri("/api/v1/breaches")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let breaches: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert!(!breaches.is_empty());
}

#[actix_web::test]
async fn test_gdpr_shred_data() {
    
    
    let app_state = web::Data::new(create_test_state().await);
    
    // Vytvoríme log entry
    let log_req = LogRequest {
        agent_id: "test-agent".to_string(),
        action: "Test Action".to_string(),
        payload: "test data".to_string(),
        target_region: Some("EU".to_string()),
        user_notified: None,
        notification_timestamp: None,
        user_id: None,
        requires_human_oversight: None,
    };
    
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .service(web::scope("/api/v1")
                .service(web::resource("/log_action").route(web::post().to(log_action)))
                .service(web::resource("/shred_data").route(web::post().to(shred_data)))
            )
    ).await;
    
    // Vytvoríme log entry
    let req = test::TestRequest::post()
        .uri("/api/v1/log_action")
        .set_json(&log_req)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    let seal_id = body["seal_id"].as_str().unwrap();
    
    // Vymažeme dáta
    let shred_req = ShredRequest {
        seal_id: seal_id.to_string(),
    };
    
    let req = test::TestRequest::post()
        .uri("/api/v1/shred_data")
        .set_json(&shred_req)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "SUCCESS");
}

#[actix_web::test]
async fn test_get_logs() {
    
    
    let app_state = web::Data::new(create_test_state().await);
    
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .service(web::scope("/api/v1")
                .service(web::resource("/log_action").route(web::post().to(log_action)))
                .service(web::resource("/logs").route(web::get().to(get_logs)))
            )
    ).await;
    
    // Vytvoríme log entry
    let log_req = LogRequest {
        agent_id: "test-agent".to_string(),
        action: "Test Action".to_string(),
        payload: "test data".to_string(),
        target_region: Some("EU".to_string()),
        user_notified: None,
        notification_timestamp: None,
        user_id: None,
        requires_human_oversight: None,
    };
    
    let req = test::TestRequest::post()
        .uri("/api/v1/log_action")
        .set_json(&log_req)
        .to_request();
    let _ = test::call_service(&app, req).await;
    
    // Získame logs
    let req = test::TestRequest::get()
        .uri("/api/v1/logs")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let logs: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert!(!logs.is_empty());
}


