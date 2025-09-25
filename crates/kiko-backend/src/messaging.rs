//! A high-performance publish-subscribe messaging system for session-based communication.
//!
//! This module provides a thread-safe, async PubSub implementation optimized for
//! real-time messaging between WebSocket clients within sessions. It uses lock-free
//! data structures where possible to minimize contention and maximize throughput.
//!
//! # Architecture
//!
//! The PubSub system is built around two core concepts:
//! - **Notifiers**: Per-session notification mechanisms using [`tokio::sync::Notify`]
//! - **Events**: Per-session message storage using [`arc_swap::ArcSwap`] for lock-free reads
//!
//! # Example
//!
//! ```ignore
//! use kiko_backend::messaging::PubSub;
//! use kiko::{data::{SessionMessage, CreateSession}, id::SessionId};
//! use std::{sync::Arc, time::Duration};
//!
//! # #[tokio::main]
//! # async fn main() {
//! let pubsub = Arc::new(PubSub::new());
//! let session_id = SessionId::new();
//!
//! // Subscribe to events for a session
//! let notifier = pubsub.subscribe(session_id.clone());
//!
//! // Publish a message to the session
//! let create_session = CreateSession {
//!     name: "Test Session".to_string(),
//!     duration: Duration::from_secs(1800),
//! };
//! let message = SessionMessage::CreateSession(create_session);
//! pubsub.publish(session_id.clone(), message);
//!
//! // Wait for notification and consume the event
//! notifier.notified().await;
//! if let Some(event) = pubsub.consume_event(&session_id) {
//!     // Process the event
//! }
//! # }
//! ```

use std::{hash::Hash, sync::Arc};

use arc_swap::ArcSwap;
use dashmap::DashMap;
use tokio::sync::Notify;

/// Trait bounds for session identifier types.
pub trait IdType: Clone + Eq + Hash + Send + Sync {}
impl<T> IdType for T where T: Clone + Eq + Hash + Send + Sync {}

/// Trait bounds for message types used in the PubSub system.
pub trait MessageType: Clone + Send + Sync {}
impl<T> MessageType for T where T: Clone + Send + Sync {}

/// Event identifier type for tracking message sequence.
///
/// Currently unused but reserved for future message ordering and deduplication features.
pub type EventId = u64;

/// A high-performance, thread-safe publish-subscribe messaging system.
///
/// `PubSub` provides session-scoped messaging capabilities with the following characteristics:
/// - **Lock-free reads**: Uses [`ArcSwap`] for message storage to minimize read contention
/// - **Async notifications**: Leverages [`tokio::sync::Notify`] for efficient subscriber wake-up
/// - **Memory efficient**: Automatic cleanup prevents memory leaks from abandoned sessions
/// - **Single message semantics**: Each session holds at most one pending message
///
/// # Thread Safety
///
/// All methods are thread-safe and most are now synchronous due to lock-free design. The implementation uses:
/// - [`DashMap`] for lock-free concurrent access to notifier and event hashmaps
/// - [`ArcSwap`] for lock-free atomic message updates
/// - [`Arc<Notify>`] for efficient cross-task notifications
///
/// # Performance Characteristics
///
/// - **Subscribe**: O(1) amortized (DashMap insert)
/// - **Publish**: O(1) for message update + O(1) for lock-free notifier lookup
/// - **Get/Consume**: O(1) lock-free read + O(1) for lock-free DashMap access
/// - **Cleanup**: O(1) for both notifier and event removal
pub struct PubSub<Id, T>
where
    Id: IdType,
    T: MessageType,
{
    /// Per-session notification handles for waking up subscribers.
    notifiers: DashMap<Id, Arc<Notify>>,
    /// Per-session message storage using lock-free atomic swaps.
    events: DashMap<Id, ArcSwap<T>>,
}

