pub mod services;

use std::sync::Arc;

use axum::{
    Router,
    http::{Method, header},
    routing::{get, post},
};
use tokio::signal;
use tower_http::cors::CorsLayer;

use kiko::errors::Report;
use kiko::log;

use crate::services::SessionServiceInMemory;

pub struct AppState {
    sessions: SessionServiceInMemory,
}

#[tokio::main]
async fn main() -> Result<(), Report> {
    // Setup logging
    kiko::log::setup()?;

    // Add application state
    let app_state = Arc::new(AppState {
        sessions: SessionServiceInMemory::new(),
    });

    // Setup the routes
    let app = setup_routes(app_state);

    // Setup the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3030").await?;
    log::info!("Starting server on http://{}", listener.local_addr()?);
    log::info!("Press Ctrl+C to stop the server");

    // Start the server with graceful shutdown
    // IMPORTANT: Use into_make_service_with_connect_info to preserve client connection info
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
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
        signal::unix::signal(signal::unix::SignalKind::terminate())
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
        .route("/hello", get(handlers::v1::hello::get))
        .route("/session", post(handlers::v1::session::create))
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
    if cfg!(debug_assertions) {
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
    } else {
        // Production CORS - replace with specific origins
        CorsLayer::permissive()
    }
}

pub mod handlers {
    //! Handlers for the API routes
    pub mod v1 {
        pub mod hello {
            use axum::Json;
            use kiko::serde_json::{Value, json};

            /// Handler to return a simple hello message
            pub async fn get() -> Json<Value> {
                let timestamp = chrono::Utc::now().to_rfc3339();
                Json(json!({
                    "message": format!("Hello, world! Current time: {timestamp}"),
                    "status": "ok"
                }))
            }
        }

        pub mod session {
            use std::sync::Arc;

            use axum::{Json, extract::State, response::IntoResponse};

            use crate::services::SessionService;
            use kiko::data::CreateSession;

            /// Handler to create a new session
            pub async fn create(
                State(state): State<Arc<crate::AppState>>,
                Json(payload): Json<CreateSession>,
            ) -> impl IntoResponse {
                match state.sessions.create(payload).await {
                    Ok(session) => (axum::http::StatusCode::CREATED, Json(session)).into_response(),
                    Err(_) => (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to create session",
                    )
                        .into_response(),
                }
            }
        }

        pub mod websocket {
            use axum::{
                extract::{
                    ConnectInfo,
                    ws::{self, WebSocket, WebSocketUpgrade},
                },
                response::Response,
            };
            use kiko::log;
            use kiko::tracing;

            use std::net::SocketAddr;

            /// Handler to upgrade HTTP connection to WebSocket
            pub async fn upgrade(
                ws: WebSocketUpgrade,
                ConnectInfo(addr): ConnectInfo<SocketAddr>,
            ) -> Response {
                // Pass the client address to the handler
                ws.on_upgrade(move |socket| handle_socket(socket, addr))
            }

            /// Handle WebSocket connection
            #[tracing::instrument(name = "websocket", skip(socket))]
            async fn handle_socket(mut socket: WebSocket, client_addr: SocketAddr) {
                log::debug!("Connection established");

                // Send a welcome message with client info
                let welcome_msg = format!(
                    "Hello WebSocket! Connected from {}:{}",
                    client_addr.ip(),
                    client_addr.port()
                );

                if let Err(e) = socket.send(ws::Message::Text(welcome_msg.into())).await {
                    log::error!("Failed to send welcome message: {}", e);
                    return;
                }

                // Echo messages back to client
                while let Some(msg) = socket.recv().await {
                    match msg {
                        Ok(ws::Message::Text(text)) => {
                            log::debug!("Received text message: {}", text);
                            let response = format!("Echo: {text}");
                            if let Err(e) = socket.send(ws::Message::Text(response.into())).await {
                                log::error!("Failed to send echo response: {}", e);
                                break;
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

                log::debug!("Connection ended");
            }
        }

        pub mod health {
            use axum::{Json, extract::State};
            use kiko::serde_json::{Value, json};
            use std::sync::Arc;

            use crate::services::SessionService;

            pub async fn get(State(state): State<Arc<crate::AppState>>) -> Json<Value> {
                let session_count = state.sessions.list().await.unwrap_or_default().len();

                Json(json!({
                    "status": "healthy",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "services": {
                        "sessions": "up",
                        "active_sessions": session_count
                    },
                    "uptime": "todo" // Track app start time
                }))
            }
        }
    }
}
