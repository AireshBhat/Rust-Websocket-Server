// Main modules
mod config;
mod errors;
mod genesis;
mod handlers;
mod models;
mod routes;
mod services;
mod storage;
#[cfg(debug_assertions)]
mod dev;

use actix_web::{web, App, HttpServer, Responder, HttpResponse, get, middleware};
use actix_cors::Cors;
use tracing::{info, Level, warn};
use tracing_subscriber::FmtSubscriber;
use std::time::Duration;
use std::sync::Arc;
use crate::services::SignatureService;
use crate::services::UserService;
use crate::storage::memory::InMemoryUserStorage;

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

    // Initialize database connection
    let pool = match &config.database.url {
        Some(url) => {
            info!("Connecting to database...");
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(config.database.max_connections)
                .acquire_timeout(Duration::from_secs(config.database.connection_timeout))
                .connect(url)
                .await
                .expect("Failed to connect to database");
                
            // In development mode, check if we should seed the database
            if cfg!(debug_assertions) && config.server.environment == "development" {
                info!("Development mode: Checking if we should seed the database");
                if config.database.seed_on_start {
                    info!("Seeding database with genesis data");
                    genesis::seed::seed_database(&pool)
                        .await
                        .expect("Failed to seed database with genesis data");
                }
            }
            
            Some(pool)
        },
        None => {
            info!("No database URL provided, using in-memory storage");
            None
        }
    };
    
    // Load genesis data in memory for testing when in development mode
    let genesis_data = if cfg!(debug_assertions) && config.server.environment == "development" {
        match genesis::GenesisData::load() {
            Ok(data) => {
                info!("Loaded genesis data for testing: {} users, {} network connections", 
                      data.users.len(), data.network_connections.len());
                Some(Arc::new(data))
            },
            Err(e) => {
                info!("Failed to load genesis data: {}", e);
                None
            }
        }
    } else {
        None
    };

    let config_data = web::Data::new(config.clone());
    let config_port = config.server.port;
    
    // Initialize in-memory storage for development
    let user_storage_instance = InMemoryUserStorage::new();
    let user_storage = web::Data::new(user_storage_instance.clone());
    
    // Seed in-memory storage with genesis data in development mode
    #[cfg(debug_assertions)]
    if config.server.environment == "development" {
        info!("Seeding in-memory storage with genesis data");
        if let Err(e) = genesis::memory_seed::seed_storage(&user_storage_instance).await {
            warn!("Failed to seed in-memory storage: {}", e);
        } else {
            info!("In-memory storage seeded successfully");
        }
        
        // Initialize and register test keys after seeding
        info!("Initializing development test keys");
        dev::test_keys::initialize_test_keys();
        
        // Register test keys with users if they weren't part of genesis data
        if let Err(e) = dev::test_keys::register_test_keys_with_users(&user_storage_instance).await {
            warn!("Failed to register test keys with users: {}", e);
        }
    }
    
    // Create and register SignatureService
    let signature_service = web::Data::new(SignatureService::new(Arc::new(user_storage_instance.clone())));

    // Create and register UserService
    let user_service = web::Data::new(UserService::new(
        Arc::new(user_storage_instance.clone()),
        config.auth.jwt_secret.clone(),
        config.auth.jwt_expiration as i64,
    ));
    
    // If we have genesis data, make it available to the application
    let genesis_data = genesis_data.map(web::Data::new);
    
    // Database pool as app data if available
    let pool_data = pool.map(web::Data::new);
    
    // Start HTTP server with WebSocket support
    HttpServer::new(move || {
        // CORS configuration
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
            
        let mut app = App::new()
            // Add shared configuration
            .app_data(config_data.clone())
            // Add storage and services
            .app_data(user_storage.clone())
            .app_data(signature_service.clone())
            .app_data(user_service.clone())
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
            .service(routes::websocket_routes());
            
        // Add database pool if available
        if let Some(ref pool) = pool_data {
            app = app.app_data(pool.clone());
        }
        
        // Add genesis data if available (dev mode)
        if let Some(ref genesis) = genesis_data {
            app = app.app_data(genesis.clone());
        }
        
        app
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
