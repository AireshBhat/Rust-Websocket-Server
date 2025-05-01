use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use chrono::{DateTime, Utc};
use nanoid::nanoid;
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::models::websocket::{WebSocketAuthMessage, WebSocketMessage};
use crate::services::SignatureService;
use crate::storage::UserStorage;
use crate::storage::memory::InMemoryUserStorage;

/// Tracks the authentication state of a WebSocket connection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuthState {
    /// No authentication attempt yet
    NotAuthenticated,
    /// Authentication is in progress
    Authenticating,
    /// Connection has been authenticated successfully
    Authenticated,
    /// Authentication has failed
    Failed,
}

/// WebSocket session data structure
pub struct WebSocketSession<T: UserStorage> {
    /// Unique session id
    pub id: String,
    /// User id if authenticated
    pub user_id: Option<i64>,
    /// Client IP address
    pub client_ip: String,
    /// Last heartbeat timestamp
    pub last_heartbeat: Instant,
    /// Authentication state
    pub auth_state: AuthState,
    /// When the connection was established
    pub connected_at: DateTime<Utc>,
    /// Public key used for authentication
    pub public_key: Option<String>,
    /// Heartbeat interval from config
    pub heartbeat_interval: Duration,
    /// Client timeout from config
    pub client_timeout: Duration,
    /// Authentication timeout for initial auth
    pub auth_timeout: Duration,
    /// Signature service for verification
    pub signature_service: Option<Arc<SignatureService<T>>>,
    /// Time to wait before closing after auth failure
    pub close_delay: Duration,
}

impl<T: UserStorage> Actor for WebSocketSession<T> {
    type Context = ws::WebsocketContext<Self>;

    /// Start the heartbeat and authentication timeout process on actor start
    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_heartbeat(ctx);
        self.start_auth_timeout(ctx);
        info!("WebSocket connection established: {}", self.id);
        
        // Send a welcome message that requests authentication
        ctx.text(json!({
            "type": "connection_established",
            "session_id": self.id,
            "auth_required": true,
            "message": "Please authenticate with an ed25519 signature"
        }).to_string());
    }

    /// Log when the actor is stopping
    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        if let Some(user_id) = self.user_id {
            info!("WebSocket connection closed for user {}: {}", user_id, self.id);
        } else {
            info!("WebSocket connection closed: {}", self.id);
        }
        actix::Running::Stop
    }
}

/// Handler for WebSocket messages
impl<T: UserStorage> StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession<T> {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.last_heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                debug!("WebSocket text message received: {:?}", text);
                if self.auth_state != AuthState::Authenticated {
                    self.handle_authentication_message(&text, ctx);
                } else {
                    self.handle_normal_message(&text, ctx);
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                debug!("WebSocket binary message received: {} bytes", bin.len());
                if self.auth_state != AuthState::Authenticated {
                    ctx.text(json!({
                        "type": "error",
                        "code": "unauthorized",
                        "message": "Authentication required"
                    }).to_string());
                    return;
                }
                ctx.binary(bin);
            }
            Ok(ws::Message::Close(reason)) => {
                info!("WebSocket closed with reason: {:?}", reason);
                ctx.close(reason);
            }
            Ok(ws::Message::Continuation(_)) => {
                warn!("WebSocket continuation frame received, not supported yet");
            }
            Ok(ws::Message::Nop) => {}
            Err(err) => {
                error!("WebSocket protocol error: {}", err);
                ctx.stop();
            }
        }
    }
}

