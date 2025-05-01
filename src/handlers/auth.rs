use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::errors::DashboardResult;
use crate::services::UserService;
use crate::storage::UserStorage;

/// Login request
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    /// User's email
    pub email: String,
    /// User's password
    pub password: String,
}

/// Login handler
pub async fn login<T: UserStorage>(
    req: HttpRequest,
    login_data: web::Json<LoginRequest>,
    user_service: web::Data<UserService<T>>,
) -> DashboardResult<impl Responder> {
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .unwrap_or("unknown")
        .to_owned();
    
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_owned();
    
    info!("Login attempt for user: {}", login_data.email);
    
    let login_response = user_service
        .login(
            &login_data.email,
            &login_data.password,
            &ip,
            &user_agent,
        )
        .await?;
    
    info!("Login successful for user: {}", login_response.user.id);
    Ok(HttpResponse::Ok().json(login_response))
} 