use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::config::Config;
use crate::errors::DashboardResult;
use crate::models::user::{CreateUserDto, UpdateUserDto, User};
use crate::services::UserService;
use crate::storage::UserStorage;

/// Request for adding a public key to a user
#[derive(Debug, Serialize, Deserialize)]
pub struct AddPublicKeyRequest {
    /// The public key to add (hex-encoded)
    pub public_key: String,
}

/// Register a new user
pub async fn register_user<T: UserStorage>(
    user_data: web::Json<CreateUserDto>,
    user_service: web::Data<UserService<T>>,
) -> DashboardResult<impl Responder> {
    info!("Registering new user with email: {}", user_data.email);
    
    let user = user_service.register_user(user_data.into_inner()).await?;
    
    info!("User registered successfully: {}", user.id);
    Ok(HttpResponse::Created().json(user))
}

/// Get user by ID
pub async fn get_user<T: UserStorage>(
    path: web::Path<i64>,
    user_service: web::Data<UserService<T>>,
) -> DashboardResult<impl Responder> {
    let user_id = path.into_inner();
    info!("Getting user with ID: {}", user_id);
    
    let user = user_service.get_user(user_id).await?;
    
    Ok(HttpResponse::Ok().json(user))
}

/// Update user
pub async fn update_user<T: UserStorage>(
    path: web::Path<i64>,
    update_data: web::Json<UpdateUserDto>,
    user_service: web::Data<UserService<T>>,
) -> DashboardResult<impl Responder> {
    let user_id = path.into_inner();
    info!("Updating user with ID: {}", user_id);
    
    let user = user_service
        .update_user(user_id, update_data.into_inner())
        .await?;
    
    info!("User updated successfully: {}", user_id);
    Ok(HttpResponse::Ok().json(user))
}

/// Delete user
pub async fn delete_user<T: UserStorage>(
    path: web::Path<i64>,
    user_service: web::Data<UserService<T>>,
) -> DashboardResult<impl Responder> {
    let user_id = path.into_inner();
    info!("Deleting user with ID: {}", user_id);
    
    let deleted = user_service.delete_user(user_id).await?;
    
    if deleted {
        info!("User deleted successfully: {}", user_id);
        Ok(HttpResponse::NoContent().finish())
    } else {
        error!("Failed to delete user: {}", user_id);
        Ok(HttpResponse::InternalServerError().finish())
    }
}

/// Add a public key to a user
pub async fn add_public_key<T: UserStorage>(
    path: web::Path<i64>,
    key_data: web::Json<AddPublicKeyRequest>,
    user_service: web::Data<UserService<T>>,
) -> DashboardResult<impl Responder> {
    let user_id = path.into_inner();
    info!("Adding public key for user: {}", user_id);
    
    user_service
        .add_public_key(user_id, &key_data.public_key)
        .await?;
    
    info!("Public key added successfully for user: {}", user_id);
    Ok(HttpResponse::Created().json(serde_json::json!({
        "status": "success",
        "message": "Public key added successfully"
    })))
}

/// Get user's public keys
pub async fn get_public_keys<T: UserStorage>(
    path: web::Path<i64>,
    user_service: web::Data<UserService<T>>,
) -> DashboardResult<impl Responder> {
    let user_id = path.into_inner();
    info!("Getting public keys for user: {}", user_id);
    
    let keys = user_service.get_public_keys(user_id).await?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user_id": user_id,
        "public_keys": keys
    })))
}

/// Revoke a public key from a user
pub async fn revoke_public_key<T: UserStorage>(
    path: web::Path<(i64, String)>,
    user_service: web::Data<UserService<T>>,
) -> DashboardResult<impl Responder> {
    let (user_id, public_key) = path.into_inner();
    info!("Revoking public key for user: {}", user_id);
    
    let revoked = user_service.revoke_public_key(user_id, &public_key).await?;
    
    if revoked {
        info!("Public key revoked successfully for user: {}", user_id);
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "Public key revoked successfully"
        })))
    } else {
        info!("Public key not found or already revoked for user: {}", user_id);
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "status": "error",
            "message": "Public key not found or already revoked"
        })))
    }
} 