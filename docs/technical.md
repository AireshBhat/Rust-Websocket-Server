# Dashboard System - Technical Specifications

## 1. Overview

This document outlines the technical architecture for a high-performance WebSocket-based dashboard system built using Rust and Actix-web. The system is designed to support millions of concurrent connections and provide real-time data for user earnings, network connections, referrals, and rewards.

## 2. Technology Stack

* **Backend Framework**: Actix-web 4.4.0
* **WebSockets**: actix-web-actors, actix
* **Database Access**: SQLx 0.7 (PostgreSQL)
* **Language**: Rust 2021 Edition
* **Logging**: tracing, tracing-subscriber, tracing-actix-web
* **Serialization**: serde, serde_json
* **ID Generation**: nanoid
* **Async Runtime**: tokio 1.32.0
* **Error Handling**: anyhow, thiserror
* **Metrics**: prometheus-client
* **Load Balancing**: nginx (for production deployment)
* **Caching**: Redis
* **Cryptography**: ed25519-dalek (for WebSocket authentication)

## 3. Core Architecture

The system follows a layered architecture pattern with clearly defined boundaries between components:

### 3.1 HTTP/WebSocket Layer (`src/main.rs`, `src/routes.rs`)

* **Server Configuration**: Actix HttpServer setup with WebSocket support
* **Route Definition**: HTTP and WebSocket endpoints
* **Middleware Pipeline**: Authentication, logging, compression, CORS
* **Connection Management**: WebSocket connection setup and session management
* **Signature Verification**: ed25519 signature verification for WebSocket connections

### 3.2 Handler Layer (`src/handlers/`)

* **WebSocket Handler**: Manages WebSocket connections and message routing
* **REST Handler**: Processes HTTP requests for non-WebSocket operations
* **Authentication Handler**: Validates user credentials and manages sessions
* **Dashboard Handler**: Processes dashboard-specific requests
* **Signature Handler**: Verifies ed25519 signatures during WebSocket handshake

### 3.3 Service Layer (`src/services/`)

* **User Service**: User management, authentication, and public key management
* **Network Service**: Network connection tracking and statistics
* **Earnings Service**: Calculation of user earnings based on connections and referrals
* **Referral Service**: Management of user referrals and rewards
* **Notification Service**: Real-time notifications to connected clients
* **Signature Service**: Management of ed25519 signature verification

### 3.4 Storage Layer (`src/storage/`)

* **Storage Traits**: Abstract interfaces for data access
* **In-memory Storage Implementation**: In-memory storage layer for development and testing
* **PostgreSQL Implementation**: Primary storage for user data and earnings
* **Redis Implementation**: Caching and session management
* **Blockchain Connector**: Interface for future blockchain integration

### 3.5 Error Handling (`src/errors/`)

* **Custom Error Types**: Domain-specific error definitions
* **Error Mapping**: Consistent error handling across layers
* **Error Responses**: Standardized error responses for WebSocket and HTTP

### 3.6 Configuration (`src/config.rs`)

* **Environment-based Configuration**: Loading settings from environment variables
* **Server Settings**: WebSocket timeouts, heartbeat intervals, connection limits
* **Database Configuration**: Connection pools and query timeouts
* **Authentication Settings**: Configuration for signature verification mechanisms

### 3.7 Monitoring (`src/metrics/`)

* **Prometheus Metrics**: Real-time system monitoring
* **Connection Stats**: Active connections, message throughput
* **Performance Metrics**: Response times, resource utilization
* **Authentication Metrics**: Signature verification success/failure rates

## 4. WebSocket Implementation

### 4.1 Connection Management

```rust
pub struct WebSocketSession {
    pub id: String,
    pub user_id: Option<i64>,
    pub last_heartbeat: Instant,
    pub addr: Addr<WebSocketActor>,
    pub public_key: Option<String>, // Added for ed25519 authentication
}

pub struct WebSocketActor {
    pub id: String,
    pub user_id: Option<i64>,
    pub last_heartbeat: Instant,
    pub services: Arc<ServiceRegistry>,
    pub authenticated: bool, // Flag for ed25519 authentication status
}
```

### 4.2 WebSocket Authentication Workflow

1. **Client Connection Initiation**:
   * Client connects to WebSocket endpoint
   * Actix spawns a new WebSocketActor for the connection

2. **Initial Message with Signature**:
   * Client sends an authentication message containing:
     * User identifier (address/public key)
     * Message timestamp to prevent replay attacks
     * Ed25519 signature of the message

3. **Signature Verification**:
   * WebSocketActor receives the message
   * Extracts public key and retrieves user information from database
   * Verifies the signature using ed25519-dalek
   * If verification succeeds, marks connection as authenticated
   * If verification fails, closes the connection

4. **Connection Establishment**:
   * After successful authentication, normal WebSocket communication begins
   * Periodic re-authentication may be required for long-lived connections

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketAuthMessage {
    pub public_key: String,
    pub timestamp: i64,
    pub nonce: String,
    pub signature: String,
}

// Authentication flow in actor
impl Handler<WebSocketAuthMessage> for WebSocketActor {
    type Result = ();

