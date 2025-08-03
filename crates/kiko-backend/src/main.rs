use std::sync::Arc;

use axum::{
    Router,
    http::{Method, header},
    routing::post, // Removed unused 'get' import
};
use tokio::signal;
use tower_http::cors::CorsLayer;

use kiko::errors::Report;
use kiko::log;

struct AppState {
    // sessions: Arc<Mutex<Sessions>>,
}

#[tokio::main]
async fn main() -> Result<(), Report> {
    // Setup logging
    kiko::log::setup()?;

    // Add application state
    let app_state = Arc::new(AppState {
         // sessions: Arc::new(Mutex::new(Sessions::new())),
    });

    // Setup the routes
    let app = setup_routes(app_state);

    // Setup the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3030").await?;
    log::info!("Starting server on http://{}", listener.local_addr()?);
    log::info!("Press Ctrl+C to stop the server");

    // Start the server with graceful shutdown
    axum::serve(listener, app)
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
    let api_routes = Router::new()
        .route("/session", post(handlers::v1::session::create))
        .with_state(app_state);

    Router::new()
        .nest("/api/v1", api_routes)
        .layer(cors_layer())
        .layer(tower_http::trace::TraceLayer::new_for_http())
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
            origins.push(format!("http://localhost:{}", port).parse().unwrap());
            origins.push(format!("http://127.0.0.1:{}", port).parse().unwrap());
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
        pub mod session {
            use axum::Json;
            use kiko::data::{CreateSessionBody, Session};
            // use std::sync::{Arc};

            /// Handler to create a new session
            pub async fn create(
                // State(state): State<Arc<crate::AppState>>,
                Json(payload): Json<CreateSessionBody>,
            ) -> Json<Session> {
                // let CreateSessionBody { name, duration } = payload;
                // let mut state = state.lock().expect("Failed to lock state");
                // let sessions = state.sessions.lock().expect("Failed to lock sessions");

                Json(Session::new(
                    "session_id".to_string(), // Replace with actual session ID generation logic
                    payload.name,
                    payload.duration,
                ))
            }
        }
    }
}
