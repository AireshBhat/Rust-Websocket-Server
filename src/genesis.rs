use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::models::network::{NetworkConnection, NetworkStatus};
use crate::models::user::{User, UserCredentials};

/// Comprehensive struct containing all genesis data for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisData {
    pub users: Vec<User>,
    pub user_credentials: Vec<UserCredentials>,
    pub network_connections: Vec<NetworkConnection>,
    pub network_statuses: Vec<NetworkStatus>,
    pub user_public_keys: Vec<UserPublicKey>,
}

/// User's public key for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublicKey {
    pub user_id: i64,
    pub public_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub revoked: bool,
}

impl GenesisData {
    /// Load genesis data from the assets directory
    pub fn load() -> Result<Self> {
        let path = Path::new("assets/genesis_data.json");
        let data = fs::read_to_string(path)?;
        let genesis_data: GenesisData = serde_json::from_str(&data)?;
        
        Ok(genesis_data)
    }
    
    /// Load genesis data only in development environment
    pub fn load_if_dev() -> Result<Option<Self>> {
        // Check if we're in development environment
        if cfg!(debug_assertions) {
            Ok(Some(Self::load()?))
        } else {
            Ok(None)
        }
    }
}

/// Functions to seed the database with genesis data
pub mod seed {
    use super::*;
    use sqlx::{Pool, Postgres};
    use tracing::info;
    
    /// Seed the database with all genesis data
    pub async fn seed_database(pool: &Pool<Postgres>) -> Result<()> {
        let genesis_data = GenesisData::load()?;
        
        info!("Seeding database with genesis data...");
        
        // Seed users
        seed_users(pool, &genesis_data.users).await?;
        
        // Seed user credentials
        seed_user_credentials(pool, &genesis_data.user_credentials).await?;
        
        // Seed network connections
        seed_network_connections(pool, &genesis_data.network_connections).await?;
        
        // Seed user public keys
        seed_user_public_keys(pool, &genesis_data.user_public_keys).await?;
        
        info!("Database seeded successfully!");
        
        Ok(())
    }
    
    /// Seed users table
    async fn seed_users(pool: &Pool<Postgres>, users: &[User]) -> Result<()> {
        for user in users {
            sqlx::query!(
                r#"
                INSERT INTO users (id, email, username, wallet_address, created_at, last_active)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (id) DO NOTHING
                "#,
                user.id,
                user.email,
                user.username,
                user.wallet_address,
                user.created_at,
                user.last_active
            )
            .execute(pool)
            .await?;
        }
        
        info!("Seeded {} users", users.len());
        Ok(())
    }
    
    /// Seed user_credentials table
    async fn seed_user_credentials(pool: &Pool<Postgres>, credentials: &[UserCredentials]) -> Result<()> {
        for cred in credentials {
            sqlx::query!(
                r#"
                INSERT INTO user_credentials (user_id, password_hash, salt, updated_at)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (user_id) DO NOTHING
                "#,
                cred.user_id,
                cred.password_hash,
                cred.salt,
                cred.updated_at
            )
            .execute(pool)
            .await?;
        }
        
        info!("Seeded {} user credentials", credentials.len());
        Ok(())
    }
    
    /// Seed network_connections table
    async fn seed_network_connections(pool: &Pool<Postgres>, connections: &[NetworkConnection]) -> Result<()> {
        for conn in connections {
            sqlx::query!(
                r#"
                INSERT INTO network_connections 
                (id, user_id, network_name, ip_address, connected, connection_time, network_score, points_earned, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (id) DO NOTHING
                "#,
                conn.id,
                conn.user_id,
                conn.network_name,
                conn.ip_address,
                conn.connected,
                conn.connection_time,
                conn.network_score,
                conn.points_earned,
                conn.created_at,
                conn.updated_at
            )
            .execute(pool)
            .await?;
        }
        
        info!("Seeded {} network connections", connections.len());
        Ok(())
    }
    
    /// Seed user_public_keys table
    async fn seed_user_public_keys(pool: &Pool<Postgres>, keys: &[UserPublicKey]) -> Result<()> {
        for key in keys {
            sqlx::query!(
                r#"
                INSERT INTO user_public_keys (user_id, public_key, created_at, last_used, revoked)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (user_id, public_key) DO NOTHING
                "#,
                key.user_id,
                key.public_key,
                key.created_at,
                key.last_used,
                key.revoked
            )
            .execute(pool)
            .await?;
        }
        
        info!("Seeded {} user public keys", keys.len());
        Ok(())
    }
}

/// Functions to seed in-memory storage for development
pub mod memory_seed {
    use super::*;
    use crate::storage::memory::InMemoryUserStorage;
    use crate::storage::UserStorage;
    use tracing::info;
    
