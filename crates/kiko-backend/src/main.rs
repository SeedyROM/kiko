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
            use kiko::{id::SessionId, tracing};
            use kiko::{log, serde_json};

            use std::{net::SocketAddr, sync::Arc};

            use crate::services::SessionService;

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
                let mut session_id: Option<SessionId> = None;
                let mut task_handle = Option::<tokio::task::JoinHandle<()>>::None;

                // Echo messages back to client
                while let Some(msg) = socket.recv().await {
                    match msg {
                        Ok(ws::Message::Text(text)) => {
                            match serde_json::from_str::<kiko::data::SessionMessage>(&text) {
                                Ok(session_msg) => {
                                    log::debug!(
                                        "Received message from {}: {:?}",
                                        client_addr,
                                        session_msg
                                    );

                                    match &session_msg {
                                        kiko::data::SessionMessage::CreateSession(create) => {
                                            log::info!("Creating session: {:?}", create);
                                            // Handle creating session logic here
                                        }
                                        kiko::data::SessionMessage::JoinSession(join) => {
                                            log::info!("Joining session: {:?}", join);
                                            session_id = Some(join.session_id.clone().into());

                                            // Check if the session exists
                                            // If not, send an error message and continue
                                            if state
                                                .sessions
                                                .get(&join.session_id.clone().into())
                                                .await
                                                .is_err()
                                            {
                                                let error_msg = format!(
                                                    "Session {} not found",
                                                    join.session_id
                                                );
                                                log::error!("{}", error_msg);
                                                if let Err(e) = socket
                                                    .send(ws::Message::Text(error_msg.into()))
                                                    .await
                                                {
                                                    log::error!(
                                                        "Failed to send error message: {}",
                                                        e
                                                    );
                                                }
                                                session_id = None;
                                                continue;
                                            }

                                            // Check if already subscribed
                                            if task_handle.is_some() {
                                                log::warn!(
                                                    "Already subscribed to a session, ignoring join request"
                                                );
                                                continue;
                                            }

                                            // Handle joining session logic here
                                            let _ = state
                                                .pub_sub
                                                .subscribe(session_id.clone().unwrap())
                                                .await;

                                            // Spawn a task to listen for messages and notify the WebSocket
                                            task_handle = Some(tokio::spawn({
                                                let state = state.clone();
                                                let session_id = session_id.clone().unwrap();
                                                async move {
                                                    loop {
                                                        if let Some(msg) = state
                                                            .pub_sub
                                                            .consume_event(&session_id)
                                                            .await
                                                        {
                                                            log::info!(
                                                                "Publishing message to session {}: {:?}",
                                                                session_id,
                                                                msg
                                                            );
                                                        }
                                                    }
                                                }
                                            }));
                                        }
                                        kiko::data::SessionMessage::AddParticipant(add) => {
                                            log::info!("Adding participant: {:?}", add);
                                            // Handle adding participant logic here
                                        }
                                        kiko::data::SessionMessage::RemoveParticipant(remove) => {
                                            log::info!("Removing participant: {:?}", remove);
                                            // Handle removing participant logic here
                                        }
                                        kiko::data::SessionMessage::SessionUpdate(update) => {
                                            log::info!("Session update: {:?}", update);
                                            // Handle session update logic here
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to parse message: {}", e);
                                    if let Err(e) = socket
                                        .send(ws::Message::Text("Invalid message format".into()))
                                        .await
                                    {
                                        log::error!("Failed to send error message: {}", e);
                                        continue;
                                    }
                                }
                            }
                        }
                        Ok(ws::Message::Close(_)) => {
                            log::debug!("Connection closed by client");
                            break;
                        }
                        Err(e) => {
                            log::error!("WebSocket error: {}", e);
                            break;
                        }
                        _ => {
                            // Handle other message types (Binary, Ping, Pong) if needed
                        }
                    }
                }

                log::debug!("Cleaning up WebSocket connection");

                // Clean up on disconnect
                if let Some(handle) = &task_handle {
                    if handle.is_finished() {
                        log::debug!("Notification task finished");
                    } else {
                        log::debug!("Aborting notification task");
                        handle.abort();
                    }
                }

                // Clean up session subscription
                if let Some(session_id) = &session_id {
                    state.pub_sub.cleanup_session(&session_id.clone()).await;
                }

                log::debug!("Connection ended");
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
