use temp_rust_websocket::models::user::{CreateUserDto, UpdateUserDto, User};
use chrono::Utc;

#[test]
fn test_user_creation() {
    let email = "test@example.com".to_string();
    let username = "testuser".to_string();
    let wallet_address = Some("0x123abc".to_string());

    let user = User::new(email.clone(), username.clone(), wallet_address.clone());

    assert_eq!(user.email, email);
    assert_eq!(user.username, username);
    assert_eq!(user.wallet_address, wallet_address);
    assert_eq!(user.id, 0); // Default id before database insertion
}

#[test]
fn test_create_user_dto() {
    let dto = CreateUserDto {
        email: "test@example.com".to_string(),
        username: "testuser".to_string(),
        password: "password123".to_string(),
        wallet_address: Some("0x123abc".to_string()),
    };

    assert_eq!(dto.email, "test@example.com");
    assert_eq!(dto.username, "testuser");
    assert_eq!(dto.password, "password123");
    assert_eq!(dto.wallet_address, Some("0x123abc".to_string()));
}

#[test]
fn test_update_user_dto() {
    let dto = UpdateUserDto {
        username: Some("newusername".to_string()),
        email: None,
        wallet_address: Some("0xnewaddress".to_string()),
    };

    assert_eq!(dto.username, Some("newusername".to_string()));
    assert_eq!(dto.email, None);
    assert_eq!(dto.wallet_address, Some("0xnewaddress".to_string()));
} 