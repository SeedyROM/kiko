//! Shared error types and utilities for the kiko project.
pub use color_eyre::Report;

#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("Failed to install color_eyre")]
    ColorEyre(#[from] color_eyre::Report),
    #[error("Failed to install tracing-subscriber")]
    TracingSubscriber(#[from] Box<dyn std::error::Error + Send + Sync>),
}
