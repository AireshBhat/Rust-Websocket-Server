// Main modules
mod config;
mod errors;
mod handlers;
mod models;
mod routes;
mod services;
mod storage;

use actix_web::{web, App, HttpServer, Responder, HttpResponse, get, middleware};
use actix_cors::Cors;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use std::time::Duration;

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

    let config_data = web::Data::new(config.clone());
    let config_port = config.server.port;
    
    // Start HTTP server with WebSocket support
    HttpServer::new(move || {
        // CORS configuration
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
            
        App::new()
            // Add shared configuration
            .app_data(config_data.clone())
            // Configure request timeouts
            .app_data(
                web::JsonConfig::default()
                    .limit(4194304) // 4MB JSON payload limit
                    .error_handler(|err, _| {
                        let err_msg = format!("JSON error: {}", err);
                        actix_web::error::InternalError::from_response(
                            err, 
                            HttpResponse::BadRequest().body(err_msg)
                        ).into()
                    })
            )
            // Add middleware
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(cors)
            // Register basic services
            .service(hello)
            .service(health_check)
            // Register API routes
            .service(routes::api_routes())
            // Register WebSocket routes
            .service(routes::websocket_routes())
    })
    .keep_alive(Duration::from_secs(60))
    .client_request_timeout(Duration::from_secs(60))
    .client_disconnect_timeout(Duration::from_secs(5))
    .server_hostname(format!("dashboard-server-{}", env!("CARGO_PKG_VERSION")))
    .workers(num_cpus::get())
    .shutdown_timeout(30) // Graceful shutdown timeout in seconds
    .bind(("0.0.0.0", config_port))?
    .run()
    .await
}
