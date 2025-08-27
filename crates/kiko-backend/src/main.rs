//! Kiko Backend Server
//!
//! A real-time session management backend built with Axum and WebSockets.
//! Provides REST APIs for session management and WebSocket connections for live updates.

pub mod handlers;
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

/// Shared application state containing services and configuration.
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
