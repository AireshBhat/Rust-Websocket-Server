use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents a network connection in the system
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct NetworkConnection {
    /// Unique identifier for the network connection
    pub id: i64,
    /// User ID that this connection belongs to
    pub user_id: i64,
    /// Name of the network
    pub network_name: String,
    /// IP address of the connection
    pub ip_address: String,
    /// Whether the connection is currently active
    pub connected: bool,
    /// Total connection time in seconds
    pub connection_time: Option<i64>,
    /// Network score (quality metric)
    pub network_score: f64,
    /// Points earned from this connection
    pub points_earned: f64,
    /// Timestamp when the connection was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the connection was last updated
    pub updated_at: DateTime<Utc>,
}

/// Represents the current status of a network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    /// Network connection ID
    pub connection_id: i64,
    /// User ID that this status belongs to
    pub user_id: i64,
    /// Name of the network
    pub network_name: String,
    /// Whether the connection is currently active
    pub connected: bool,
    /// Current connection status message
    pub status_message: String,
    /// Current network score (quality metric)
    pub network_score: f64,
    /// Timestamp when the status was last updated
    pub updated_at: DateTime<Utc>,
}

/// Network statistics for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatistics {
    /// User ID that these statistics belong to
    pub user_id: i64,
    /// Total number of networks connected
    pub total_networks: i64,
    /// Number of currently active connections
    pub active_connections: i64,
    /// Total connection time across all networks (in seconds)
    pub total_connection_time: i64,
    /// Average network score
    pub average_network_score: f64,
    /// Total points earned from all networks
    pub total_points_earned: f64,
    /// Timestamp when the statistics were last updated
    pub last_updated: DateTime<Utc>,
}

/// Data needed to create a new network connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNetworkConnectionDto {
    /// User ID for the connection
    pub user_id: i64,
    /// Name of the network
    pub network_name: String,
    /// IP address of the connection
    pub ip_address: String,
    /// Initial network score
    pub initial_score: Option<f64>,
}

/// Data needed to update a network connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNetworkConnectionDto {
    /// Whether the connection is active
    pub connected: Option<bool>,
    /// Updated network score
    pub network_score: Option<f64>,
    /// Additional connection time to add (in seconds)
    pub additional_time: Option<i64>,
    /// Additional points earned
    pub additional_points: Option<f64>,
}

impl NetworkConnection {
    /// Create a new network connection
    pub fn new(
        user_id: i64,
        network_name: String,
        ip_address: String,
        initial_score: Option<f64>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Will be set by the database
            user_id,
            network_name,
            ip_address,
            connected: true,
            connection_time: Some(0),
            network_score: initial_score.unwrap_or(0.0),
            points_earned: 0.0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the connection status
    pub fn update_status(&mut self, connected: bool) {
        self.connected = connected;
        self.updated_at = Utc::now();
    }

    /// Add connection time
    pub fn add_connection_time(&mut self, seconds: i64) {
        self.connection_time = Some(self.connection_time.unwrap_or(0) + seconds);
        self.updated_at = Utc::now();
    }

    /// Update network score
    pub fn update_score(&mut self, score: f64) {
        self.network_score = score;
        self.updated_at = Utc::now();
    }

    /// Add points earned
    pub fn add_points(&mut self, points: f64) {
        self.points_earned += points;
        self.updated_at = Utc::now();
    }
} 