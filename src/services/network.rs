use crate::errors::{DashboardError, DashboardResult};
use crate::models::network::{
    CreateNetworkConnectionDto, NetworkConnection, NetworkStatistics, NetworkStatus,
    UpdateNetworkConnectionDto,
};
use crate::storage::NetworkStorage;
use std::sync::Arc;
use tracing::{error, info};

/// Network service for handling network-related operations
pub struct NetworkService<T: NetworkStorage> {
    storage: Arc<T>,
}

impl<T: NetworkStorage> NetworkService<T> {
    /// Create a new NetworkService with the given storage
    pub fn new(storage: Arc<T>) -> Self {
        Self { storage }
    }

    /// Get a network connection by ID
    pub async fn get_connection(&self, id: i64) -> DashboardResult<NetworkConnection> {
        self.storage
            .find_connection_by_id(id)
            .await?
            .ok_or_else(|| {
                DashboardError::not_found(format!("Network connection with ID {} not found", id))
            })
    }

    /// Get all network connections for a user
    pub async fn get_user_connections(&self, user_id: i64) -> DashboardResult<Vec<NetworkConnection>> {
        self.storage.find_connections_by_user_id(user_id).await
    }

    /// Get active network connections for a user
    pub async fn get_active_user_connections(
        &self,
        user_id: i64,
    ) -> DashboardResult<Vec<NetworkConnection>> {
        self.storage.find_active_connections_by_user_id(user_id).await
    }

    /// Create a new network connection
    pub async fn create_connection(
        &self,
        connection: CreateNetworkConnectionDto,
    ) -> DashboardResult<NetworkConnection> {
        let connection = self.storage.create_connection(connection).await?;

        // Initialize network status
        self.storage
            .update_network_status(
                connection.id,
                true,
                "Connection established",
                Some(connection.network_score),
            )
            .await?;

        Ok(connection)
    }

    /// Update a network connection
    pub async fn update_connection(
        &self,
        id: i64,
        update: UpdateNetworkConnectionDto,
    ) -> DashboardResult<NetworkConnection> {
        // Check if connection exists
        self.get_connection(id).await?;

        let connection = self.storage.update_connection(id, update.clone()).await?;

        // Update network status if connection status changed
        if let Some(connected) = update.connected {
            let status_message = if connected {
                "Connection re-established"
            } else {
                "Connection closed"
            };

            let network_score = update.clone().network_score;

            self.storage
                .update_network_status(id, connected, status_message, network_score)
                .await?;
        }

        Ok(connection)
    }

    /// Delete a network connection
    pub async fn delete_connection(&self, id: i64) -> DashboardResult<bool> {
        // Check if connection exists
        self.get_connection(id).await?;

        self.storage.delete_connection(id).await
    }

    /// Get current network status
    pub async fn get_network_status(&self, connection_id: i64) -> DashboardResult<NetworkStatus> {
        self.storage
            .get_network_status(connection_id)
            .await?
            .ok_or_else(|| {
                DashboardError::not_found(format!(
                    "Network status for connection {} not found",
                    connection_id
                ))
            })
    }

    /// Update network status
    pub async fn update_network_status(
        &self,
        connection_id: i64,
        connected: bool,
        status_message: &str,
        network_score: Option<f64>,
    ) -> DashboardResult<NetworkStatus> {
        // Check if connection exists
        self.get_connection(connection_id).await?;

        self.storage
            .update_network_status(connection_id, connected, status_message, network_score)
            .await
    }

    /// Get network statistics for a user
    pub async fn get_network_statistics(&self, user_id: i64) -> DashboardResult<NetworkStatistics> {
        self.storage.get_network_statistics(user_id).await
    }

    /// Record connection time
    pub async fn record_connection_time(
        &self,
        connection_id: i64,
        seconds: i64,
    ) -> DashboardResult<i64> {
        // Check if connection exists
        self.get_connection(connection_id).await?;

        self.storage.record_connection_time(connection_id, seconds).await
    }

    /// Record earned points
    pub async fn record_earned_points(
        &self,
        connection_id: i64,
        points: f64,
    ) -> DashboardResult<f64> {
        // Check if connection exists
        self.get_connection(connection_id).await?;

        self.storage.record_earned_points(connection_id, points).await
    }

    /// Calculate network score based on connection metrics
    pub async fn calculate_network_score(&self, connection_id: i64) -> DashboardResult<f64> {
        // This is a placeholder for the actual scoring algorithm
        // In a real implementation, this would incorporate various metrics
        let connection = self.get_connection(connection_id).await?;
        
        // Simple scoring based on connection time
        let base_score = 50.0; // Base score out of 100
        let time_factor = connection.connection_time.unwrap_or(0) as f64 / 3600.0; // Hours connected
        let time_bonus = time_factor.min(24.0) * 2.0; // Cap at 48 points for 24 hours
        
        // Calculate final score (capped at 100)
        let score = (base_score + time_bonus).min(100.0);
        
        // Update the connection with the new score
        self.storage
            .update_connection(
                connection_id,
                UpdateNetworkConnectionDto {
                    connected: None,
                    network_score: Some(score),
                    additional_time: None,
                    additional_points: None,
                },
            )
            .await?;
        
        Ok(score)
    }
} 