    /// Seed in-memory storage with all genesis data
    pub async fn seed_storage(user_storage: &InMemoryUserStorage) -> Result<()> {
        let genesis_data = GenesisData::load()?;
        
        info!("Seeding in-memory storage with genesis data...");
        
        // Seed users
        seed_users(user_storage, &genesis_data.users).await?;
        
        // Seed user credentials
        seed_user_credentials(user_storage, &genesis_data.user_credentials).await?;
        
        // Seed user public keys
        seed_user_public_keys(user_storage, &genesis_data.user_public_keys).await?;
        
        info!("In-memory storage seeded successfully!");
        
        Ok(())
    }
    
    /// Seed users in in-memory storage
    async fn seed_users(storage: &InMemoryUserStorage, users: &[User]) -> Result<()> {
        for user in users {
            // We need to manually insert users since InMemoryUserStorage's create_user
            // generates its own IDs, but we need to use the IDs from genesis data
            let users_lock = storage.get_users_map();
            let mut users_map = users_lock.lock().map_err(|e| anyhow::anyhow!("Failed to lock users map: {}", e))?;
            
            let emails_lock = storage.get_emails_map();
            let mut emails_map = emails_lock.lock().map_err(|e| anyhow::anyhow!("Failed to lock emails map: {}", e))?;
            
            // Insert user data
            users_map.insert(user.id, user.clone());
            emails_map.insert(user.email.clone(), user.id);
            
            // Ensure next_id is greater than any existing user id
            let next_id_lock = storage.get_next_id();
            let mut next_id = next_id_lock.lock().map_err(|e| anyhow::anyhow!("Failed to lock next_id: {}", e))?;
            if *next_id <= user.id {
                *next_id = user.id + 1;
            }
        }
        
        info!("Seeded {} users in memory", users.len());
        Ok(())
    }
    
    /// Seed user credentials in in-memory storage
    async fn seed_user_credentials(storage: &InMemoryUserStorage, credentials: &[UserCredentials]) -> Result<()> {
        for cred in credentials {
            let credentials_lock = storage.get_credentials_map();
            let mut credentials_map = credentials_lock.lock().map_err(|e| anyhow::anyhow!("Failed to lock credentials map: {}", e))?;
            
            credentials_map.insert(cred.user_id, cred.clone());
        }
        
        info!("Seeded {} user credentials in memory", credentials.len());
        Ok(())
    }
    
    /// Seed user public keys in in-memory storage
    async fn seed_user_public_keys(storage: &InMemoryUserStorage, keys: &[UserPublicKey]) -> Result<()> {
        for key in keys {
            if key.revoked {
                continue; // Skip revoked keys
            }
            
            // Store the public key using the built-in method
            storage.store_public_key(key.user_id, &key.public_key).await
                .map_err(|e| anyhow::anyhow!("Failed to store public key: {}", e))?;
        }
        
        info!("Seeded user public keys in memory");
        Ok(())
    }
}

/// Test functions for the genesis module
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_load_genesis_data() {
        let result = GenesisData::load();
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert!(!data.users.is_empty());
        assert!(!data.network_connections.is_empty());
        assert!(!data.user_credentials.is_empty());
        assert!(!data.user_public_keys.is_empty());
    }
} 