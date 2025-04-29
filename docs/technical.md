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

## 3. Core Architecture

The system follows a layered architecture pattern with clearly defined boundaries between components:

### 3.1 HTTP/WebSocket Layer (`src/main.rs`, `src/routes.rs`)

* **Server Configuration**: Actix HttpServer setup with WebSocket support
* **Route Definition**: HTTP and WebSocket endpoints
* **Middleware Pipeline**: Authentication, logging, compression, CORS
* **Connection Management**: WebSocket connection setup and session management

### 3.2 Handler Layer (`src/handlers/`)

* **WebSocket Handler**: Manages WebSocket connections and message routing
* **REST Handler**: Processes HTTP requests for non-WebSocket operations
* **Authentication Handler**: Validates user credentials and manages sessions
* **Dashboard Handler**: Processes dashboard-specific requests

### 3.3 Service Layer (`src/services/`)

* **User Service**: User management and authentication
* **Network Service**: Network connection tracking and statistics
* **Earnings Service**: Calculation of user earnings based on connections and referrals
* **Referral Service**: Management of user referrals and rewards
* **Notification Service**: Real-time notifications to connected clients

### 3.4 Storage Layer (`src/storage/`)

* **Storage Traits**: Abstract interfaces for data access
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

### 3.7 Monitoring (`src/metrics/`)

* **Prometheus Metrics**: Real-time system monitoring
* **Connection Stats**: Active connections, message throughput
* **Performance Metrics**: Response times, resource utilization

## 4. WebSocket Implementation

### 4.1 Connection Management

```rust
pub struct WebSocketSession {
    pub id: String,
    pub user_id: Option<i64>,
    pub last_heartbeat: Instant,
    pub addr: Addr<WebSocketActor>,
}

pub struct WebSocketActor {
    pub id: String,
    pub user_id: Option<i64>,
    pub last_heartbeat: Instant,
    pub services: Arc<ServiceRegistry>,
}
```

### 4.2 Scalability Approach

* **Actor Model**: Leveraging Actix's actor system for concurrent connection handling
* **Connection Pooling**: Efficient database connection management
* **Heartbeat Mechanism**: Detecting and cleaning up stale connections
* **Sharding**: Distribution of WebSocket connections across server instances
* **Backpressure Handling**: Flow control for message processing

### 4.3 Message Handling

```rust
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WebSocketMessage {
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

### 6.2 Error Structure

```rust
pub type DashboardResult<T> = Result<T, DashboardError>;

pub struct DashboardError {
    pub error_type: DashboardErrorType,
    pub inner: anyhow::Error,
    pub context: Option<String>,
}

impl actix_web::error::ResponseError for DashboardError {
    fn status_code(&self) -> StatusCode {
        match self.error_type {
            DashboardErrorType::Unauthorized | 
            DashboardErrorType::InvalidCredentials |
            DashboardErrorType::SessionExpired => StatusCode::UNAUTHORIZED,
            
            DashboardErrorType::EntityNotFound => StatusCode::NOT_FOUND,
            
            DashboardErrorType::InvalidNetworkData |
            DashboardErrorType::MessageParseError => StatusCode::BAD_REQUEST,
            
            DashboardErrorType::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
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

## 8. Scaling Strategy

### 8.1 Horizontal Scaling

* Multiple server instances behind a load balancer
* Sticky sessions for WebSocket connections
* Distributed session management with Redis

### 8.2 Database Scaling

* Connection pooling
* Read replicas for query-heavy operations
* Partitioning for historical data

### 8.3 Message Queue Integration

* Optional RabbitMQ or Kafka for message processing
* Buffering for handling traffic spikes

### 8.4 Optimization Techniques

* Binary protocol for WebSocket messages (MessagePack)
* Batched database operations
* Aggressive caching for dashboard data
* Selective updates to minimize WebSocket traffic

## 9. Blockchain Integration Strategy

### 9.1 Current Implementation (Server-based)

* Traditional server architecture with PostgreSQL storage
* Preparation for future blockchain migration

### 9.2 Future Blockchain Integration

* Abstract storage layer with blockchain implementation
* Smart contract interface for rewards and referrals
* Gradual migration strategy
* Hybrid operation during transition

## 10. Development Guidelines

### 10.1 Code Organization

* Follow Rust idioms and best practices
* Clear module boundaries
* Comprehensive documentation
* Unit and integration tests for all components

### 10.2 Error Handling

* Use custom error types
* Proper error propagation
* Detailed logging
* Client-friendly error messages

### 10.3 Performance Considerations

* Benchmark critical paths
* Profile for memory usage
* Stress test WebSocket connections
* Optimize database queries