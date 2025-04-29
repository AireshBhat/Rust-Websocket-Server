use actix_web::{web, Scope};

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
}

pub fn auth_routes() -> Scope {
    web::scope("/auth")
        // Login, registration, etc.
}

pub fn user_routes() -> Scope {
    web::scope("/user")
        // User profile, settings, etc.
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
        // Earnings WebSocket endpoint
        // Referrals WebSocket endpoint
} 