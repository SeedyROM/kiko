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
    use super::handlers;
    use warp::{Filter, filters};

    fn cors() -> filters::cors::Builder {
        let origins = if cfg!(debug_assertions) {
            let dev_ports = vec![3000, 8000, 8080, 8081, 5173];
            let mut allowed_origins = Vec::new();
            for port in dev_ports {
                allowed_origins.push(format!("http://localhost:{}", port));
                allowed_origins.push(format!("http://127.0.0.1:{}", port));
            }
            allowed_origins
        } else {
            // Production origins - add your domains here
            vec![]
        };

        warp::cors()
            .allow_origins(origins.iter().map(|s| s.as_str()))
            .allow_headers(vec!["content-type"])
            .allow_methods(vec!["GET", "POST"])
    }

    // Helper function to create the base API path
    fn api_v1() -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
        warp::path!("api" / "v1")
    }

    // Individual route definitions
    fn hello_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        api_v1()
            .and(warp::path("hello"))
            .and(warp::get())
            .and_then(handlers::v1::hello)
    }

    /// Setup the routes for the server and configure CORS
    /// Setup the routes for the server and configure CORS
    pub fn setup_routes()
    -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let routes = hello_route()
            // Add more routes here with .or()
            // .or(another_route())
            ;

        routes.with(cors()).with(warp::trace::request())
    }
}

mod handlers {
    pub mod v1 {
        use kiko::data::HelloWorld;
        use std::convert::Infallible;

        /// Handle the /hello route
        pub async fn hello() -> Result<impl warp::Reply, Infallible> {
            let response = HelloWorld {
                message: "Hello, from the backend baby!".to_string(),
            };
            Ok(warp::reply::json(&response))
        }
    }
}
