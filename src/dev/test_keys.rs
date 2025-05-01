use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::sync::Once;
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::{info, warn};

// Use a seed to generate deterministic keys for development
const TEST_SEED: &[u8; 32] = b"dashboard_test_key_seed_123456\0\0";
const NUM_TEST_KEYS: usize = 10;

// Singleton pattern to ensure we only generate keys once
static INIT: Once = Once::new();
static TEST_KEYS: Mutex<Option<Vec<TestKeyPair>>> = Mutex::new(None);

/// Represents an ed25519 key pair for testing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestKeyPair {
    /// User ID associated with this key pair
    pub user_id: i64,
    /// Username for this test key (helps with identification)
    pub username: String,
    /// Private key in hex format
    pub private_key: String,
    /// Public key in hex format
    pub public_key: String,
    /// Index number of this test key
    pub index: usize,
}

/// Initialize the test keys
pub fn initialize_test_keys() {
    INIT.call_once(|| {
        let test_keys = generate_test_keys();
        *TEST_KEYS.lock().unwrap() = Some(test_keys);
        info!("Test keys initialized for development");
    });
}

/// Get all test keys
pub fn get_test_keys() -> Vec<TestKeyPair> {
    initialize_test_keys();
    TEST_KEYS.lock().unwrap().clone().unwrap_or_default()
}

/// Get a test key by index
pub fn get_test_key(index: usize) -> Option<TestKeyPair> {
    initialize_test_keys();
    TEST_KEYS.lock().unwrap()
        .as_ref()
        .and_then(|keys| keys.get(index).cloned())
}

/// Get a test key by user ID
pub fn get_test_key_for_user(user_id: i64) -> Option<TestKeyPair> {
    initialize_test_keys();
    TEST_KEYS.lock().unwrap()
        .as_ref()
        .and_then(|keys| keys.iter().find(|k| k.user_id == user_id).cloned())
}

/// Get a mapping of public keys to user IDs
pub fn get_public_key_to_user_id_map() -> HashMap<String, i64> {
    initialize_test_keys();
    let mut map = HashMap::new();
    
    if let Some(keys) = TEST_KEYS.lock().unwrap().as_ref() {
        for key in keys {
            map.insert(key.public_key.clone(), key.user_id);
        }
    }
    
    map
}

/// Generate deterministic test keys
fn generate_test_keys() -> Vec<TestKeyPair> {
    let mut keys = Vec::with_capacity(NUM_TEST_KEYS);
    
    for i in 0..NUM_TEST_KEYS {
        // Create a deterministic seed based on the index
        let mut seed = *TEST_SEED;
        seed[31] = i as u8;
        
        // Generate the key pair
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = VerifyingKey::from(&signing_key);
        
        // Convert to hex strings
        let private_key = hex::encode(signing_key.to_bytes());
        let public_key = hex::encode(verifying_key.to_bytes());
        
        // User ID is just the index + 1 for simplicity
        let user_id = (i + 1) as i64;
        let username = format!("test_user_{}", i + 1);
        
        keys.push(TestKeyPair {
            user_id,
            username,
            private_key,
            public_key,
            index: i,
        });
    }
    
    keys
}

/// Register test keys with users in the database
#[cfg(debug_assertions)]
pub async fn register_test_keys_with_users<T: crate::storage::UserStorage>(
    storage: &T
) -> Result<(), crate::errors::DashboardError> {
    info!("Registering test keys with users");
    
    let test_keys = get_test_keys();
    let mut registered_count = 0;
    let mut already_registered_count = 0;
    
    for key in test_keys {
        // Check if user exists
        if let Some(user) = storage.find_user_by_id(key.user_id).await? {
            // Check if this key is already registered
            let existing_keys = storage.get_public_keys_for_user(user.id).await?;
            if existing_keys.contains(&key.public_key) {
                already_registered_count += 1;
                continue;
            }
            
            // Register the public key with this user
            storage.store_public_key(user.id, &key.public_key).await?;
            registered_count += 1;
            
            info!("Registered test key {} for user {}: {}", 
                  key.index, user.username, key.public_key);
        } else {
            warn!("Test user with ID {} not found, cannot register key", key.user_id);
        }
    }
    
    if already_registered_count > 0 {
        info!("{} test keys were already registered", already_registered_count);
    }
    
    info!("Registered {} test keys with users", registered_count);
    Ok(())
}

/// Generate a signature for test purposes
pub fn sign_test_message(private_key_hex: &str, message: &str) -> Result<String, String> {
    use ed25519_dalek::Signer;
    
    // Decode private key
    let private_key_bytes = hex::decode(private_key_hex)
        .map_err(|e| format!("Invalid private key format: {}", e))?;
    
    // Create signing key
    let signing_key = SigningKey::from_bytes(
        private_key_bytes.as_slice().try_into()
            .map_err(|_| "Invalid private key length".to_string())?
    );
    
    // Sign the message
    let signature = signing_key.sign(message.as_bytes());
    
    // Convert to hex
    Ok(hex::encode(signature.to_bytes()))
}

/// Generate a complete WebSocket authentication message
pub fn generate_auth_message(key_index: usize) -> Result<serde_json::Value, String> {
    let key = get_test_key(key_index)
        .ok_or_else(|| format!("Test key with index {} not found", key_index))?;
    
    // Create auth message components
    let timestamp = chrono::Utc::now().timestamp();
    let nonce = nanoid::nanoid!();
    
    // Message to sign: timestamp:nonce
    let message_to_sign = format!("{}:{}", timestamp, nonce);
    
    // Sign the message
    let signature = sign_test_message(&key.private_key, &message_to_sign)?;
    
    // Create the complete auth message
    let auth_message = serde_json::json!({
        "type": "auth",
        "data": {
            "public_key": key.public_key,
            "timestamp": timestamp,
            "nonce": nonce,
            "signature": signature
        }
    });
    
    Ok(auth_message)
} 