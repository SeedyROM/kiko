use async_trait::async_trait;
use dashmap::DashMap;

use kiko::id::{ParticipantId, SessionId};

/// A trait for managing sessions and their participants.
///
/// This trait provides a complete interface for session lifecycle management,
/// including creation, retrieval, participant management, and cleanup. It is
/// designed to be implementation-agnostic, allowing for in-memory, database,
/// or other storage backends.
///
/// # Examples
///
/// ```rust
/// use kiko::id::SessionId;
/// use kiko::data::CreateSession;
///
/// async fn example_usage<S: SessionService>(service: &S) -> Result<(), S::Error> {
///     // Create a new session
///     let create_request = CreateSession {
///         name: "Team Meeting".to_string(),
///         duration: std::time::Duration::from_secs(3600),
///     };
///     let session = service.create(create_request).await?;
///     
///     // Add participants
///     let participant = service.join(&session.id, "Alice").await?;
///     
///     // List all sessions
///     let sessions = service.list().await?;
///     
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait SessionService<Session = kiko::data::Session, Participant = kiko::data::Participant> {
    /// The error type returned by operations on this service.
    type Error;

    /// Creates a new session with the given name and duration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let create_request = CreateSession {
    ///     name: "Daily Standup".to_string(),
    ///     duration: Duration::from_secs(1800), // 30 minutes
    /// };
    /// let session = service.create(create_request).await?;
    /// ```
    async fn create(&self, session: kiko::data::CreateSession) -> Result<Session, Self::Error>;

    /// Updates an existing session with new data.
    ///
    /// # Errors
    ///
    /// Returns an error if the session with the given ID doesn't exist.
    async fn update(
        &self,
        session_id: &SessionId,
        session: &Session,
    ) -> Result<Session, Self::Error>;

    /// Retrieves a session by its unique identifier.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let session = service.get(&session_id).await?;
    /// println!("Session: {} ({})", session.name, session.id);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if no session exists with the given ID.
    async fn get(&self, session_id: &SessionId) -> Result<Session, Self::Error>;

    /// Returns all active sessions.
    ///
    /// The returned vector may be empty if no sessions exist. For implementations
    /// with large numbers of sessions, consider using pagination instead of this method.
    async fn list(&self) -> Result<Vec<Session>, Self::Error>;

    /// Adds a participant to an existing session.
    ///
    /// Creates a new participant with the given name and assigns them a unique ID.
    /// The participant is immediately added to the session's participant list.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let participant = service.join(&session_id, "Bob").await?;
    /// println!("Participant {} joined session", participant.name);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the session doesn't exist.
    async fn join(
        &self,
        session_id: &SessionId,
        participant_name: &str,
    ) -> Result<Participant, Self::Error>;

    /// Removes a participant from a session.
    ///
    /// If the participant is not found in the session, this operation succeeds
    /// without error (idempotent behavior).
    ///
    /// # Errors
    ///
    /// Returns an error if the session doesn't exist.
    async fn leave(
        &self,
        session_id: &SessionId,
        participant_id: &ParticipantId,
    ) -> Result<(), Self::Error>;

    /// Ends a session and removes it completely.
    ///
    /// This operation removes all participants from the session and deletes all
    /// associated data. The session ID becomes invalid after this operation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// service.end(&session_id).await?;
    /// // Session is now deleted and participants are removed
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the session doesn't exist.
    async fn end(&self, session_id: &SessionId) -> Result<(), Self::Error>;
}

/// An in-memory implementation of the `SessionService` trait.
///
/// This implementation uses a `DashMap` to store sessions, allowing for concurrent access
/// and modifications. It is suitable for testing or applications where persistence is not required.
///
/// # Examples
/// ```rust
/// use kiko::data::*;
/// use kiko::id::*;
/// use kiko::services::*;
///
/// let service = SessionServiceInMemory::new();
/// // Create a new session
/// let create_request = CreateSession {
///     name: "Team Planning".to_string(),
///    duration: std::time::Duration::from_secs(3600), // 1 hour
/// };
/// let session = service.create(create_request).await.unwrap();
/// // Join a participant
/// let participant = service.join(&session.id, "Alice").await.unwrap();
/// // List all sessions
/// let sessions = service.list().await.unwrap();
/// assert_eq!(sessions.len(), 1);
/// assert_eq!(sessions[0].name, "Team Planning");
/// ```
pub struct SessionServiceInMemory {
    sessions: DashMap<SessionId, kiko::data::Session>,
}

impl SessionServiceInMemory {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }
}

impl Default for SessionServiceInMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionService for SessionServiceInMemory {
    type Error = kiko::errors::Report;

    async fn create(
        &self,
        session: kiko::data::CreateSession,
    ) -> Result<kiko::data::Session, Self::Error> {
        let new_session = kiko::data::Session::new(session.name, session.duration);
        let session_id = SessionId::from_string(new_session.id.clone());
        self.sessions.insert(session_id, new_session.clone());
        Ok(new_session)
    }

    async fn update(
        &self,
        session_id: &SessionId,
        session: &kiko::data::Session,
    ) -> Result<kiko::data::Session, Self::Error> {
        // Check if exists first, then update
        if self.sessions.contains_key(session_id) {
            self.sessions.insert(session_id.clone(), session.clone());
            Ok(session.clone())
        } else {
            Err(Self::Error::msg("Session not found"))
        }
    }

    async fn get(&self, session_id: &SessionId) -> Result<kiko::data::Session, Self::Error> {
        self.sessions
            .get(session_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| Self::Error::msg("Session not found"))
    }

    async fn list(&self) -> Result<Vec<kiko::data::Session>, Self::Error> {
        Ok(self
            .sessions
            .iter()
            .map(|entry| entry.value().clone())
            .collect())
    }

    async fn join(
        &self,
        session_id: &SessionId,
        participant_name: &str,
    ) -> Result<kiko::data::Participant, Self::Error> {
        let mut session_entry = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| Self::Error::msg("Session not found"))?;

        let participant_id = ParticipantId::new();
        let new_participant =
            kiko::data::Participant::new(participant_id.to_string(), participant_name.to_string());

        session_entry.add_participant(new_participant.clone());
        Ok(new_participant)
    }

    async fn leave(
        &self,
        session_id: &SessionId,
        participant_id: &ParticipantId,
    ) -> Result<(), Self::Error> {
        let mut session_entry = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| Self::Error::msg("Session not found"))?;

        session_entry.remove_participant(participant_id.to_string());
        Ok(())
    }

    async fn end(&self, session_id: &SessionId) -> Result<(), Self::Error> {
        self.sessions
            .remove(session_id)
            .map(|_| ())
            .ok_or_else(|| Self::Error::msg("Session not found"))
    }
}
