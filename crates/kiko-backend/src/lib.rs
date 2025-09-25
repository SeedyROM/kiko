pub mod handlers;
pub mod messaging;
pub mod services;

use crate::{messaging::PubSub, services::SessionServiceInMemory};
use chrono::DateTime;
use kiko::{data::SessionMessage, id::SessionId};

/// Shared application state containing services and configuration.
pub struct AppState {
    pub started_at: DateTime<chrono::Utc>,
    pub sessions: SessionServiceInMemory,
    pub pub_sub: PubSub<SessionId, SessionMessage>,
}
