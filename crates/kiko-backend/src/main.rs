pub mod messaging;
pub mod services;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    http::{Method, header},
    routing::{get, post},
};
use chrono::DateTime;
use tokio::{net::TcpListener, signal};
use tower_http::cors::CorsLayer;

use kiko::errors::Report;
use kiko::log;

use crate::{messaging::PubSub, services::SessionServiceInMemory};

pub struct AppState {
    started_at: DateTime<chrono::Utc>,
    sessions: SessionServiceInMemory,
    pub_sub: PubSub,
}

#[tokio::main]
async fn main() -> Result<(), Report> {
    // Setup logging
    kiko::log::setup()?;

    // Add application state
    let app_state = Arc::new(AppState {
        started_at: chrono::Utc::now(),
        sessions: SessionServiceInMemory::new(),
        pub_sub: PubSub::new(),
    });

    // Setup the routes
    let app = setup_routes(app_state);

    // Setup the server
    let listener = TcpListener::bind("127.0.0.1:3030").await?;
    log::info!("Starting server on http://{}", listener.local_addr()?);
    log::info!("Press Ctrl+C to stop the server");

    // Start the server with graceful shutdown
    // IMPORTANT: Use into_make_service_with_connect_info to preserve client connection info
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    log::info!("Shutting down server");
    Ok(())
}

/// Wait for a shutdown signal (Ctrl+C or SIGTERM)
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::SignalKind;

        signal::unix::signal(SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    log::info!("Signal received, starting graceful shutdown");
}

/// Setup the application routes
fn setup_routes(app_state: Arc<AppState>) -> Router {
    // TODO(SeedyROM): Add rate limiting configuration

    let api_routes = Router::new()
        .route("/session", post(handlers::v1::session::create))
        .route("/session/{session_id}", get(handlers::v1::session::get))
        .route("/ws", get(handlers::v1::websocket::upgrade))
        .with_state(app_state.clone());

    Router::new()
        .route("/health", get(handlers::v1::health::get))
        .nest("/api/v1", api_routes)
        .layer(cors_layer())
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(app_state)
}

/// Setup CORS layer
/// This function configures CORS settings based on the environment.
/// In debug mode, it allows requests from specific local development ports.
/// In production, it allows all origins (permissive).
fn cors_layer() -> CorsLayer {
    // if cfg!(debug_assertions) {
    let dev_ports = vec![3000, 8000, 8080, 8081, 5173];
    let mut origins = Vec::new();

    for port in dev_ports {
        origins.push(format!("http://localhost:{port}").parse().unwrap());
        origins.push(format!("http://127.0.0.1:{port}").parse().unwrap());
    }

    CorsLayer::new()
        .allow_origin(origins)
        .allow_headers([header::CONTENT_TYPE])
        .allow_methods([Method::GET, Method::POST])
    // } else {
    //     // Production CORS - replace with specific origins
    //     CorsLayer::permissive()
    // }
}

pub mod handlers {
    //! Handlers for the API routes
    pub mod v1 {
        pub mod session {
            use std::sync::Arc;

            use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

            use crate::services::SessionService;
            use kiko::data::CreateSession;

            /// Handler to create a new session
            pub async fn create(
                State(state): State<Arc<crate::AppState>>,
                Json(payload): Json<CreateSession>,
            ) -> impl IntoResponse {
                match state.sessions.create(payload).await {
                    Ok(session) => (StatusCode::CREATED, Json(session)).into_response(),
                    Err(_) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to create session",
                    )
                        .into_response(),
                }
            }

            /// Handler to get a session by ID
            pub async fn get(
                State(state): State<Arc<crate::AppState>>,
                axum::extract::Path(session_id): axum::extract::Path<String>,
            ) -> impl IntoResponse {
                match state.sessions.get(&session_id.into()).await {
                    Ok(session) => (StatusCode::OK, Json(session)).into_response(),
                    Err(_) => (StatusCode::NOT_FOUND, "Session not found").into_response(),
                }
            }
        }

