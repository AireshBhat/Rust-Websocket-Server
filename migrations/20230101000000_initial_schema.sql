-- Initial database schema

-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(255) NOT NULL,
    wallet_address VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create user_credentials table
CREATE TABLE IF NOT EXISTS user_credentials (
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    password_hash VARCHAR(255) NOT NULL,
    salt VARCHAR(255) NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id)
);

-- Create network_connections table
CREATE TABLE IF NOT EXISTS network_connections (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    network_name VARCHAR(255) NOT NULL,
    ip_address VARCHAR(45) NOT NULL,
    connected BOOLEAN NOT NULL DEFAULT true,
    connection_time BIGINT,
    network_score DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    points_earned DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create user_public_keys table
CREATE TABLE IF NOT EXISTS user_public_keys (
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    public_key VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used TIMESTAMPTZ,
    revoked BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (user_id, public_key)
);

-- Add indexes for performance
CREATE INDEX IF NOT EXISTS idx_network_connections_user_id ON network_connections(user_id);
CREATE INDEX IF NOT EXISTS idx_user_public_keys_public_key ON user_public_keys(public_key);

-- Table for WebSocket sessions (for in-memory use, but can be persisted)
CREATE TABLE IF NOT EXISTS websocket_sessions (
    id VARCHAR(255) PRIMARY KEY,
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_heartbeat TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address VARCHAR(45) NOT NULL,
    user_agent VARCHAR(255),
    connected BOOLEAN NOT NULL DEFAULT true
); 