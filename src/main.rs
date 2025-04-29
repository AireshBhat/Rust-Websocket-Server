// Main modules
mod config;
mod errors;
mod models;
mod routes;
mod services;
mod storage;

use actix_web::{web, App, HttpServer, Responder, HttpResponse, get};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("WebSocket Dashboard System")
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configuration from environment
    let config = config::Config::from_env().expect("Failed to load configuration");
    
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(match config.server.log_level.as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        })
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set up the logger");
    
    info!("Starting server on port {}", config.server.port);

    let config_port = config.server.port;
    
    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .service(hello)
            .service(health_check)
    })
    .bind(("0.0.0.0", config_port))?
    .run()
    .await
}
