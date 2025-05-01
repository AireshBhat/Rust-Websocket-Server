use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Message for WebSocket authentication using ed25519 signatures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketAuthMessage {
    /// User's public key for signature verification
    pub public_key: String,
    /// Timestamp to prevent replay attacks
    pub timestamp: i64,
    /// Random nonce to ensure uniqueness of signatures
    pub nonce: String,
    /// Ed25519 signature of the message (timestamp + nonce)
    pub signature: String,
}

/// Response to a WebSocket authentication attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketAuthResponse {
    /// Whether authentication was successful
    pub success: bool,
    /// Message explaining the result
    pub message: String,
    /// Session identifier if authentication was successful
    pub session_id: Option<String>,
}

/// Common structure for all WebSocket messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketMessage {
    /// Authentication message
    Auth(WebSocketAuthMessage),
    /// Heartbeat message to keep connection alive
    Heartbeat,
    /// Connection status update
    ConnectionUpdate { connected: bool },
    /// Network status update
    NetworkUpdate { status: String, score: f64 },
    /// Earnings update
    EarningsUpdate { amount: f64, source: String },
    /// Generic error message
    Error { code: String, message: String },
    /// Custom data message
    Data { content: serde_json::Value },
}

/// WebSocket connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConnectionInfo {
    /// Unique session identifier
    pub session_id: String,
    /// User ID if authenticated
    pub user_id: Option<i64>,
    /// Client IP address
    pub client_ip: String,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the last message was received
    pub last_active: DateTime<Utc>,
    /// Authentication status
    pub authenticated: bool,
}

impl WebSocketAuthMessage {
    /// Create a new authentication message
    pub fn new(public_key: String, timestamp: i64, nonce: String, signature: String) -> Self {
        Self {
            public_key,
            timestamp,
            nonce,
            signature,
        }
    }

    /// Get the message that was signed (timestamp + nonce)
    pub fn get_signed_message(&self) -> String {
        format!("{}:{}", self.timestamp, self.nonce)
    }

    /// Validate the basic structure of the message
    pub fn validate(&self) -> Result<(), String> {
        // Check public key format (should be a valid hex string)
        if self.public_key.len() != 64 && self.public_key.len() != 128 {
            return Err("Invalid public key length".to_string());
        }

        if !self.public_key.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("Public key must be a hexadecimal string".to_string());
        }

        // Ensure the timestamp is reasonable (not too old or in the future)
        let now = chrono::Utc::now().timestamp();
        let time_diff = now - self.timestamp;
        
        if time_diff < -60 { // Allow 1 minute of clock skew
            return Err("Timestamp is in the future".to_string());
        }
        
        if time_diff > 300 { // 5 minutes expiration
            return Err("Authentication message has expired".to_string());
        }

        // Verify nonce is present and reasonable length
        if self.nonce.is_empty() || self.nonce.len() < 8 || self.nonce.len() > 64 {
            return Err("Invalid nonce length".to_string());
        }

        // Validate signature format
        if self.signature.len() != 128 {
            return Err("Invalid signature length".to_string());
        }

        if !self.signature.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("Signature must be a hexadecimal string".to_string());
        }

        Ok(())
    }
} 