impl<T: UserStorage> WebSocketSession<T> {
    /// Start the heartbeat process
    fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(self.heartbeat_interval, |act, ctx| {
            // Check if client has been responsive
            if Instant::now().duration_since(act.last_heartbeat) > act.client_timeout {
                warn!("WebSocket client timeout, disconnecting: {}", act.id);
                ctx.stop();
                return;
            }
            // Send ping
            ctx.ping(b"");
        });
    }
    
    /// Start the authentication timeout - close connection if not authenticated in time
    fn start_auth_timeout(&self, ctx: &mut ws::WebsocketContext<Self>) {
        if self.auth_state == AuthState::Authenticated {
            return;
        }
        ctx.run_later(self.auth_timeout, |act, ctx| {
            if act.auth_state != AuthState::Authenticated {
                warn!("WebSocket authentication timeout, disconnecting: {}", act.id);
                ctx.text(json!({
                    "type": "error",
                    "code": "auth_timeout",
                    "message": "Authentication timeout"
                }).to_string());
                // Give client time to receive the message before closing
                ctx.run_later(act.close_delay, |_, ctx| ctx.stop());
            }
        });
    }
    
    /// Handle authentication message
    fn handle_authentication_message(&mut self, text: &str, ctx: &mut ws::WebsocketContext<Self>) {
        let auth_result = match serde_json::from_str::<WebSocketMessage>(text) {
            Ok(WebSocketMessage::Auth(auth_msg)) => {
                self.auth_state = AuthState::Authenticating;
                self.verify_authentication(auth_msg, ctx)
            },
            Ok(_) => {
                ctx.text(json!({
                    "type": "error",
                    "code": "auth_required",
                    "message": "Authentication required as first message"
                }).to_string());
                return;
            },
            Err(e) => {
                ctx.text(json!({
                    "type": "error",
                    "code": "invalid_message",
                    "message": format!("Failed to parse message: {}", e)
                }).to_string());
                return;
            }
        };
        if let Err(e) = auth_result {
            self.auth_state = AuthState::Failed;
            ctx.text(json!({
                "type": "error",
                "code": "auth_failed",
                "message": format!("Authentication failed: {}", e)
            }).to_string());
            ctx.run_later(self.close_delay, |_, ctx| ctx.stop());
        }
    }
    
    /// Verify authentication message asynchronously
    fn verify_authentication(&mut self, auth_msg: WebSocketAuthMessage, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {
        // Ensure we have a signature service
        let signature_service = match &self.signature_service {
            Some(s) => s.clone(),
            None => return Err("Signature service not configured".to_string()),
        };
        let auth_clone = auth_msg.clone();
        let session_id = self.id.clone();
        let public_key = auth_msg.public_key.clone();
        // Spawn asynchronous verification future
        use actix::fut::wrap_future;
        use actix::ActorFutureExt;
        let fut = wrap_future(async move {
            signature_service.verify_websocket_auth(&auth_clone).await
        })
        .map(move |res, act: &mut WebSocketSession<T>, ctx| {
            match res {
                Ok(Some(user_id)) => {
                    act.auth_state = AuthState::Authenticated;
                    act.user_id = Some(user_id);
                    act.public_key = Some(public_key.clone());
                    info!("WebSocket authenticated for user {}: {}", user_id, session_id);
                    ctx.text(json!({
                        "type": "auth_success",
                        "user_id": user_id,
                        "session_id": session_id
                    }).to_string());
                }
                Ok(None) => {
                    act.auth_state = AuthState::Failed;
                    warn!("WebSocket valid signature but no user: {}", session_id);
                    ctx.text(json!({
                        "type": "error",
                        "code": "unknown_key",
                        "message": "Valid signature but no user associated with this public key"
                    }).to_string());
                    ctx.run_later(act.close_delay, |_, ctx| ctx.stop());
                }
                Err(e) => {
                    act.auth_state = AuthState::Failed;
                    error!("WebSocket authentication error: {}: {}", e, session_id);
                    ctx.text(json!({
                        "type": "error",
                        "code": "auth_failed",
                        "message": format!("Authentication failed: {}", e)
                    }).to_string());
                    ctx.run_later(act.close_delay, |_, ctx| ctx.stop());
                }
            }
        });
        ctx.spawn(fut);
        Ok(())
    }
    
    /// Handle normal message for authenticated connections
    fn handle_normal_message(&mut self, text: &str, ctx: &mut ws::WebsocketContext<Self>) {
        if self.auth_state != AuthState::Authenticated {
            ctx.text(json!({
                "type": "error",
                "code": "unauthorized",
                "message": "Authentication required"
            }).to_string());
            return;
        }
        match serde_json::from_str::<WebSocketMessage>(text) {
            Ok(message) => {
                match message {
                    WebSocketMessage::Heartbeat => {
                        self.last_heartbeat = Instant::now();
                        ctx.text(json!({
                            "type": "heartbeat_ack",
                            "timestamp": chrono::Utc::now().timestamp()
                        }).to_string());
                    },
                    WebSocketMessage::ConnectionUpdate { connected } => {
                        debug!("Connection update from user {}: connected={}", self.user_id.unwrap_or(0), connected);
                        ctx.text(json!({
                            "type": "connection_update_ack",
                            "connected": connected
                        }).to_string());
                    },
                    WebSocketMessage::NetworkUpdate { status, score } => {
                        debug!("Network update from user {}: status={}, score={}", self.user_id.unwrap_or(0), status, score);
                        ctx.text(json!({
                            "type": "network_update_ack",
                            "status": status,
                            "score": score
                        }).to_string());
                    },
                    WebSocketMessage::Auth(_) => {
                        ctx.text(json!({
                            "type": "info",
                            "message": "Already authenticated"
                        }).to_string());
                    },
                    _ => {
                        ctx.text(text);
                    }
                }
            },
            Err(e) => {
                ctx.text(json!({
                    "type": "error",
                    "code": "invalid_message",
                    "message": format!("Failed to parse message: {}", e)
                }).to_string());
            }
        }
    }
}

