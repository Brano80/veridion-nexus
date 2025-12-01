use veridion_nexus::api_state::AppState;
use veridion_nexus::routes::{log_action, get_logs, download_report, LogActionRequest, LogActionResponse};
use veridion_nexus::annex_iv_compiler::ComplianceRecord;
use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use dotenv::dotenv;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// OpenAPI documentation for Veridion Nexus API
#[derive(OpenApi)]
#[openapi(
    paths(
        veridion_nexus::routes::log_action,
        veridion_nexus::routes::get_logs,
        veridion_nexus::routes::download_report
    ),
    components(schemas(
        LogActionRequest,
        LogActionResponse,
        ComplianceRecord
    )),
    tags(
        (name = "Compliance", description = "EU AI Act compliance endpoints")
    ),
    info(
        title = "VERIDION Nexus API",
        description = "Sovereign Trust Layer for High-Risk AI Agents in the EU",
        version = "1.0.0",
        contact(
            name = "Veridion Support",
            email = "support@veridion.nexus"
        )
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize environment variables
    dotenv().ok();
    
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // Initialize application state
    let app_state = web::Data::new(AppState::new());
    
    println!("🚀 Starting VERIDION Nexus API Server...");
    println!("📡 Server listening on http://0.0.0.0:8080");
    println!("📋 POST /log_action - Process agent actions through compliance pipeline");
    println!("📋 GET /logs - Retrieve compliance log history");
    println!("📄 GET /download_report - Download Annex IV compliance report PDF");
    println!("📚 Swagger UI available at http://127.0.0.1:8080/swagger-ui/");
    
    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::permissive(); // Allow all for MVP
        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .service(web::resource("/log_action").route(web::post().to(log_action)))
            .service(web::resource("/logs").route(web::get().to(get_logs)))
            .service(web::resource("/download_report").route(web::get().to(download_report)))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
