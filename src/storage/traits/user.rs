use crate::errors::DashboardResult;
use crate::models::user::{CreateUserDto, UpdateUserDto, User, UserCredentials, UserSession};
use async_trait::async_trait;

/// Trait defining storage operations for User-related data
#[async_trait]
pub trait UserStorage: Send + Sync + 'static {
    /// Find a user by their ID
    async fn find_user_by_id(&self, id: i64) -> DashboardResult<Option<User>>;
    
    /// Find a user by their email
    async fn find_user_by_email(&self, email: &str) -> DashboardResult<Option<User>>;
    
    /// Create a new user
    async fn create_user(&self, user: CreateUserDto) -> DashboardResult<User>;
    
    /// Update an existing user
    async fn update_user(&self, id: i64, update: UpdateUserDto) -> DashboardResult<User>;
    
    /// Delete a user
    async fn delete_user(&self, id: i64) -> DashboardResult<bool>;
    
    /// Store user credentials
    async fn store_credentials(&self, user_id: i64, password_hash: &str, salt: &str) -> DashboardResult<()>;
    
    /// Get user credentials
    async fn get_credentials(&self, user_id: i64) -> DashboardResult<Option<UserCredentials>>;
    
    /// Create a user session
    async fn create_session(
        &self,
        user_id: i64,
        ip_address: &str,
        user_agent: &str,
        expires_in_seconds: i64,
    ) -> DashboardResult<UserSession>;
    
    /// Find a session by ID
    async fn find_session_by_id(&self, session_id: &str) -> DashboardResult<Option<UserSession>>;
    
    /// Delete a session
    async fn delete_session(&self, session_id: &str) -> DashboardResult<bool>;
    
    /// Delete all sessions for a user
    async fn delete_user_sessions(&self, user_id: i64) -> DashboardResult<i64>;
    
    /// Update user's last active timestamp
    async fn update_last_active(&self, user_id: i64) -> DashboardResult<()>;
} 