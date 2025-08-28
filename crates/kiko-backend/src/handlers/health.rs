use std::sync::Arc;

use axum::{Json, extract::State};
use kiko::data::{HealthResponse, HealthStatus, ServiceInfo, UptimeInfo};
use kiko::log;

use crate::services::SessionService;

fn uptime_seconds(started_at: chrono::DateTime<chrono::Utc>) -> i64 {
    (chrono::Utc::now() - started_at).num_seconds()
}

fn human_readable_uptime(started_at: chrono::DateTime<chrono::Utc>) -> String {
    let uptime_duration: chrono::TimeDelta = chrono::Utc::now().signed_duration_since(started_at);

    let uptime_seconds = uptime_duration.num_seconds();
    let days = uptime_duration.num_days();
    let hours = (uptime_seconds % 86400) / 3600;
    let minutes = (uptime_seconds % 3600) / 60;
    let secs = uptime_seconds % 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m {secs}s")
    } else if hours > 0 {
        format!("{hours}h {minutes}m {secs}s")
    } else if minutes > 0 {
        format!("{minutes}m {secs}s")
    } else {
        format!("{secs}s")
    }
}

fn service_uptime(started_at: chrono::DateTime<chrono::Utc>) -> (i64, String) {
    let seconds = uptime_seconds(started_at);
    let human = human_readable_uptime(started_at);
    (seconds, human)
}

pub async fn get(State(state): State<Arc<crate::AppState>>) -> Json<HealthResponse> {
    let session_count = state.sessions.list().await.unwrap_or_default().len();
    let (seconds, human) = service_uptime(state.started_at);

    let health_response = HealthResponse {
        status: HealthStatus::Healthy,
        timestamp: chrono::Utc::now().to_rfc3339(),
        started_at: state.started_at.to_rfc3339(),
        uptime: UptimeInfo { seconds, human },
        services: ServiceInfo {
            sessions: "up".to_string(),
            active_sessions: session_count,
        },
    };

    log::info!("Health check: {:?}", health_response);

    Json(health_response)
}
