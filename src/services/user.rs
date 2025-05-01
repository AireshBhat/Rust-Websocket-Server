use crate::errors::{DashboardError, DashboardResult};
use crate::models::user::{CreateUserDto, UpdateUserDto, User, UserLoginResponse, UserSession};
use crate::storage::UserStorage;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

/// Claims for JWT token
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// Subject (user ID)
    sub: String,
    /// Issuer
    iss: String,
    /// Expiration time
    exp: usize,
    /// Issued at
    iat: usize,
}

/// User service for handling user-related operations
pub struct UserService<T: UserStorage> {
    storage: Arc<T>,
    jwt_secret: String,
    jwt_expiration: i64,
}

impl<T: UserStorage> UserService<T> {
    /// Create a new UserService with the given storage
    pub fn new(storage: Arc<T>, jwt_secret: String, jwt_expiration: i64) -> Self {
        Self {
            storage,
            jwt_secret,
            jwt_expiration,
        }
    }

    /// Register a new user
    pub async fn register_user(&self, user_data: CreateUserDto) -> DashboardResult<User> {
        // Check if email already exists
        if let Some(_) = self.storage.find_user_by_email(&user_data.email).await? {
            return Err(DashboardError::validation(format!(
                "User with email {} already exists",
                user_data.email
            )));
        }

        // Hash password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(user_data.password.as_bytes(), &salt)
            .map_err(|e| DashboardError::internal_server(format!("Password hashing error: {}", e)))?
            .to_string();

        // Create user
        let user = self.storage.create_user(user_data).await?;

        // Store credentials
        self.storage
            .store_credentials(user.id, &password_hash, &salt.to_string())
            .await?;

        Ok(user)
    }

    /// Authenticate user and return JWT token
    pub async fn login(
        &self,
        email: &str,
        password: &str,
        ip_address: &str,
        user_agent: &str,
    ) -> DashboardResult<UserLoginResponse> {
        // Find user by email
        let user = self
            .storage
            .find_user_by_email(email)
            .await?
            .ok_or_else(|| DashboardError::authentication("Invalid email or password"))?;

        // Get credentials
        let credentials = self
            .storage
            .get_credentials(user.id)
            .await?
            .ok_or_else(|| DashboardError::authentication("Credentials not found"))?;

        // Verify password
        let parsed_hash = PasswordHash::new(&credentials.password_hash)
            .map_err(|e| DashboardError::internal_server(format!("Password parsing error: {}", e)))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| DashboardError::authentication("Invalid email or password"))?;

        // Create session
        self.storage
            .create_session(user.id, ip_address, user_agent, self.jwt_expiration)
            .await?;

        // Update last active
        self.storage.update_last_active(user.id).await?;

        // Generate JWT token
        let now = Utc::now();
        let exp_time = now + Duration::seconds(self.jwt_expiration);
        let claims = Claims {
            sub: user.id.to_string(),
            iss: "dashboard_system".to_string(),
            exp: exp_time.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| DashboardError::internal_server(format!("Token generation error: {}", e)))?;

        Ok(UserLoginResponse {
            token,
            user,
            expires_at: exp_time,
        })
    }

    /// Verify JWT token and return user ID
    pub async fn verify_token(&self, token: &str) -> DashboardResult<i64> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| DashboardError::authentication(format!("Invalid token: {}", e)))?;

        let user_id = token_data
            .claims
            .sub
            .parse::<i64>()
            .map_err(|_| DashboardError::authentication("Invalid user ID in token"))?;

        Ok(user_id)
    }

    /// Get user by ID
    pub async fn get_user(&self, id: i64) -> DashboardResult<User> {
        self.storage
            .find_user_by_id(id)
            .await?
            .ok_or_else(|| DashboardError::not_found(format!("User with ID {} not found", id)))
    }

    /// Update user
    pub async fn update_user(&self, id: i64, update: UpdateUserDto) -> DashboardResult<User> {
        // Check if user exists
        self.get_user(id).await?;
        
        // If email is being updated, check if it's available
        if let Some(ref email) = update.email {
            if let Some(existing) = self.storage.find_user_by_email(email).await? {
                if existing.id != id {
                    return Err(DashboardError::validation(format!(
                        "Email {} is already in use",
                        email
                    )));
                }
            }
        }

        self.storage.update_user(id, update).await
    }

    /// Delete user
    pub async fn delete_user(&self, id: i64) -> DashboardResult<bool> {
        // Check if user exists
        self.get_user(id).await?;
        
        // Delete user sessions
        self.storage.delete_user_sessions(id).await?;
        
        // Delete user
        self.storage.delete_user(id).await
    }

    /// Add a public key to a user
    pub async fn add_public_key(&self, user_id: i64, public_key: &str) -> DashboardResult<()> {
        // Validate that user exists
        self.get_user(user_id).await?;
        
        // Validate public key format - should be a 64-character hex string
        if !Self::is_valid_ed25519_public_key(public_key) {
            return Err(DashboardError::validation("Invalid public key format. Expected a 64-character hex string."));
        }
        
        // Store the public key
        self.storage.store_public_key(user_id, public_key).await
    }
    
    /// Get public keys for a user
    pub async fn get_public_keys(&self, user_id: i64) -> DashboardResult<Vec<String>> {
        // Validate that user exists
        self.get_user(user_id).await?;
        
        // Get public keys
        self.storage.get_public_keys_for_user(user_id).await
    }
    
    /// Revoke a public key for a user
    pub async fn revoke_public_key(&self, user_id: i64, public_key: &str) -> DashboardResult<bool> {
        // Validate that user exists
        self.get_user(user_id).await?;
        
        // Revoke the public key
        self.storage.revoke_public_key(user_id, public_key).await
    }
    
    /// Find a user by public key
    pub async fn find_user_by_public_key(&self, public_key: &str) -> DashboardResult<Option<User>> {
        self.storage.find_user_by_public_key(public_key).await
    }
    
    /// Validate that a string is a valid ed25519 public key (64-character hex string)
    fn is_valid_ed25519_public_key(public_key: &str) -> bool {
        public_key.len() == 64 && public_key.chars().all(|c| c.is_ascii_hexdigit())
    }
} 