        pub mod websocket {
            use axum::{
                extract::{
                    ConnectInfo, State,
                    ws::{self, WebSocket, WebSocketUpgrade},
                },
                response::Response,
            };
            use kiko::errors::WebSocketError;
            use kiko::{id::SessionId, tracing};
            use kiko::{log, serde_json};
            use tokio::sync::mpsc;

            use std::{net::SocketAddr, sync::Arc};

            use crate::services::SessionService;

            #[derive(Debug)]
            pub enum WebSocketResponse {
                Success(String),
                SubscriptionStarted,
                None,
            }

            pub struct ConnectionState {
                session_id: Option<SessionId>,
                task_handle: Option<tokio::task::JoinHandle<()>>,
                outbound_rx: Option<mpsc::UnboundedReceiver<String>>,
            }

            impl ConnectionState {
                pub fn new() -> Self {
                    Self {
                        session_id: None,
                        task_handle: None,
                        outbound_rx: None,
                    }
                }

                pub fn is_subscribed(&self) -> bool {
                    self.task_handle.is_some()
                }

                pub fn cleanup_subscription(&mut self) {
                    if let Some(handle) = &self.task_handle {
                        if !handle.is_finished() {
                            handle.abort();
                        }
                    }
                    self.task_handle = None;
                    self.outbound_rx = None;
                }
            }

            impl Default for ConnectionState {
                fn default() -> Self {
                    Self::new()
                }
            }

            async fn handle_create_session(
                _create: &kiko::data::CreateSession,
                _state: &Arc<crate::AppState>,
            ) -> Result<WebSocketResponse, WebSocketError> {
                log::info!("Creating session: {:?}", _create);
                // TODO: Implement session creation logic
                Ok(WebSocketResponse::Success(
                    "Session creation not implemented".to_string(),
                ))
            }

            async fn handle_join_session(
                join: &kiko::data::JoinSession,
                state: &Arc<crate::AppState>,
                conn_state: &mut ConnectionState,
            ) -> Result<WebSocketResponse, WebSocketError> {
                log::info!("Joining session: {:?}", join);

                if conn_state.is_subscribed() {
                    return Err(WebSocketError::AlreadySubscribed);
                }

                let session_id: SessionId = join.session_id.clone().into();

                // Check if the session exists
                if state.sessions.get(&session_id).await.is_err() {
                    return Err(WebSocketError::SessionNotFound(join.session_id.clone()));
                }

                conn_state.session_id = Some(session_id.clone());

                // Subscribe to the session
                let _ = state.pub_sub.subscribe(session_id.clone()).await;

                // Create a channel for sending messages to the WebSocket
                let (outbound_tx, rx) = mpsc::unbounded_channel::<String>();
                conn_state.outbound_rx = Some(rx);

                // Spawn a task to listen for messages and notify the WebSocket
                let task_handle = tokio::spawn({
                    let state = state.clone();
                    let session_id = session_id.clone();
                    async move {
                        let notifier = state.pub_sub.subscribe(session_id.clone()).await;
                        loop {
                            notifier.notified().await;
                            if let Some(msg) = state.pub_sub.consume_event(&session_id).await {
                                if let Ok(json) = serde_json::to_string(&*msg) {
                                    if outbound_tx.send(json).is_err() {
                                        break; // Channel closed
                                    }
                                }
                            }
                        }
                    }
                });

                conn_state.task_handle = Some(task_handle);

                Ok(WebSocketResponse::SubscriptionStarted)
            }

            async fn handle_add_participant(
                add: &kiko::data::AddParticipant,
                _state: &Arc<crate::AppState>,
            ) -> Result<WebSocketResponse, WebSocketError> {
                log::info!("Adding participant: {:?}", add);
                // TODO: Implement participant addition logic
                Ok(WebSocketResponse::Success(
                    "Participant addition not implemented".to_string(),
                ))
            }

