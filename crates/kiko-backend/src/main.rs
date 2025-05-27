use axum::{
    Router,
    http::{Method, header},
    routing::get,
};
use tokio::signal;
use tower_http::cors::CorsLayer;

use kiko::errors::Report;
use kiko::log;

#[tokio::main]
async fn main() -> Result<(), Report> {
    // Setup logging
    kiko::log::setup()?;

    // Setup the routes
    let app = setup_routes();

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

fn setup_routes() -> Router {
    let api_routes = Router::new().route("/hello", get(handlers::v1::hello));

    Router::new()
        .nest("/api/v1", api_routes)
        .layer(cors_layer())
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

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

mod handlers {
    pub mod v1 {
        use axum::response::Json;
        use kiko::data::HelloWorld;

        /// Handle the /hello route
        pub async fn hello() -> Json<HelloWorld> {
            let response = HelloWorld {
                message: "Hello, from the backend baby!".to_string(),
            };
            Json(response)
        }
    }
}
