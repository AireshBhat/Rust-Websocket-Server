use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a user in the system
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier for the user
    pub id: i64,
    /// User's email address (unique)
    pub email: String,
    /// Username for display
    pub username: String,
    /// Optional wallet address for blockchain integration
    pub wallet_address: Option<String>,
    /// Timestamp when the user was created
    pub created_at: DateTime<Utc>,
    /// Timestamp of the user's last activity
    pub last_active: DateTime<Utc>,
}

/// Represents a user's authentication credentials
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserCredentials {
    /// User ID that these credentials belong to
    pub user_id: i64,
    /// Hashed password
    pub password_hash: String,
    /// Salt used for password hashing
    pub salt: String,
    /// Timestamp when the password was last updated
    pub updated_at: DateTime<Utc>,
}

/// Data needed to create a new user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserDto {
    /// Email address for the new user
    pub email: String,
    /// Username for the new user
    pub username: String,
    /// Plain text password (will be hashed)
    pub password: String,
    /// Optional wallet address
    pub wallet_address: Option<String>,
}

/// Data needed to update a user's profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserDto {
    /// Updated username (optional)
    pub username: Option<String>,
    /// Updated email (optional)
    pub email: Option<String>,
    /// Updated wallet address (optional)
    pub wallet_address: Option<String>,
}

/// User session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// Unique session identifier
    pub id: String,
    /// User ID that this session belongs to
    pub user_id: i64,
    /// Time when the session was created
    pub created_at: DateTime<Utc>,
    /// Time when the session expires
    pub expires_at: DateTime<Utc>,
    /// IP address of the client
    pub ip_address: String,
    /// User agent of the client
    pub user_agent: String,
}

/// User login response with token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoginResponse {
    /// JWT token for authentication
    pub token: String,
    /// User information
    pub user: User,
    /// Token expiration time
    pub expires_at: DateTime<Utc>,
}

impl User {
    /// Create a new user with the given details
    pub fn new(email: String, username: String, wallet_address: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Will be set by the database
            email,
            username,
            wallet_address,
            created_at: now,
            last_active: now,
        }
    }
} 