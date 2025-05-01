use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use nanoid::nanoid;

use crate::errors::{DashboardError, DashboardResult};
use crate::models::user::{CreateUserDto, UpdateUserDto, User, UserCredentials, UserSession};
use crate::storage::UserStorage;

/// In-memory implementation of the UserStorage trait for development and testing
#[derive(Clone)]
pub struct InMemoryUserStorage {
    users: Arc<Mutex<HashMap<i64, User>>>,
    emails: Arc<Mutex<HashMap<String, i64>>>,
    credentials: Arc<Mutex<HashMap<i64, UserCredentials>>>,
    sessions: Arc<Mutex<HashMap<String, UserSession>>>,
    public_keys: Arc<Mutex<HashMap<String, i64>>>,
    user_public_keys: Arc<Mutex<HashMap<i64, Vec<String>>>>,
    next_id: Arc<Mutex<i64>>,
}

impl Default for InMemoryUserStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryUserStorage {
    /// Create a new empty in-memory user storage
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            emails: Arc::new(Mutex::new(HashMap::new())),
            credentials: Arc::new(Mutex::new(HashMap::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            public_keys: Arc::new(Mutex::new(HashMap::new())),
            user_public_keys: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }
    
    /// Get direct access to the users map for genesis data seeding
    pub fn get_users_map(&self) -> &Arc<Mutex<HashMap<i64, User>>> {
        &self.users
    }
    
    /// Get direct access to the emails map for genesis data seeding
    pub fn get_emails_map(&self) -> &Arc<Mutex<HashMap<String, i64>>> {
        &self.emails
    }
    
    /// Get direct access to the credentials map for genesis data seeding
    pub fn get_credentials_map(&self) -> &Arc<Mutex<HashMap<i64, UserCredentials>>> {
        &self.credentials
    }
    
    /// Get direct access to the next_id for genesis data seeding
    pub fn get_next_id(&self) -> &Arc<Mutex<i64>> {
        &self.next_id
    }
    
    /// Get direct access to the public_keys map for genesis data seeding
    pub fn get_public_keys_map(&self) -> &Arc<Mutex<HashMap<String, i64>>> {
        &self.public_keys
    }
    
    /// Get direct access to the user_public_keys map for genesis data seeding
    pub fn get_user_public_keys_map(&self) -> &Arc<Mutex<HashMap<i64, Vec<String>>>> {
        &self.user_public_keys
    }
}

#[async_trait]
impl UserStorage for InMemoryUserStorage {
    async fn find_user_by_id(&self, id: i64) -> DashboardResult<Option<User>> {
        let users = self.users.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        
        Ok(users.get(&id).cloned())
    }
    
    async fn find_user_by_email(&self, email: &str) -> DashboardResult<Option<User>> {
        let user_id = {
            let emails = self.emails.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
            emails.get(email).copied()
        };
        
        match user_id {
            Some(id) => self.find_user_by_id(id).await,
            None => Ok(None),
        }
    }
    
    async fn create_user(&self, user_dto: CreateUserDto) -> DashboardResult<User> {
        let mut users = self.users.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        let mut emails = self.emails.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        let mut next_id = self.next_id.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        
        // Check if email already exists
        if emails.contains_key(&user_dto.email) {
            return Err(DashboardError::validation(format!("Email {} is already in use", user_dto.email)));
        }
        
        let id = *next_id;
        *next_id += 1;
        
        let now = Utc::now();
        let user = User {
            id,
            email: user_dto.email.clone(),
            username: user_dto.username,
            wallet_address: user_dto.wallet_address,
            created_at: now,
            last_active: now,
        };
        
        emails.insert(user_dto.email, id);
        users.insert(id, user.clone());
        
        Ok(user)
    }
    
    async fn update_user(&self, id: i64, update: UpdateUserDto) -> DashboardResult<User> {
        let mut users = self.users.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        let mut emails = self.emails.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        
        let user = users.get_mut(&id).ok_or_else(|| DashboardError::not_found(format!("User with ID {} not found", id)))?;
        
        // Update email if provided and it's different
        if let Some(email) = update.email {
            if email != user.email {
                // Check if new email is already in use
                if emails.contains_key(&email) {
                    return Err(DashboardError::validation(format!("Email {} is already in use", email)));
                }
                
                emails.remove(&user.email);
                emails.insert(email.clone(), id);
                user.email = email;
            }
        }
        
        // Update username if provided
        if let Some(username) = update.username {
            user.username = username;
        }
        
        // Update wallet address if provided
        if let Some(wallet_address) = update.wallet_address {
            user.wallet_address = Some(wallet_address);
        }
        
        Ok(user.clone())
    }
    
