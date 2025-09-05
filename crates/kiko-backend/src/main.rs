//! Kiko Backend Server
//!
//! A real-time session management backend built with Axum and WebSockets.
//! Provides REST APIs for session management and WebSocket connections for live updates.

use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    http::{Method, header},
    routing::{get, post},
};
use tokio::{net::TcpListener, signal};
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::cors::CorsLayer;

use kiko::errors::Report;
use kiko::log;
use kiko_backend::{AppState, handlers, messaging::PubSub, services::SessionServiceInMemory};

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
    let port = std::env::var("KIKO_BACKEND_PORT")
        .unwrap_or_else(|_| "3030".to_string())
        .parse::<u16>()
        .unwrap_or(3030);
    let ipv6 = SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], port));
    let listener = TcpListener::bind(ipv6).await?;
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
    // Session endpoints
    let mut session_routes = Router::new().route("/session", post(handlers::v1::session::create));

    // Other API endpoints
    let mut other_routes = Router::new()
        .route("/session/{session_id}", get(handlers::v1::session::get))
        .route("/ws", get(handlers::v1::websocket::upgrade));

    // Only apply rate limiting in production (not in debug builds or tests)
    if !cfg!(debug_assertions) {
        // General rate limiting config: 60 requests per minute with burst of 10
        let general_config = GovernorConfigBuilder::default()
            .per_second(1) // 1 request per second = 60 per minute
            .burst_size(10) // Allow bursts of up to 10 requests
            .finish()
            .unwrap();

        // Stricter rate limiting for session creation: burst of 3
        let session_config = GovernorConfigBuilder::default()
            .per_second(1) // Base rate: 1 per second
            .burst_size(3) // Smaller burst for session creation
            .finish()
            .unwrap();

        session_routes = session_routes.layer(GovernorLayer::new(session_config));
        other_routes = other_routes.layer(GovernorLayer::new(general_config));
    }

    let api_routes = Router::new()
        .merge(session_routes)
        .merge(other_routes)
        .with_state(app_state.clone());

    Router::new()
        .route("/health", get(handlers::health::get))
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
