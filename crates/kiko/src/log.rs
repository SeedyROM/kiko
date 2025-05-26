pub use tracing::{debug, error, info, trace, warn};

use crate::errors::LogError;

#[cfg(target_arch = "wasm32")]
/// Setup the logging system for the application for WASM.
/// This function will install the [`tracing-web`] logging system.
pub fn setup() -> Result<(), LogError> {
    use tracing_subscriber::fmt::format::{FmtSpan, Pretty};
    use tracing_subscriber::fmt::time::UtcTime;
    use tracing_subscriber::layer::SubscriberExt;

    use tracing_subscriber::util::SubscriberInitExt;

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_timer(UtcTime::rfc_3339())
        .with_writer(tracing_web::MakeConsoleWriter)
        .with_span_events(FmtSpan::ACTIVE);
    let perf_layer = tracing_web::performance_layer().with_details_from_fields(Pretty::default());

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
/// Setup the logging system for the application.
/// This function will install the [`color_eyre`] error reporting system
/// and the [`tracing-subscriber`] logging system.
/// It will also set the `RUST_LIB_BACKTRACE`` environment variable to `1`
/// and the `RUST_LOG`` environment variable to `"info"`.
/// If the environment variables are not set, they will be set to the default values.
/// If the color_eyre or tracing-subscriber installation fails,
/// an error will be returned.
pub fn setup() -> Result<(), LogError> {
    use tracing_subscriber::EnvFilter;

    // Get / set backtrace
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        unsafe { std::env::set_var("RUST_LIB_BACKTRACE", "1") }
    }
    // Install color_eyre
    color_eyre::install().map_err(|e: color_eyre::Report| LogError::ColorEyre(e))?;

    // Get/set the log level
    if std::env::var("RUST_LOG").is_err() {
        unsafe { std::env::set_var("RUST_LOG", "tracing=info,warp=debug,kiko_backend=debug") }
    }
    // Setup tracing and tracing-subscriber
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init()
        .map_err(LogError::TracingSubscriber)?;

    Ok(())
}