impl<Id, T> PubSub<Id, T>
where
    Id: IdType,
    T: MessageType,
{
    /// Creates a new empty PubSub instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use kiko_backend::messaging::PubSub;
    /// use kiko::{data::SessionMessage, id::SessionId};
    ///
    /// let pubsub: PubSub<SessionId, SessionMessage> = PubSub::new();
    /// ```
    pub fn new() -> Self {
        Self {
            notifiers: DashMap::new(),
            events: DashMap::new(),
        }
    }

    /// Subscribes to notifications for a given session.
    ///
    /// Returns a [`Notify`] handle that will be signaled when new messages
    /// are published to the session. If a notifier already exists for the session,
    /// the same instance is returned (shared among multiple subscribers).
    ///
    /// # Arguments
    ///
    /// * `session_id` - The unique identifier of the session to subscribe to
    ///
    /// # Returns
    ///
    /// An [`Arc<Notify>`] that can be awaited with `.notified().await` to
    /// receive notifications when messages are published.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use kiko_backend::messaging::PubSub;
    /// use kiko::id::SessionId;
    /// use std::sync::Arc;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let pubsub = Arc::new(PubSub::new());
    /// let session_id = SessionId::new();
    /// let notifier = pubsub.subscribe(session_id);
    /// notifier.notified().await;
    /// # }
    /// ```
    pub fn subscribe(&self, id: Id) -> Arc<Notify> {
        self.notifiers
            .entry(id)
            .or_insert_with(|| Arc::new(Notify::new()))
            .clone()
    }

    /// Publishes a message to all subscribers of a session.
    ///
    /// If subscribers exist for the session, the message is stored and all
    /// waiting subscribers are notified. If no subscribers exist, the message
    /// is discarded (following typical pub/sub semantics).
    ///
    /// **Note**: Only one message per session is stored at a time. Publishing
    /// a new message will overwrite any existing unprocessed message.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session to publish the message to
    /// * `message` - The message to publish
    ///
    /// # Examples
    ///
    /// ```rust
    /// use kiko_backend::messaging::PubSub;
    /// use kiko::{data::{SessionMessage, CreateSession}, id::SessionId};
    /// use std::{sync::Arc, time::Duration};
    ///
    /// # async fn example() {
    /// let pubsub = Arc::new(PubSub::new());
    /// let session_id = SessionId::new();
    ///
    /// // First subscribe to ensure message isn't discarded
    /// let _notifier = pubsub.subscribe(session_id.clone());
    ///
    /// // Then publish a message
    /// let create_session = CreateSession {
    ///     name: "Test Session".to_string(),
    ///     duration: Duration::from_secs(1800),
    /// };
    /// let message = SessionMessage::CreateSession(create_session);
    /// pubsub.publish(session_id, message);
    /// # }
    /// ```
    pub fn publish(&self, id: Id, message: T) {
        // Get the notifier first
        let notifier = self.notifiers.get(&id).map(|entry| entry.clone());

        if let Some(notifier) = notifier {
            // Store the message - no locks needed with DashMap
            self.events.insert(id, ArcSwap::from_pointee(message));

            // Now notify waiters
            notifier.notify_waiters();
        }
    }

    /// Retrieves the current message for a session without removing it.
    ///
    /// This method performs a lock-free read of the message using [`ArcSwap::load_full`].
    /// The message remains available for subsequent calls to [`get_event`] or [`consume_event`].
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to retrieve the message for
    ///
    /// # Returns
    ///
    /// * `Some(Arc<SessionMessage>)` - If a message exists for the session
    /// * `None` - If no message exists for the session
    ///
    /// # Examples
    ///
    /// ```
    /// use kiko_backend::messaging::PubSub;
    /// use kiko::{data::SessionMessage, id::SessionId};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let pubsub: PubSub<SessionId, SessionMessage> = PubSub::new();
    /// let session_id = SessionId::new();
    /// if let Some(message) = pubsub.get_event(&session_id) {
    ///     // Process message, but it remains in the queue
    /// }
    /// # }
    /// ```
    pub fn get_event(&self, id: &Id) -> Option<Arc<T>> {
        self.events.get(id).map(|entry| entry.load_full())
    }

    /// Consumes and removes the current message for a session.
    ///
    /// This method retrieves the message and removes it from storage in a single
    /// atomic operation. Subsequent calls will return `None` until a new message
    /// is published.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to consume the message for
    ///
    /// # Returns
    ///
    /// * `Some(Arc<SessionMessage>)` - If a message existed and was consumed
    /// * `None` - If no message existed for the session
    ///
    /// # Examples
    ///
    /// ```
    /// use kiko_backend::messaging::PubSub;
    /// use kiko::{data::SessionMessage, id::SessionId};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let pubsub: PubSub<SessionId, SessionMessage> = PubSub::new();
    /// let session_id = SessionId::new();
    /// if let Some(message) = pubsub.consume_event(&session_id) {
    ///     // Process message - it's no longer in the queue
    /// }
    /// # }
    /// ```
    pub fn consume_event(&self, id: &Id) -> Option<Arc<T>> {
        self.events
            .remove(id)
            .map(|(_, arc_swap)| arc_swap.load_full())
    }

    /// Completely removes all data associated with a session.
    ///
    /// This method removes both the notifier and any pending messages for the session,
    /// effectively cleaning up all resources. This should be called when a session
    /// ends to prevent memory leaks.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to clean up
    ///
    /// # Examples
    ///
    /// ```
    /// use kiko_backend::messaging::PubSub;
    /// use kiko::{data::SessionMessage, id::SessionId};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let pubsub: PubSub<SessionId, SessionMessage> = PubSub::new();
    /// let session_id = SessionId::new();
    /// pubsub.cleanup_session(&session_id);
    /// # }
    /// ```
    pub fn cleanup_session(&self, id: &Id) {
        self.events.remove(id);
        self.notifiers.remove(id);
    }

    /// Returns the number of active sessions with subscribers.
    ///
    /// This count represents sessions that have at least one subscriber,
    /// regardless of whether they have pending messages.
    ///
    /// # Returns
    ///
    /// The number of sessions with active subscribers.
    ///
    /// # Examples
    ///
    /// ```
    /// use kiko_backend::messaging::PubSub;
    /// use kiko::{data::SessionMessage, id::SessionId};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let pubsub: PubSub<SessionId, SessionMessage> = PubSub::new();
    /// assert_eq!(pubsub.session_count(), 0);
    /// let session_id = SessionId::new();
    /// let _notifier = pubsub.subscribe(session_id);
    /// assert_eq!(pubsub.session_count(), 1);
    /// # }
    /// ```
    pub fn session_count(&self) -> usize {
        self.notifiers.len()
    }

    /// Checks if a session has a pending message without retrieving it.
    ///
    /// This is a non-destructive way to check for message availability,
    /// useful for conditional processing logic.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to check
    ///
    /// # Returns
    ///
    /// * `true` - If the session has a pending message
    /// * `false` - If the session has no pending message
    ///
    /// # Examples
    ///
    /// ```
    /// use kiko_backend::messaging::PubSub;
    /// use kiko::{data::SessionMessage, id::SessionId};
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let pubsub: PubSub<SessionId, SessionMessage> = PubSub::new();
    /// let session_id = SessionId::new();
    /// if pubsub.has_event(&session_id) {
    ///     let message = pubsub.consume_event(&session_id);
    ///     // Process the message
    /// }
    /// # }
    /// ```
    pub fn has_event(&self, id: &Id) -> bool {
        self.events.contains_key(id)
    }
}

