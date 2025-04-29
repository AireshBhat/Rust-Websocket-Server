use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use chrono::{DateTime, Utc};
use nanoid::nanoid;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use crate::config::Config;

/// WebSocket session data structure
pub struct WebSocketSession {
    /// Unique session id
    pub id: String,
    /// User id if authenticated
    pub user_id: Option<i64>,
    /// Client IP address
    pub client_ip: String,
    /// Last heartbeat timestamp
    pub last_heartbeat: Instant,
    /// Is this connection authenticated
    pub authenticated: bool,
    /// When the connection was established
    pub connected_at: DateTime<Utc>,
    /// Public key for ed25519 authentication
    pub public_key: Option<String>,
    /// Heartbeat interval from config
    pub heartbeat_interval: Duration,
    /// Client timeout from config
    pub client_timeout: Duration,
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    /// Start the heartbeat process on actor start
    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_heartbeat(ctx);
        info!("WebSocket connection established: {}", self.id);
    }

    /// Log when the actor is stopping
    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        info!("WebSocket connection closed: {}", self.id);
        actix::Running::Stop
    }
}

/// Handler for WebSocket messages
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
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
                
                // Here we would call a message handler with proper deserialization
                // and routing based on message type
                // For now, we'll just echo back the message
                ctx.text(text);
            }
            Ok(ws::Message::Binary(bin)) => {
                debug!("WebSocket binary message received: {} bytes", bin.len());
                
                // Handle binary messages - we'll just echo for now
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

impl WebSocketSession {
    /// Start the heartbeat process
    fn start_heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(self.heartbeat_interval, |act, ctx| {
            // Check if client has been responsive
            if Instant::now().duration_since(act.last_heartbeat) > act.client_timeout {
                // Heartbeat timed out
                warn!("WebSocket client timeout, disconnecting: {}", act.id);
                ctx.stop();
                return;
            }
            
            // Send ping
            ctx.ping(b"");
        });
    }
}

/// WebSocket connection handler
pub async fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    config: web::Data<Config>,
) -> Result<HttpResponse, Error> {
    // Create a new WebSocket session
    let session = WebSocketSession {
        id: nanoid!(),
        user_id: None,
        client_ip: req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_owned(),
        last_heartbeat: Instant::now(),
        authenticated: false,
        connected_at: Utc::now(),
        public_key: None,
        heartbeat_interval: Duration::from_secs(config.websocket.heartbeat_interval),
        client_timeout: Duration::from_secs(config.websocket.client_timeout),
    };
    
    // Start websocket connection
    let resp = ws::start(session, &req, stream);
    match &resp {
        Ok(_) => info!("WebSocket connection started"),
        Err(e) => error!("WebSocket connection error: {}", e),
    }
    resp
}

/// Dashboard-specific WebSocket endpoint
pub async fn dashboard_ws(
    req: HttpRequest,
    stream: web::Payload,
    config: web::Data<Config>,
) -> Result<HttpResponse, Error> {
    websocket_route(req, stream, config).await
}

/// Earnings-specific WebSocket endpoint 
pub async fn earnings_ws(
    req: HttpRequest,
    stream: web::Payload,
    config: web::Data<Config>,
) -> Result<HttpResponse, Error> {
    websocket_route(req, stream, config).await
}

/// Referrals-specific WebSocket endpoint
pub async fn referrals_ws(
    req: HttpRequest,
    stream: web::Payload,
    config: web::Data<Config>,
) -> Result<HttpResponse, Error> {
    websocket_route(req, stream, config).await
} 