            async fn handle_remove_participant(
                remove: &kiko::data::RemoveParticipant,
                _state: &Arc<crate::AppState>,
            ) -> Result<WebSocketResponse, WebSocketError> {
                log::info!("Removing participant: {:?}", remove);
                // TODO: Implement participant removal logic
                Ok(WebSocketResponse::Success(
                    "Participant removal not implemented".to_string(),
                ))
            }

            async fn handle_session_update(
                update: &kiko::data::Session,
                _state: &Arc<crate::AppState>,
            ) -> Result<WebSocketResponse, WebSocketError> {
                log::info!("Session update: {:?}", update);
                // TODO: Implement session update logic
                Ok(WebSocketResponse::Success(
                    "Session update not implemented".to_string(),
                ))
            }

            async fn send_error(socket: &mut WebSocket, error: &WebSocketError) -> bool {
                let error_msg = error.to_string();
                log::error!("{}", error_msg);

                if let Err(e) = socket.send(ws::Message::Text(error_msg.into())).await {
                    log::error!("Failed to send error message: {}", e);
                    return false; // Connection broken
                }
                true
            }

            async fn send_response(socket: &mut WebSocket, response: WebSocketResponse) -> bool {
                match response {
                    WebSocketResponse::Success(msg) => {
                        if let Err(e) = socket.send(ws::Message::Text(msg.into())).await {
                            log::error!("Failed to send success message: {}", e);
                            return false;
                        }
                    }
                    WebSocketResponse::SubscriptionStarted => {
                        log::debug!("Subscription started successfully");
                        // No message sent to client for this response type
                    }
                    WebSocketResponse::None => {
                        // No response needed
                    }
                }
                true
            }

            /// Handler to upgrade HTTP connection to WebSocket
            pub async fn upgrade(
                ws: WebSocketUpgrade,
                ConnectInfo(addr): ConnectInfo<SocketAddr>,
                State(state): State<Arc<crate::AppState>>,
            ) -> Response {
                // Pass the client address to the handler
                ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
            }

            /// Handle WebSocket connection
            #[tracing::instrument(name = "websocket", skip(socket, state))]
            async fn handle_socket(
                mut socket: WebSocket,
                client_addr: SocketAddr,
                state: Arc<crate::AppState>,
            ) {
                log::debug!("Connection established");
                let mut conn_state = ConnectionState::new();

                loop {
                    tokio::select! {
                        // Handle incoming WebSocket messages
                        msg = socket.recv() => {
                            if !handle_incoming_message(msg, &mut socket, &mut conn_state, &state, client_addr).await {
                                break;
                            }
                        }

                        // Handle outbound messages from the subscription task
                        outbound_msg = async {
                            match &mut conn_state.outbound_rx {
                                Some(rx) => rx.recv().await,
                                None => std::future::pending().await,
                            }
                        } => {
                            if !handle_outbound_message(outbound_msg, &mut socket).await {
                                break;
                            }
                        }
                    }
                }

                log::debug!("Cleaning up WebSocket connection");
                cleanup_connection(&mut conn_state, &state).await;
                log::debug!("Connection ended");
            }

            async fn handle_incoming_message(
                msg: Option<Result<ws::Message, axum::Error>>,
                socket: &mut WebSocket,
                conn_state: &mut ConnectionState,
                state: &Arc<crate::AppState>,
                client_addr: SocketAddr,
            ) -> bool {
                match msg {
                    Some(Ok(ws::Message::Text(text))) => {
                        handle_text_message(
                            text.to_string(),
                            socket,
                            conn_state,
                            state,
                            client_addr,
                        )
                        .await
                    }
                    Some(Ok(ws::Message::Close(_))) => {
                        log::debug!("Connection closed by client");
                        false
                    }
                    Some(Err(e)) => {
                        log::error!("WebSocket error: {}", e);
                        false
                    }
                    None => {
                        log::debug!("WebSocket connection closed");
                        false
                    }
                    _ => {
                        // Handle other message types (Binary, Ping, Pong) if needed
                        true
                    }
                }
            }

