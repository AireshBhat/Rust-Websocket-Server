use actix_web::{web, Scope, get, HttpResponse, Responder};
use crate::handlers::websocket::{dashboard_ws, earnings_ws, referrals_ws};
use crate::handlers::user::{
    register_user, get_user, update_user, delete_user,
    add_public_key, get_public_keys, revoke_public_key
};
use crate::handlers::auth::login;

pub fn api_routes() -> Scope {
    web::scope("/api")
        // Auth routes will go here
        .service(auth_routes())
        // User routes will go here
        .service(user_routes())
        // Network routes will go here
        .service(network_routes())
        // Earnings routes will go here
        .service(earnings_routes())
        // Referral routes will go here
        .service(referral_routes())
        // Development routes (only in debug builds)
        .service(dev_routes())
}

pub fn auth_routes() -> Scope {
    web::scope("/auth")
        // Login endpoint
        .route("/login", web::post().to(login::<crate::storage::memory::InMemoryUserStorage>))
}

pub fn user_routes() -> Scope {
    web::scope("/users")
        // User registration
        .route("", web::post().to(register_user::<crate::storage::memory::InMemoryUserStorage>))
        // Get user by ID
        .route("/{id}", web::get().to(get_user::<crate::storage::memory::InMemoryUserStorage>))
        // Update user
        .route("/{id}", web::put().to(update_user::<crate::storage::memory::InMemoryUserStorage>))
        // Delete user
        .route("/{id}", web::delete().to(delete_user::<crate::storage::memory::InMemoryUserStorage>))
        // Public key management
        .route("/{id}/keys", web::post().to(add_public_key::<crate::storage::memory::InMemoryUserStorage>))
        .route("/{id}/keys", web::get().to(get_public_keys::<crate::storage::memory::InMemoryUserStorage>))
        .route("/{id}/keys/{key}", web::delete().to(revoke_public_key::<crate::storage::memory::InMemoryUserStorage>))
}

pub fn network_routes() -> Scope {
    web::scope("/networks")
        // Network information, status, etc.
}

pub fn earnings_routes() -> Scope {
    web::scope("/earnings")
        // Earnings history, statistics, etc.
}

pub fn referral_routes() -> Scope {
    web::scope("/referrals")
        // Referral generation, tracking, etc.
}

pub fn websocket_routes() -> Scope {
    web::scope("/ws")
        // Dashboard WebSocket endpoint
        .route("/dashboard", web::get().to(dashboard_ws))
        // Earnings WebSocket endpoint
        .route("/earnings", web::get().to(earnings_ws))
        // Referrals WebSocket endpoint
        .route("/referrals", web::get().to(referrals_ws))
}

// Development routes - only available in debug builds
#[cfg(debug_assertions)]
pub fn dev_routes() -> Scope {
    web::scope("/dev")
        // Test keys endpoint
        .service(get_test_keys)
        .service(get_test_key)
        .service(get_test_auth_message)
}

// Empty scope for production builds
#[cfg(not(debug_assertions))]
pub fn dev_routes() -> Scope {
    web::scope("/dev")
}

// Development endpoints for test keys

#[cfg(debug_assertions)]
#[get("/test-keys")]
async fn get_test_keys() -> impl Responder {
    let keys = crate::dev::test_keys::get_test_keys();
    HttpResponse::Ok().json(keys)
}

#[cfg(debug_assertions)]
#[get("/test-keys/{index}")]
async fn get_test_key(path: web::Path<usize>) -> impl Responder {
    let index = path.into_inner();
    
    match crate::dev::test_keys::get_test_key(index) {
        Some(key) => HttpResponse::Ok().json(key),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Test key with index {} not found", index)
        }))
    }
}

#[cfg(debug_assertions)]
#[get("/test-auth-message/{index}")]
async fn get_test_auth_message(path: web::Path<usize>) -> impl Responder {
    let index = path.into_inner();
    
    match crate::dev::test_keys::generate_auth_message(index) {
        Ok(message) => HttpResponse::Ok().json(message),
        Err(error) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": error
        }))
    }
} 