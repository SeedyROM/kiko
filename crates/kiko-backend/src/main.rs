use tokio::signal;

use kiko::errors::Report;
use kiko::log;

#[tokio::main]
async fn main() -> Result<(), Report> {
    // Setup logging
    kiko::log::setup()?;

    // Setup the routes
    let routes = filters::setup_routes();

    // Setup the server
    let (addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), shutdown_signal());
    log::info!("Starting server on http://{}", addr);
    log::info!("Press Ctrl+C to stop the server");

    // Start the server
    server.await;

    // Wait for the shutdown signal
    log::info!("Shutting down server");

    Ok(())
}

/// Wait for a shutdown signal (Ctrl+C or SIGTERM)
/// and log a message when it is received.
/// This function uses `tokio::signal` to listen for signals
/// and `tokio::select!` to wait for either the Ctrl+C signal
/// or the SIGTERM signal.
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

mod filters {
    use warp::Filter;

    use super::handlers;

    /// Setup the routes for the server
    pub fn setup_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        // Accepted CORS origins depending on the environment
        let origins = if cfg!(debug_assertions) {
            // Common development ports
            let dev_ports = vec![3000, 8000, 8080, 8081, 5173];

            let mut allowed_origins = Vec::new();
            for port in dev_ports {
                allowed_origins.push(format!("http://localhost:{}", port));
                allowed_origins.push(format!("http://127.0.0.1:{}", port));
            }
            allowed_origins
        } else {
            // Production origins
            // TODO(SeedyROM): None for now, but we should add the production origins here
            // e.g. vec!["https://myapp.com".to_string()]
            vec![]
        };

        let cors = warp::cors()
            .allow_origins(origins.iter().map(|s| s.as_str()))
            .allow_headers(vec!["content-type"])
            .allow_methods(vec!["GET", "POST"]);

        warp::path!("hello")
            .and(warp::get())
            .and_then(handlers::hello)
            .with(cors)
    }
}

mod handlers {
    use std::convert::Infallible;

    use kiko::data::HelloWorld;

    /// Handle the /hello route
    pub async fn hello() -> Result<impl warp::Reply, Infallible> {
        let response = HelloWorld {
            message: "Hello, from the backend baby!".to_string(),
        };
        Ok(warp::reply::json(&response))
    }
}