            async fn handle_text_message(
                text: String,
                socket: &mut WebSocket,
                conn_state: &mut ConnectionState,
                state: &Arc<crate::AppState>,
                client_addr: SocketAddr,
            ) -> bool {
                let session_msg = match serde_json::from_str::<kiko::data::SessionMessage>(&text) {
                    Ok(msg) => msg,
                    Err(e) => {
                        let error = WebSocketError::InvalidMessage(e.to_string());
                        return send_error(socket, &error).await;
                    }
                };

                log::debug!("Received message from {}: {:?}", client_addr, session_msg);

                let result = match &session_msg {
                    kiko::data::SessionMessage::CreateSession(create) => {
                        handle_create_session(create, state).await
                    }
                    kiko::data::SessionMessage::JoinSession(join) => {
                        handle_join_session(join, state, conn_state).await
                    }
                    kiko::data::SessionMessage::AddParticipant(add) => {
                        handle_add_participant(add, state).await
                    }
                    kiko::data::SessionMessage::RemoveParticipant(remove) => {
                        handle_remove_participant(remove, state).await
                    }
                    kiko::data::SessionMessage::SessionUpdate(update) => {
                        handle_session_update(update, state).await
                    }
                };

                match result {
                    Ok(response) => send_response(socket, response).await,
                    Err(error) => send_error(socket, &error).await,
                }
            }

            async fn handle_outbound_message(
                outbound_msg: Option<String>,
                socket: &mut WebSocket,
            ) -> bool {
                match outbound_msg {
                    Some(json) => {
                        if let Err(e) = socket.send(ws::Message::Text(json.into())).await {
                            log::error!("Failed to send message to WebSocket: {}", e);
                            false
                        } else {
                            true
                        }
                    }
                    None => {
                        log::debug!("Outbound channel closed");
                        false
                    }
                }
            }

            async fn cleanup_connection(
                conn_state: &mut ConnectionState,
                state: &Arc<crate::AppState>,
            ) {
                conn_state.cleanup_subscription();

                if let Some(session_id) = &conn_state.session_id {
                    state.pub_sub.cleanup_session(session_id).await;
                }
            }
        }

        pub mod health {
            use std::sync::Arc;

            use axum::{Json, extract::State};
            use kiko::log;
            use kiko::serde_json::{Value, json};

            use crate::services::SessionService;

            fn uptime_seconds(started_at: chrono::DateTime<chrono::Utc>) -> i64 {
                (chrono::Utc::now() - started_at).num_seconds()
            }

            fn human_readable_uptime(started_at: chrono::DateTime<chrono::Utc>) -> String {
                let uptime_duration: chrono::TimeDelta =
                    chrono::Utc::now().signed_duration_since(started_at);

                let uptime_seconds = uptime_duration.num_seconds();
                let days = uptime_duration.num_days();
                let hours = (uptime_seconds % 86400) / 3600;
                let minutes = (uptime_seconds % 3600) / 60;
                let secs = uptime_seconds % 60;

                if days > 0 {
                    format!("{days}d {hours}h {minutes}m {secs}s")
                } else if hours > 0 {
                    format!("{hours}h {minutes}m {secs}s")
                } else if minutes > 0 {
                    format!("{minutes}m {secs}s")
                } else {
                    format!("{secs}s")
                }
            }

            fn service_uptime(started_at: chrono::DateTime<chrono::Utc>) -> (i64, String) {
                let seconds = uptime_seconds(started_at);
                let human = human_readable_uptime(started_at);
                (seconds, human)
            }

            pub async fn get(State(state): State<Arc<crate::AppState>>) -> Json<Value> {
                let session_count = state.sessions.list().await.unwrap_or_default().len();
                let (seconds, human) = service_uptime(state.started_at);

                let health_json = json!({
                    "status": "healthy",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "started_at": state.started_at.to_rfc3339(),
                    "uptime": {
                        "seconds": seconds,
                        "human": human
                    },
                    "services": {
                        "sessions": "up",
                        "active_sessions": session_count
                    }
                });

                log::info!("Health check: {}", health_json);

                Json(health_json)
            }
        }
    }
}