    fn handle(&mut self, msg: WebSocketAuthMessage, ctx: &mut Self::Context) {
        // Verify signature
        if self.services.signature_service.verify_signature(
            &msg.public_key,
            &format!("{}:{}", msg.timestamp, msg.nonce),
            &msg.signature,
        ) {
            self.authenticated = true;
            self.user_id = Some(self.services.user_service.get_user_id_by_public_key(&msg.public_key));
        } else {
            // Close connection on authentication failure
            ctx.stop();
        }
    }
}
```

### 4.3 Scalability Approach

* **Actor Model**: Leveraging Actix's actor system for concurrent connection handling
* **Connection Pooling**: Efficient database connection management
* **Heartbeat Mechanism**: Detecting and cleaning up stale connections
* **Sharding**: Distribution of WebSocket connections across server instances
* **Backpressure Handling**: Flow control for message processing
* **Efficient Signature Verification**: Optimized cryptographic operations

### 4.4 Message Handling

```rust
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WebSocketMessage {
    Auth { auth_data: WebSocketAuthMessage },
    Connect { user_id: Option<i64>, device_id: String },
    Heartbeat,
    NetworkUpdate { networks: Vec<NetworkStatus> },
    EarningsUpdate { earnings: EarningsData },
    ReferralUpdate { referrals: ReferralData },
    Disconnect,
}
```

## 5. Data Models

### 5.1 User Model

```rust
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub wallet_address: Option<String>,
    pub public_key: Option<String>, // Added for ed25519 authentication
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}
```

### 5.2 Network Connection Model

```rust
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub id: i64,
    pub user_id: i64,
    pub network_name: String,
    pub ip_address: String,
    pub connected: bool,
    pub connection_time: Option<i64>,
    pub network_score: f64,
    pub points_earned: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 5.3 Earnings Model

```rust
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Earnings {
    pub id: i64,
    pub user_id: i64,
    pub amount: f64,
    pub source: EarningSource,
    pub timestamp: DateTime<Utc>,
    pub epoch: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EarningSource {
    Network,
    Referral,
    Bonus,
}
```

### 5.4 Referral Model

```rust
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Referral {
    pub id: i64,
    pub referrer_id: i64,
    pub referee_id: i64,
    pub status: ReferralStatus,
    pub points: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReferralStatus {
    Pending,
    Completed,
    Rewarded,
}
```

### 5.5 Public Key Model

```rust
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct UserPublicKey {
    pub user_id: i64,
    pub public_key: String,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub revoked: bool,
}
```

## 6. Error Handling Mechanism

### 6.1 Error Types

```rust
#[derive(Display, Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum DashboardErrorType {
    // Authentication errors
    Unauthorized,
    InvalidCredentials,
    SessionExpired,
    InvalidSignature,
    
    // WebSocket errors
    ConnectionError(String),
    MessageParseError,
    ConnectionClosed,
    
    // Database errors
    DatabaseError(String),
    EntityNotFound,
    
    // Business logic errors
    InvalidNetworkData,
    ReferralProcessingError,
    
    // System errors
    InternalError(String),
    RateLimited,
    ServiceUnavailable,
}
```

## 7. API Endpoints

### 7.1 WebSocket Endpoints

* **`/ws/dashboard`**: Main WebSocket connection for dashboard updates
* **`/ws/earnings`**: Real-time earnings updates
* **`/ws/referrals`**: Real-time referral tracking

### 7.2 REST Endpoints

* **`/api/auth`**: Authentication endpoints
* **`/api/user`**: User management
* **`/api/networks`**: Network connection management
* **`/api/earnings`**: Earnings history and statistics
* **`/api/referrals`**: Referral management
* **`/api/keys`**: Public key management

## 8. Signature Verification Service

### 8.1 Service Implementation

```rust
pub struct SignatureService {
    // Dependencies
}

impl SignatureService {
    // Create a new SignatureService
    pub fn new() -> Self {
        Self {}
    }

    // Verify an ed25519 signature
    pub async fn verify_signature(
        &self,
        public_key: &str,
        message: &str,
        signature: &str,
    ) -> Result<bool, DashboardError> {
        // Decode public key from hex/base64
        let public_key_bytes = decode_public_key(public_key)?;
        
        // Parse as ed25519 public key
        let public_key = PublicKey::from_bytes(&public_key_bytes)
            .map_err(|e| DashboardError::InvalidSignature(e.to_string()))?;
        
        // Decode signature from hex/base64
        let signature_bytes = decode_signature(signature)?;
        
        // Parse as ed25519 signature
        let signature = Signature::from_bytes(&signature_bytes)
            .map_err(|e| DashboardError::InvalidSignature(e.to_string()))?;
        
        // Verify the signature
        match public_key.verify(message.as_bytes(), &signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
```

### 8.2 Integration with User Service

The UserService will be extended to support retrieving users by their public key:

```rust
impl<T: UserStorage> UserService<T> {
    // Get user by public key
    pub async fn get_user_by_public_key(&self, public_key: &str) -> DashboardResult<Option<User>> {
        self.storage.find_user_by_public_key(public_key).await
    }
    
    // Register a new public key for a user
    pub async fn register_public_key(&self, user_id: i64, public_key: &str) -> DashboardResult<()> {
        self.storage.store_public_key(user_id, public_key).await
    }
}
```

## 9. Integration with Existing Authentication

The system will support both traditional JWT-based authentication for REST APIs and ed25519 signature-based authentication for WebSocket connections:

1. **REST API Authentication**: Continues to use JWT tokens
2. **WebSocket Authentication**: Uses ed25519 signatures
3. **Hybrid Approach**: WebSocket connections can also be authenticated with JWT tokens when needed

This dual-authentication approach provides flexibility while maintaining security for real-time connections.

## 10. Performance Considerations

* **Signature Verification Cost**: ed25519 verification is computationally efficient but should be cached appropriately
* **Connection Rate Limiting**: Prevent DoS attacks through signature verification
* **Caching Public Keys**: Store frequently used public keys in memory/Redis
* **Batched Database Queries**: Optimize public key lookups
* **Pre-authentication**: Verify signatures before fully establishing WebSocket connections