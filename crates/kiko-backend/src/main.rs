use tokio::signal;

use kiko::errors::Report;
use kiko::log;

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

mod filters {
    use warp::Filter;

    use super::handlers;

    pub fn setup_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        warp::path!("hello" / ..)
            .and(warp::get())
            .and_then(handlers::hello)
    }
}

mod handlers {
    use std::convert::Infallible;

    use kiko::data::HelloWorld;

    pub async fn hello() -> Result<impl warp::Reply, Infallible> {
        let response = HelloWorld {
            message: "Hello, world!".to_string(),
        };
        Ok(warp::reply::json(&response))
    }
}
