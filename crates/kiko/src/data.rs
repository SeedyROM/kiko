//! Data structures and message types for session management.
//!
//! This module contains the core data types used throughout the Kiko application
//! for managing sessions, participants, and communication between frontend and backend.
//! All types are serializable and designed to work seamlessly with JSON APIs and WebSocket messaging.

use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};

use crate::id::{ParticipantId, SessionId};
use crate::log;

/// Represents a participant in a session.
///
/// A participant has a unique ID and a display name. Participants can join and leave
/// sessions dynamically, and their information is synchronized across all clients.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Participant {
    id: ParticipantId,
    name: String,
}

impl Participant {
    pub fn new(id: ParticipantId, name: String) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> &ParticipantId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Session {
    pub id: SessionId,
    name: String,
    started: u64,
    duration: Duration,
    members: Vec<Participant>,
    current_topic: String,
    current_points: HashMap<ParticipantId, Option<u32>>,
    hide_points: bool,
}

impl Session {
    pub fn new(name: String, duration: Duration) -> Self {
        let started = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
        let id = SessionId::new();

        Self {
            id,
            name,
            started,
            duration,
            members: Vec::new(),
            current_topic: String::new(),
            current_points: HashMap::new(),
            hide_points: false,
        }
    }

    pub fn add_participant(&mut self, participant: Participant) {
        self.members.push(participant);
    }

    pub fn remove_participant(&mut self, participant_id: &ParticipantId) {
        self.members.retain(|p| &p.id != participant_id);
    }

    pub fn is_active(&self) -> bool {
        let elapsed = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs() - self.started;
        elapsed < self.duration.as_secs()
    }

    pub fn remaining_time(&self) -> Duration {
        let elapsed = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs() - self.started;
        if elapsed < self.duration.as_secs() {
            Duration::from_secs(self.duration.as_secs() - elapsed)
        } else {
            Duration::from_secs(0)
        }
    }

    pub fn set_topic(&mut self, topic: String) {
        self.current_topic = topic;
    }

    pub fn clear_points(&mut self) {
        self.current_points.clear();
    }

    pub fn toggle_hide_points(&mut self) {
        self.hide_points = !self.hide_points;
    }

    pub fn point(&mut self, participant_id: &ParticipantId, points: Option<u32>) {
        // Validate participant exists
        if !self.members.iter().any(|p| &p.id == participant_id) {
            log::warn!(
                "Participant ID {:?} not found in session {:?}",
                participant_id,
                self.id
            );
            return;
        }
        // Update points
        self.current_points.insert(participant_id.clone(), points);
    }

    pub fn participants(&self) -> &Vec<Participant> {
        &self.members
    }

    pub fn current_topic(&self) -> &String {
        &self.current_topic
    }

    pub fn current_points(&self) -> &HashMap<ParticipantId, Option<u32>> {
        &self.current_points
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn started(&self) -> u64 {
        self.started
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateSession {
    pub name: String,
    pub duration: Duration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JoinSession {
    pub session_id: String,
    pub participant_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubscribeToSession {
    pub session_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddParticipant {
    pub session_id: String,
    pub participant_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoveParticipant {
    pub session_id: String,
    pub participant_id: String,
}

/// Health status enumeration.
///
/// Represents the overall health state of the server.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Dead,
}

/// Health check response structure.
///
/// Contains server health information including status, uptime, and service states.
/// Used by the `/health` endpoint to provide structured health check data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub timestamp: String,
    pub started_at: String,
    pub uptime: UptimeInfo,
    pub services: ServiceInfo,
}

/// Uptime information in both seconds and human-readable format.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UptimeInfo {
    pub seconds: i64,
    pub human: String,
}

/// Service status information.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceInfo {
    pub sessions: String,
    pub active_sessions: usize,
}

/// Point a given task in the session.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PointSession {
    pub session_id: String,
    pub participant_id: String,
    pub points: Option<u32>,
}

/// WebSocket message types for session operations.
///
/// This enum defines all possible messages that can be sent between clients and the server
/// for session management operations. Messages are JSON-serialized for WebSocket transport.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SessionMessage {
    CreateSession(CreateSession),
    JoinSession(JoinSession),
    SubscribeToSession(SubscribeToSession),
    AddParticipant(AddParticipant),
    RemoveParticipant(RemoveParticipant),
    PointSession(PointSession),
    SetTopic(String),
    ClearPoints,
    SessionUpdate(Session),
    ToggleHidePoints,
}
