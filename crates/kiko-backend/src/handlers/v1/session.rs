use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use kiko::data::CreateSession;

use crate::services::SessionService;

/// Handler to create a new session
pub async fn create(
    State(state): State<Arc<crate::AppState>>,
    Json(payload): Json<CreateSession>,
) -> impl IntoResponse {
    match state.sessions.create(payload).await {
        Ok(session) => (StatusCode::CREATED, Json(session)).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create session",
        )
            .into_response(),
    }
}

/// Handler to get a session by ID
pub async fn get(
    State(state): State<Arc<crate::AppState>>,
    axum::extract::Path(session_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.sessions.get(&session_id.into()).await {
        Ok(session) => (StatusCode::OK, Json(session)).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Session not found").into_response(),
    }
}
