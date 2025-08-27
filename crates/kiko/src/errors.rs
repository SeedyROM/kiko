//! Shared error types and utilities for the kiko project.
pub use color_eyre::Report;

#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("Failed to install color_eyre")]
    ColorEyre(#[from] color_eyre::Report),
    #[error("Failed to install tracing-subscriber")]
    TracingSubscriber(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
    #[error("Session {0} not found")]
    SessionNotFound(String),
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),
    #[error("Already subscribed to a session")]
    AlreadySubscribed,
    #[error("Failed to serialize message: {0}")]
    SerializationFailed(#[from] serde_json::Error),
    #[error("Failed to send message")]
    SendFailed,
    #[error("Communication channel closed")]
    ChannelClosed,
    #[error("Not subscribed to any session")]
    NotSubscribed,
}
