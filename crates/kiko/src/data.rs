//! Data structures used between the frontend and backend of the Kiko application.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::id::{ParticipantId, SessionId};

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

    pub fn participants(&self) -> &Vec<Participant> {
        &self.members
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
pub struct AddParticipant {
    pub session_id: String,
    pub participant_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoveParticipant {
    pub session_id: String,
    pub participant_id: String,
}
