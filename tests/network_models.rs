use temp_rust_websocket::models::network::{CreateNetworkConnectionDto, NetworkConnection, UpdateNetworkConnectionDto};
use chrono::Utc;

#[test]
fn test_network_connection_creation() {
    let user_id = 123;
    let network_name = "TestNetwork".to_string();
    let ip_address = "192.168.1.1".to_string();
    let initial_score = Some(75.0);

    let connection = NetworkConnection::new(
        user_id,
        network_name.clone(),
        ip_address.clone(),
        initial_score,
    );

    assert_eq!(connection.user_id, user_id);
    assert_eq!(connection.network_name, network_name);
    assert_eq!(connection.ip_address, ip_address);
    assert_eq!(connection.network_score, initial_score.unwrap());
    assert_eq!(connection.id, 0); // Default id before database insertion
    assert_eq!(connection.points_earned, 0.0);
    assert_eq!(connection.connection_time, Some(0));
    assert!(connection.connected);
}

#[test]
fn test_update_status() {
    let mut connection = NetworkConnection::new(
        123,
        "TestNetwork".to_string(),
        "192.168.1.1".to_string(),
        Some(75.0),
    );

    // Test disconnecting
    connection.update_status(false);
    assert!(!connection.connected);

    // Test reconnecting
    connection.update_status(true);
    assert!(connection.connected);
}

#[test]
fn test_add_connection_time() {
    let mut connection = NetworkConnection::new(
        123,
        "TestNetwork".to_string(),
        "192.168.1.1".to_string(),
        Some(75.0),
    );

    // Initial time should be 0
    assert_eq!(connection.connection_time, Some(0));

    // Add 60 seconds
    connection.add_connection_time(60);
    assert_eq!(connection.connection_time, Some(60));

    // Add another 30 seconds
    connection.add_connection_time(30);
    assert_eq!(connection.connection_time, Some(90));
}

#[test]
fn test_update_score() {
    let mut connection = NetworkConnection::new(
        123,
        "TestNetwork".to_string(),
        "192.168.1.1".to_string(),
        Some(75.0),
    );

    // Initial score should be 75.0
    assert_eq!(connection.network_score, 75.0);

    // Update score to 85.0
    connection.update_score(85.0);
    assert_eq!(connection.network_score, 85.0);
}

#[test]
fn test_add_points() {
    let mut connection = NetworkConnection::new(
        123,
        "TestNetwork".to_string(),
        "192.168.1.1".to_string(),
        Some(75.0),
    );

    // Initial points should be 0
    assert_eq!(connection.points_earned, 0.0);

    // Add 10 points
    connection.add_points(10.0);
    assert_eq!(connection.points_earned, 10.0);

    // Add another 5 points
    connection.add_points(5.0);
    assert_eq!(connection.points_earned, 15.0);
} 