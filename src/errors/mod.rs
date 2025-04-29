use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    InternalServer(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
}

impl DashboardError {
    pub fn authentication(msg: impl Into<String>) -> Self {
        DashboardError::Authentication(msg.into())
    }

    pub fn authorization(msg: impl Into<String>) -> Self {
        DashboardError::Authorization(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        DashboardError::Validation(msg.into())
    }

    pub fn database(msg: impl Into<String>) -> Self {
        DashboardError::Database(msg.into())
    }

    pub fn websocket(msg: impl Into<String>) -> Self {
        DashboardError::WebSocket(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        DashboardError::NotFound(msg.into())
    }

    pub fn internal_server(msg: impl Into<String>) -> Self {
        DashboardError::InternalServer(msg.into())
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        DashboardError::BadRequest(msg.into())
    }

    pub fn rate_limit(msg: impl Into<String>) -> Self {
        DashboardError::RateLimit(msg.into())
    }
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    status: String,
    message: String,
    code: u16,
}

impl ResponseError for DashboardError {
    fn status_code(&self) -> StatusCode {
        match self {
            DashboardError::Authentication(_) => StatusCode::UNAUTHORIZED,
            DashboardError::Authorization(_) => StatusCode::FORBIDDEN,
            DashboardError::Validation(_) => StatusCode::BAD_REQUEST,
            DashboardError::NotFound(_) => StatusCode::NOT_FOUND,
            DashboardError::BadRequest(_) => StatusCode::BAD_REQUEST,
            DashboardError::RateLimit(_) => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        HttpResponse::build(status).json(ErrorResponse {
            status: status.to_string(),
            message: self.to_string(),
            code: status.as_u16(),
        })
    }
}

impl From<sqlx::Error> for DashboardError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DashboardError::NotFound("Entity not found".into()),
            _ => DashboardError::Database(err.to_string()),
        }
    }
}

impl From<redis::RedisError> for DashboardError {
    fn from(err: redis::RedisError) -> Self {
        DashboardError::Database(format!("Redis error: {}", err))
    }
}

pub type DashboardResult<T> = Result<T, DashboardError>; 