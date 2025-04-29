use crate::errors::DashboardResult;
use crate::models::network::{
    CreateNetworkConnectionDto, NetworkConnection, NetworkStatistics, NetworkStatus,
    UpdateNetworkConnectionDto,
};
use async_trait::async_trait;

/// Trait defining storage operations for Network-related data
#[async_trait]
pub trait NetworkStorage: Send + Sync + 'static {
    /// Find a network connection by ID
    async fn find_connection_by_id(&self, id: i64) -> DashboardResult<Option<NetworkConnection>>;
    
    /// Find all network connections for a user
    async fn find_connections_by_user_id(&self, user_id: i64) -> DashboardResult<Vec<NetworkConnection>>;
    
    /// Find active network connections for a user
    async fn find_active_connections_by_user_id(&self, user_id: i64) -> DashboardResult<Vec<NetworkConnection>>;
    
    /// Create a new network connection
    async fn create_connection(&self, connection: CreateNetworkConnectionDto) -> DashboardResult<NetworkConnection>;
    
    /// Update a network connection
    async fn update_connection(
        &self,
        id: i64,
        update: UpdateNetworkConnectionDto,
    ) -> DashboardResult<NetworkConnection>;
    
    /// Delete a network connection
    async fn delete_connection(&self, id: i64) -> DashboardResult<bool>;
    
    /// Get current network status
    async fn get_network_status(&self, connection_id: i64) -> DashboardResult<Option<NetworkStatus>>;
    
    /// Update network status
    async fn update_network_status(
        &self,
        connection_id: i64,
        connected: bool,
        status_message: &str,
        network_score: Option<f64>,
    ) -> DashboardResult<NetworkStatus>;
    
    /// Get network statistics for a user
    async fn get_network_statistics(&self, user_id: i64) -> DashboardResult<NetworkStatistics>;
    
    /// Record network connection time
    async fn record_connection_time(&self, connection_id: i64, seconds: i64) -> DashboardResult<i64>;
    
    /// Record earned points for a connection
    async fn record_earned_points(&self, connection_id: i64, points: f64) -> DashboardResult<f64>;
} 