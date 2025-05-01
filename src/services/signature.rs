use crate::errors::{DashboardError, DashboardResult};
use crate::models::websocket::WebSocketAuthMessage;
use crate::storage::UserStorage;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use hex;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Service for handling ed25519 signature verification
pub struct SignatureService<T: UserStorage> {
    user_storage: Arc<T>,
    // Optionally add caching for frequently used public keys
}

impl<T: UserStorage> SignatureService<T> {
    /// Create a new SignatureService with the given user storage
    pub fn new(user_storage: Arc<T>) -> Self {
        Self { user_storage }
    }

    /// Verify a WebSocket authentication message
    pub async fn verify_websocket_auth(
        &self,
        auth_msg: &WebSocketAuthMessage,
    ) -> DashboardResult<Option<i64>> {
        // Validate message structure
        if let Err(validation_error) = auth_msg.validate() {
            return Err(DashboardError::validation(validation_error));
        }

        // Verify the signature
        let verified = self.verify_signature(
            &auth_msg.public_key,
            &auth_msg.get_signed_message(),
            &auth_msg.signature,
        )?;

        if !verified {
            return Err(DashboardError::authentication("Invalid signature"));
        }

        // Find user by public key
        let user = self.user_storage.find_user_by_public_key(&auth_msg.public_key).await?;

        if let Some(user) = user {
            // Update last used timestamp
            self.user_storage
                .update_public_key_last_used(user.id, &auth_msg.public_key)
                .await?;

            info!("User {} authenticated via WebSocket", user.id);
            Ok(Some(user.id))
        } else {
            warn!("Valid signature but unknown public key: {}", auth_msg.public_key);
            Ok(None)
        }
    }

    /// Verify an ed25519 signature against a message and public key
    pub fn verify_signature(
        &self,
        public_key_hex: &str,
        message: &str,
        signature_hex: &str,
    ) -> DashboardResult<bool> {
        // Decode public key
        let public_key_bytes = hex::decode(public_key_hex)
            .map_err(|e| DashboardError::validation(format!("Invalid public key format: {}", e)))?;

        if public_key_bytes.len() != 32 {
            return Err(DashboardError::validation(format!(
                "Public key must be 32 bytes, got {} bytes",
                public_key_bytes.len()
            )));
        }

        let verifying_key = VerifyingKey::from_bytes(
            &public_key_bytes
                .as_slice()
                .try_into()
                .expect("slice with incorrect length"),
        )
        .map_err(|e| DashboardError::validation(format!("Invalid public key: {}", e)))?;

        // Decode signature
        let signature_bytes = hex::decode(signature_hex)
            .map_err(|e| DashboardError::validation(format!("Invalid signature format: {}", e)))?;

        if signature_bytes.len() != 64 {
            return Err(DashboardError::validation(format!(
                "Signature must be 64 bytes, got {} bytes",
                signature_bytes.len()
            )));
        }

        // Handle the signature creation - ed25519 2.0 has changed how signatures work
        let signature_array: [u8; 64] = signature_bytes.as_slice().try_into()
            .map_err(|_| DashboardError::validation("Invalid signature length".to_string()))?;
        let signature = Signature::from_bytes(&signature_array);

        match verifying_key.verify(message.as_bytes(), &signature) {
            Ok(_) => {
                debug!("Valid signature from {}", public_key_hex);
                Ok(true)
            }
            Err(e) => {
                debug!("Invalid signature from {}: {}", public_key_hex, e);
                Ok(false)
            }
        }
    }

    /// Register a new public key for a user
    pub async fn register_public_key(
        &self,
        user_id: i64,
        public_key: &str,
    ) -> DashboardResult<()> {
        if public_key.len() != 64 || !public_key.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(DashboardError::validation("Invalid public key format"));
        }
        self.user_storage.store_public_key(user_id, public_key).await?;
        info!("Registered new public key for user {}", user_id);
        Ok(())
    }

    /// Revoke a public key for a user
    pub async fn revoke_public_key(
        &self,
        user_id: i64,
        public_key: &str,
    ) -> DashboardResult<bool> {
        let revoked = self.user_storage.revoke_public_key(user_id, public_key).await?;
        if revoked {
            info!("Revoked public key for user {}", user_id);
        } else {
            warn!("Failed to revoke public key {} for user {}", public_key, user_id);
        }
        Ok(revoked)
    }

    /// Get all public keys for a user
    pub async fn get_user_public_keys(
        &self,
        user_id: i64,
    ) -> DashboardResult<Vec<String>> {
        self.user_storage.get_public_keys_for_user(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;
    use crate::models::User;

    mock! {
        UserStorage {}
        #[async_trait::async_trait]
        impl UserStorage for UserStorage {
            async fn find_user_by_id(&self, id: i64) -> DashboardResult<Option<User>>;
            async fn find_user_by_email(&self, email: &str) -> DashboardResult<Option<User>>;
            async fn create_user(&self, user: crate::models::user::CreateUserDto) -> DashboardResult<User>;
            async fn update_user(&self, id: i64, update: crate::models::user::UpdateUserDto) -> DashboardResult<User>;
            async fn delete_user(&self, id: i64) -> DashboardResult<bool>;
            async fn store_credentials(&self, user_id: i64, password_hash: &str, salt: &str) -> DashboardResult<()>;
            async fn get_credentials(&self, user_id: i64) -> DashboardResult<Option<crate::models::user::UserCredentials>>;
            async fn create_session(&self, user_id: i64, ip_address: &str, user_agent: &str, expires_in_seconds: i64) -> DashboardResult<crate::models::user::UserSession>;
            async fn find_session_by_id(&self, session_id: &str) -> DashboardResult<Option<crate::models::user::UserSession>>;
            async fn delete_session(&self, session_id: &str) -> DashboardResult<bool>;
            async fn delete_user_sessions(&self, user_id: i64) -> DashboardResult<i64>;
            async fn update_last_active(&self, user_id: i64) -> DashboardResult<()>;
            async fn find_user_by_public_key(&self, public_key: &str) -> DashboardResult<Option<User>>;
            async fn store_public_key(&self, user_id: i64, public_key: &str) -> DashboardResult<()>;
            async fn revoke_public_key(&self, user_id: i64, public_key: &str) -> DashboardResult<bool>;
            async fn get_public_keys_for_user(&self, user_id: i64) -> DashboardResult<Vec<String>>;
            async fn update_public_key_last_used(&self, user_id: i64, public_key: &str) -> DashboardResult<()>;
        }
    }

    // TODO: Add unit tests for the SignatureService
} 