impl<Id, T> Default for PubSub<Id, T>
where
    Id: IdType,
    T: MessageType,
{
    /// Creates a default PubSub instance.
    ///
    /// Equivalent to calling [`PubSub::new()`].
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kiko::data::SessionMessage;
    use kiko::id::SessionId;
    use std::time::Duration;
    use tokio::time::{sleep, timeout};

    #[tokio::test]
    async fn consume_event_removes_message() {
        let pubsub: PubSub<SessionId, SessionMessage> = PubSub::new();
        let session_id = SessionId::new();

        let _notifier = pubsub.subscribe(session_id.clone());

        let message = SessionMessage::CreateSession(kiko::data::CreateSession {
            name: "Test Message".to_string(),
            duration: Duration::from_secs(1800),
        });

        pubsub.publish(session_id.clone(), message);

        // First consume should return the message
        let event = pubsub.consume_event(&session_id);
        assert!(event.is_some());

        // Second consume should return None (message was removed)
        let event2 = pubsub.consume_event(&session_id);
        assert!(event2.is_none());

        // get_event should also return None now
        let event3 = pubsub.get_event(&session_id);
        assert!(event3.is_none());
    }

    #[tokio::test]
    async fn get_event_vs_consume_event() {
        let pubsub: PubSub<SessionId, SessionMessage> = PubSub::new();
        let session_id = SessionId::new();

        let _notifier = pubsub.subscribe(session_id.clone());

        let message = SessionMessage::CreateSession(kiko::data::CreateSession {
            name: "Persistent Test".to_string(),
            duration: Duration::from_secs(1800),
        });

        pubsub.publish(session_id.clone(), message);

        // get_event should not remove the message
        let event1 = pubsub.get_event(&session_id);
        assert!(event1.is_some());

        let event2 = pubsub.get_event(&session_id);
        assert!(event2.is_some());

        // consume_event should remove it
        let event3 = pubsub.consume_event(&session_id);
        assert!(event3.is_some());

        // Now it should be gone
        let event4 = pubsub.get_event(&session_id);
        assert!(event4.is_none());
    }

    #[tokio::test]
    async fn cleanup_session() {
        let pubsub: PubSub<SessionId, SessionMessage> = PubSub::new();
        let session_id = SessionId::new();

        let _notifier = pubsub.subscribe(session_id.clone());

        let message = SessionMessage::CreateSession(kiko::data::CreateSession {
            name: "Cleanup Test".to_string(),
            duration: Duration::from_secs(1800),
        });

        pubsub.publish(session_id.clone(), message);

        // Verify session exists
        assert_eq!(pubsub.session_count(), 1);
        assert!(pubsub.has_event(&session_id));

        // Cleanup session
        pubsub.cleanup_session(&session_id);

        // Verify session is gone
        assert_eq!(pubsub.session_count(), 0);
        assert!(!pubsub.has_event(&session_id));

        let event = pubsub.get_event(&session_id);
        assert!(event.is_none());
    }

    #[tokio::test]
    async fn memory_leak_prevention() {
        let pubsub: PubSub<SessionId, SessionMessage> = PubSub::new();
        let mut session_ids = Vec::new();

        // Create many sessions
        for i in 0..100 {
            let session_id = SessionId::new();
            session_ids.push(session_id.clone());

            let _notifier = pubsub.subscribe(session_id.clone());

            let message = SessionMessage::CreateSession(kiko::data::CreateSession {
                name: format!("Session {i}"),
                duration: Duration::from_secs(1800),
            });
            pubsub.publish(session_id, message);
        }

        assert_eq!(pubsub.session_count(), 100);

        // Consume all events
        for session_id in &session_ids {
            let event = pubsub.consume_event(session_id);
            assert!(event.is_some());
        }

        // Events should be removed, but notifiers should still exist
        assert_eq!(pubsub.session_count(), 100);

        // Clean up all sessions
        for session_id in &session_ids {
            pubsub.cleanup_session(session_id);
        }

        assert_eq!(pubsub.session_count(), 0);
    }

    #[tokio::test]
    async fn has_event_utility() {
        let pubsub: PubSub<SessionId, SessionMessage> = PubSub::new();
        let session_id = SessionId::new();

        let _notifier = pubsub.subscribe(session_id.clone());

        // No event initially
        assert!(!pubsub.has_event(&session_id));

        // Publish message
        let message = SessionMessage::CreateSession(kiko::data::CreateSession {
            name: "Event Check".to_string(),
            duration: Duration::from_secs(1800),
        });
        pubsub.publish(session_id.clone(), message);

        // Should have event now
        assert!(pubsub.has_event(&session_id));

        // Consume event
        let _event = pubsub.consume_event(&session_id);

        // Should not have event anymore
        assert!(!pubsub.has_event(&session_id));
    }

    #[tokio::test]
    async fn correct_pubsub_behavior_subscribe_after_publish() {
        let pubsub: PubSub<SessionId, SessionMessage> = PubSub::new();
        let session_id = SessionId::new();

        // Publish first (no subscribers yet) - this is correct behavior
        let message = SessionMessage::CreateSession(kiko::data::CreateSession {
            name: "Early Message".to_string(),
            duration: Duration::from_secs(1800),
        });

        pubsub.publish(session_id.clone(), message);

        // Then subscribe
        let _notifier = pubsub.subscribe(session_id.clone());

        // Message should not be available since we subscribed after publishing
        // This is CORRECT pub/sub behavior
        let event = pubsub.get_event(&session_id);
        assert!(
            event.is_none(),
            "Should not receive messages published before subscription"
        );
    }

    #[tokio::test]
    async fn event_processing_loop_with_consume() {
        let pubsub = Arc::new(PubSub::new());
        let session_id = SessionId::new();

        let notifier = pubsub.subscribe(session_id.clone());

        let pubsub_clone = pubsub.clone();
        let session_id_clone = session_id.clone();

        // Publisher task
        let publisher = tokio::spawn(async move {
            for i in 0..3 {
                let message = SessionMessage::RemoveParticipant(kiko::data::RemoveParticipant {
                    session_id: session_id_clone.to_string(),
                    participant_id: format!("participant_{i}"),
                });
                pubsub_clone.publish(session_id_clone.clone(), message);
                sleep(Duration::from_millis(50)).await;
            }
        });

        // Consumer loop - this will now work correctly with consume_event
        let mut processed_count = 0;
        let max_messages = 3;

        while processed_count < max_messages {
            // Wait for notification
            let notification_result =
                timeout(Duration::from_millis(200), notifier.notified()).await;
            if notification_result.is_ok() {
                // Process the event
                if let Some(event) = pubsub.consume_event(&session_id) {
                    match event.as_ref() {
                        SessionMessage::RemoveParticipant(remove_participant) => {
                            println!(
                                "Processed participant: {}",
                                remove_participant.participant_id
                            );
                            processed_count += 1;
                        }
                        _ => panic!("Unexpected message type"),
                    }
                }
            } else {
                break; // Timeout
            }
        }

        publisher.await.unwrap();
        assert_eq!(processed_count, 3);

        // All events should be consumed
        assert!(!pubsub.has_event(&session_id));
    }

    // Keep your original working tests too
    #[tokio::test]
    async fn multiple_subscribers() {
        let pubsub: PubSub<SessionId, SessionMessage> = PubSub::new();
        let session_id = SessionId::new();
        let _notifier1 = pubsub.subscribe(session_id.clone());
        let _notifier2 = pubsub.subscribe(session_id.clone());

        let message = SessionMessage::AddParticipant(kiko::data::AddParticipant {
            session_id: session_id.clone().to_string(),
            participant_name: "participant1".to_string(),
        });

        pubsub.publish(session_id.clone(), message.clone());
        let event1 = pubsub.get_event(&session_id);
        let event2 = pubsub.get_event(&session_id);

        assert!(event1.is_some());
        assert!(event2.is_some());

        match event1.unwrap().as_ref() {
            SessionMessage::AddParticipant(add_participant) => {
                assert_eq!(add_participant.participant_name, "participant1");
            }
            _ => panic!("Unexpected message type"),
        }

        match event2.unwrap().as_ref() {
            SessionMessage::AddParticipant(add_participant) => {
                assert_eq!(add_participant.participant_name, "participant1");
            }
            _ => panic!("Unexpected message type"),
        }
    }
}
