# Dashboard System

A high-performance WebSocket-based dashboard system built using Rust and Actix-web. The system is designed to support millions of concurrent connections and provide real-time data for user earnings, network connections, referrals, and rewards.

## Features

- Real-time WebSocket communication
- User authentication and authorization
- Network connection tracking
- Earnings calculation and statistics
- Referral management system
- High performance and scalability
- Integration with PostgreSQL and Redis

## Tech Stack

- **Backend Framework**: Actix-web 4.4.0
- **WebSockets**: actix-web-actors, actix
- **Database Access**: SQLx 0.7 (PostgreSQL)
- **Language**: Rust 2021 Edition
- **Logging**: tracing, tracing-subscriber, tracing-actix-web
- **Serialization**: serde, serde_json
- **Caching**: Redis

## Prerequisites

- Rust (1.73.0 or higher)
- Docker and Docker Compose
- PostgreSQL 15
- Redis 7

## Development Setup

1. Clone the repository:
   ```
   git clone <repository-url>
   cd temp-rust-websocket
   ```

2. Create a `.env` file in the project root with the following content:
   ```
   # Server configuration
   SERVER_PORT=8080
   RUST_LOG=debug

   # Database configuration
   DATABASE_URL=postgres://postgres:postgres@localhost:5432/dashboard
   DATABASE_MAX_CONNECTIONS=5

   # Redis configuration
   REDIS_URL=redis://localhost:6379

   # WebSocket configuration
   WS_HEARTBEAT_INTERVAL=30
   WS_CLIENT_TIMEOUT=120

   # Authentication
   JWT_SECRET=your_development_jwt_secret_change_in_production
   JWT_EXPIRATION=3600

   # Feature flags
   ENABLE_METRICS=true
   ```

3. Start the development environment using Docker Compose:
   ```
   docker-compose up -d
   ```

4. Build and run the project:
   ```
   cargo run
   ```

5. Access the application at `http://localhost:8080`

## Project Structure

- `src/main.rs`: Application entry point
- `src/config.rs`: Configuration handling
- `src/routes.rs`: API route definitions
- `src/handlers/`: Request handlers
- `src/services/`: Business logic
- `src/storage/`: Data access layer
- `src/models/`: Data models
- `src/errors/`: Error handling
- `src/metrics/`: Metrics collection

## Testing

Run the test suite:
```
cargo test
```

Run linting:
```
cargo clippy
```

Format code:
```
cargo fmt
```

## Docker

The project includes Docker configuration for both development and production environments. To build the Docker image:

```
docker build -t dashboard-system .
```

## License

[MIT](LICENSE) 