/// WebSocket connection handler
pub async fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    config: web::Data<Config>,
    signature_service: web::Data<SignatureService<InMemoryUserStorage>>,
) -> Result<HttpResponse, Error> {
    // Create a new WebSocket session
    let session = WebSocketSession::<InMemoryUserStorage> {
        id: nanoid!(),
        user_id: None,
        client_ip: req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_owned(),
        last_heartbeat: Instant::now(),
        auth_state: AuthState::NotAuthenticated,
        connected_at: Utc::now(),
        public_key: None,
        heartbeat_interval: Duration::from_secs(config.websocket.heartbeat_interval),
        client_timeout: Duration::from_secs(config.websocket.client_timeout),
        auth_timeout: Duration::from_secs(30), // 30 seconds to authenticate
        signature_service: Some(signature_service.into_inner()),
        close_delay: Duration::from_secs(2), // 2 seconds before closing after auth failure
    };
    
    // Start websocket connection
    let resp = ws::start(session, &req, stream);
    match &resp {
        Ok(_) => info!("WebSocket connection started: {}", req.connection_info().realip_remote_addr().unwrap_or("unknown")),
        Err(e) => error!("WebSocket connection error: {}", e),
    }
    resp
}

/// Dashboard-specific WebSocket endpoint
pub async fn dashboard_ws(
    req: HttpRequest,
    stream: web::Payload,
    config: web::Data<Config>,
    signature_service: web::Data<SignatureService<InMemoryUserStorage>>,
) -> Result<HttpResponse, Error> {
    websocket_route(req, stream, config, signature_service).await
}

/// Earnings-specific WebSocket endpoint 
pub async fn earnings_ws(
    req: HttpRequest,
    stream: web::Payload,
    config: web::Data<Config>,
    signature_service: web::Data<SignatureService<InMemoryUserStorage>>,
) -> Result<HttpResponse, Error> {
    websocket_route(req, stream, config, signature_service).await
}

/// Referrals-specific WebSocket endpoint
pub async fn referrals_ws(
    req: HttpRequest,
    stream: web::Payload,
    config: web::Data<Config>,
    signature_service: web::Data<SignatureService<InMemoryUserStorage>>,
) -> Result<HttpResponse, Error> {
    websocket_route(req, stream, config, signature_service).await
} 