    async fn delete_user(&self, id: i64) -> DashboardResult<bool> {
        // First check if user exists to avoid complex error handling later
        if let Ok(None) = self.find_user_by_id(id).await {
            return Ok(false);
        }
        
        // Obtain user email for later removal
        let user_email = {
            let users = self.users.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
            users.get(&id).map(|u| u.email.clone())
        };
        
        // If user has no email (shouldn't happen), return false
        let user_email = match user_email {
            Some(email) => email,
            None => return Ok(false),
        };
        
        // Delete user's sessions
        let _ = self.delete_user_sessions(id).await?;
        
        // Get user public keys for removal
        let keys_to_remove = {
            let user_public_keys = self.user_public_keys.lock()
                .map_err(|e| DashboardError::internal_server(e.to_string()))?;
            user_public_keys.get(&id).cloned().unwrap_or_default()
        };
        
        // Remove user from various storage
        {
            let mut users = self.users.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
            users.remove(&id);
        }
        
        {
            let mut emails = self.emails.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
            emails.remove(&user_email);
        }
        
        {
            let mut credentials = self.credentials.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
            credentials.remove(&id);
        }
        
        {
            let mut user_public_keys = self.user_public_keys.lock()
                .map_err(|e| DashboardError::internal_server(e.to_string()))?;
            user_public_keys.remove(&id);
        }
        
        // Remove all user's public keys
        {
            let mut public_keys = self.public_keys.lock()
                .map_err(|e| DashboardError::internal_server(e.to_string()))?;
            
            for key in keys_to_remove {
                public_keys.remove(&key);
            }
        }
        
        Ok(true)
    }
    
    async fn store_credentials(&self, user_id: i64, password_hash: &str, salt: &str) -> DashboardResult<()> {
        let mut credentials = self.credentials.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        
        credentials.insert(user_id, UserCredentials {
            user_id,
            password_hash: password_hash.to_string(),
            salt: salt.to_string(),
            updated_at: Utc::now(),
        });
        
        Ok(())
    }
    
    async fn get_credentials(&self, user_id: i64) -> DashboardResult<Option<UserCredentials>> {
        let credentials = self.credentials.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        
        Ok(credentials.get(&user_id).cloned())
    }
    
    async fn create_session(
        &self,
        user_id: i64,
        ip_address: &str,
        user_agent: &str,
        expires_in_seconds: i64,
    ) -> DashboardResult<UserSession> {
        let mut sessions = self.sessions.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        
        let now = Utc::now();
        let expires_at = now + Duration::seconds(expires_in_seconds);
        
        let session = UserSession {
            id: nanoid!(),
            user_id,
            created_at: now,
            expires_at,
            ip_address: ip_address.to_string(),
            user_agent: user_agent.to_string(),
        };
        
        sessions.insert(session.id.clone(), session.clone());
        
        Ok(session)
    }
    
    async fn find_session_by_id(&self, session_id: &str) -> DashboardResult<Option<UserSession>> {
        let sessions = self.sessions.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        
        Ok(sessions.get(session_id).cloned())
    }
    
    async fn delete_session(&self, session_id: &str) -> DashboardResult<bool> {
        let mut sessions = self.sessions.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        
        Ok(sessions.remove(session_id).is_some())
    }
    
    async fn delete_user_sessions(&self, user_id: i64) -> DashboardResult<i64> {
        let mut sessions = self.sessions.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        
        let mut count = 0;
        sessions.retain(|_, session| {
            if session.user_id == user_id {
                count += 1;
                false
            } else {
                true
            }
        });
        
        Ok(count)
    }
    
    async fn update_last_active(&self, user_id: i64) -> DashboardResult<()> {
        let mut users = self.users.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        
        if let Some(user) = users.get_mut(&user_id) {
            user.last_active = Utc::now();
            Ok(())
        } else {
            Err(DashboardError::not_found(format!("User with ID {} not found", user_id)))
        }
    }
    
    async fn find_user_by_public_key(&self, public_key: &str) -> DashboardResult<Option<User>> {
        let user_id = {
            let public_keys = self.public_keys.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
            public_keys.get(public_key).copied()
        };
        
        match user_id {
            Some(id) => self.find_user_by_id(id).await,
            None => Ok(None),
        }
    }
    
    async fn store_public_key(&self, user_id: i64, public_key: &str) -> DashboardResult<()> {
        let mut public_keys = self.public_keys.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        let mut user_public_keys = self.user_public_keys.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        
        // Check if public key already exists
        if let Some(existing_user_id) = public_keys.get(public_key) {
            if *existing_user_id != user_id {
                return Err(DashboardError::validation(format!("Public key already associated with another user")));
            }
            return Ok(());
        }
        
        // Add public key
        public_keys.insert(public_key.to_string(), user_id);
        
        // Add to user's public keys
        user_public_keys.entry(user_id)
            .or_insert_with(Vec::new)
            .push(public_key.to_string());
        
        Ok(())
    }
    
    async fn revoke_public_key(&self, user_id: i64, public_key: &str) -> DashboardResult<bool> {
        let mut public_keys = self.public_keys.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        let mut user_public_keys = self.user_public_keys.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        
        // Check if public key exists and belongs to user
        match public_keys.get(public_key) {
            Some(existing_user_id) if *existing_user_id == user_id => {
                public_keys.remove(public_key);
                
                if let Some(keys) = user_public_keys.get_mut(&user_id) {
                    keys.retain(|k| k != public_key);
                }
                
                Ok(true)
            },
            Some(_) => Err(DashboardError::validation(format!("Public key belongs to another user"))),
            None => Ok(false),
        }
    }
    
    async fn get_public_keys_for_user(&self, user_id: i64) -> DashboardResult<Vec<String>> {
        let user_public_keys = self.user_public_keys.lock().map_err(|e| DashboardError::internal_server(e.to_string()))?;
        
        Ok(user_public_keys.get(&user_id).cloned().unwrap_or_default())
    }
    
    async fn update_public_key_last_used(&self, user_id: i64, public_key: &str) -> DashboardResult<()> {
        // For in-memory storage, we don't track last used timestamp
        // This would be implemented in a real database storage
        Ok(())
    }
} 