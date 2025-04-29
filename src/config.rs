use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub websocket: WebSocketConfig,
    pub auth: AuthConfig,
    pub features: FeatureFlags,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub log_level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebSocketConfig {
    pub heartbeat_interval: u64,
    pub client_timeout: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FeatureFlags {
    pub enable_metrics: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        // Load .env file if present
        dotenv().ok();

        let server = ServerConfig {
            port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            log_level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        };

        let database = DatabaseConfig {
            url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
        };

        let redis = RedisConfig {
            url: env::var("REDIS_URL").expect("REDIS_URL must be set"),
        };

        let websocket = WebSocketConfig {
            heartbeat_interval: env::var("WS_HEARTBEAT_INTERVAL")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            client_timeout: env::var("WS_CLIENT_TIMEOUT")
                .unwrap_or_else(|_| "120".to_string())
                .parse()
                .unwrap_or(120),
        };

        let auth = AuthConfig {
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "default_jwt_secret".to_string()),
            jwt_expiration: env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600),
        };

        let features = FeatureFlags {
            enable_metrics: env::var("ENABLE_METRICS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
        };

        Ok(Config {
            server,
            database,
            redis,
            websocket,
            auth,
            features,
        })
    }
} 