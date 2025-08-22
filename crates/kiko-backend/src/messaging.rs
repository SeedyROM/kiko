use std::{collections::HashMap, sync::Arc};

use arc_swap::ArcSwap;
use kiko::{data::SessionMessage, id::SessionId};
use tokio::sync::{Notify, RwLock};

pub type EventId = u64;

pub struct PubSub {
    notifiers: RwLock<HashMap<SessionId, Arc<Notify>>>,
    events: RwLock<HashMap<SessionId, ArcSwap<SessionMessage>>>,
}

impl PubSub {
    pub fn new() -> Self {
        Self {
            notifiers: RwLock::new(HashMap::new()),
            events: RwLock::new(HashMap::new()),
        }
    }

    pub async fn subscribe(&self, session_id: SessionId) -> Arc<Notify> {
        let mut notifiers = self.notifiers.write().await;
        notifiers
            .entry(session_id)
            .or_insert_with(|| Arc::new(Notify::new()))
            .clone()
    }

    pub async fn publish(&self, session_id: SessionId, message: SessionMessage) {
        let mut events = self.events.write().await;
        let notifier = self.notifiers.read().await.get(&session_id).cloned();

        if let Some(notifier) = notifier {
            events.insert(session_id, ArcSwap::from_pointee(message));
            notifier.notify_waiters();
        }
    }

    pub async fn get_event(&self, session_id: &SessionId) -> Option<Arc<SessionMessage>> {
        let events = self.events.read().await;
        events.get(session_id).map(|arc_swap| arc_swap.load_full())
    }

    pub async fn consume_event(&self, session_id: &SessionId) -> Option<Arc<SessionMessage>> {
        let mut events = self.events.write().await;
        events
            .remove(session_id)
            .map(|arc_swap| arc_swap.load_full())
    }

    pub async fn cleanup_session(&self, session_id: &SessionId) {
        let mut events = self.events.write().await;
        let mut notifiers = self.notifiers.write().await;
        events.remove(session_id);
        notifiers.remove(session_id);
    }

    pub async fn session_count(&self) -> usize {
        self.notifiers.read().await.len()
    }

    pub async fn has_event(&self, session_id: &SessionId) -> bool {
        let events = self.events.read().await;
        events.contains_key(session_id)
    }
}

impl Default for PubSub {
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
        let pubsub = PubSub::new();
        let session_id = SessionId::new();

        let _notifier = pubsub.subscribe(session_id.clone()).await;

        let message = SessionMessage::CreateSession(kiko::data::CreateSession {
            name: "Test Message".to_string(),
            duration: Duration::from_secs(1800),
        });

        pubsub.publish(session_id.clone(), message).await;

        // First consume should return the message
        let event = pubsub.consume_event(&session_id).await;
        assert!(event.is_some());

        // Second consume should return None (message was removed)
        let event2 = pubsub.consume_event(&session_id).await;
        assert!(event2.is_none());

        // get_event should also return None now
        let event3 = pubsub.get_event(&session_id).await;
        assert!(event3.is_none());
    }

    #[tokio::test]
    async fn get_event_vs_consume_event() {
        let pubsub = PubSub::new();
        let session_id = SessionId::new();

        let _notifier = pubsub.subscribe(session_id.clone()).await;

        let message = SessionMessage::CreateSession(kiko::data::CreateSession {
            name: "Persistent Test".to_string(),
            duration: Duration::from_secs(1800),
        });

        pubsub.publish(session_id.clone(), message).await;

        // get_event should not remove the message
        let event1 = pubsub.get_event(&session_id).await;
        assert!(event1.is_some());

        let event2 = pubsub.get_event(&session_id).await;
        assert!(event2.is_some());

        // consume_event should remove it
        let event3 = pubsub.consume_event(&session_id).await;
        assert!(event3.is_some());

        // Now it should be gone
        let event4 = pubsub.get_event(&session_id).await;
        assert!(event4.is_none());
    }

    #[tokio::test]
    async fn cleanup_session() {
        let pubsub = PubSub::new();
        let session_id = SessionId::new();

        let _notifier = pubsub.subscribe(session_id.clone()).await;

        let message = SessionMessage::CreateSession(kiko::data::CreateSession {
            name: "Cleanup Test".to_string(),
            duration: Duration::from_secs(1800),
        });

        pubsub.publish(session_id.clone(), message).await;

        // Verify session exists
        assert_eq!(pubsub.session_count().await, 1);
        assert!(pubsub.has_event(&session_id).await);

        // Cleanup session
        pubsub.cleanup_session(&session_id).await;

        // Verify session is gone
        assert_eq!(pubsub.session_count().await, 0);
        assert!(!pubsub.has_event(&session_id).await);

        let event = pubsub.get_event(&session_id).await;
        assert!(event.is_none());
    }

    #[tokio::test]
    async fn memory_leak_prevention() {
        let pubsub = PubSub::new();
        let mut session_ids = Vec::new();

        // Create many sessions
        for i in 0..100 {
            let session_id = SessionId::new();
            session_ids.push(session_id.clone());

            let _notifier = pubsub.subscribe(session_id.clone()).await;

            let message = SessionMessage::CreateSession(kiko::data::CreateSession {
                name: format!("Session {i}"),
                duration: Duration::from_secs(1800),
            });
            pubsub.publish(session_id, message).await;
        }

        assert_eq!(pubsub.session_count().await, 100);

        // Consume all events
        for session_id in &session_ids {
            let event = pubsub.consume_event(session_id).await;
            assert!(event.is_some());
        }

        // Events should be removed, but notifiers should still exist
        assert_eq!(pubsub.session_count().await, 100);

        // Clean up all sessions
        for session_id in &session_ids {
            pubsub.cleanup_session(session_id).await;
        }

        assert_eq!(pubsub.session_count().await, 0);
    }

    #[tokio::test]
    async fn has_event_utility() {
        let pubsub = PubSub::new();
        let session_id = SessionId::new();

        let _notifier = pubsub.subscribe(session_id.clone()).await;

        // No event initially
        assert!(!pubsub.has_event(&session_id).await);

        // Publish message
        let message = SessionMessage::CreateSession(kiko::data::CreateSession {
            name: "Event Check".to_string(),
            duration: Duration::from_secs(1800),
        });
        pubsub.publish(session_id.clone(), message).await;

        // Should have event now
        assert!(pubsub.has_event(&session_id).await);

        // Consume event
        let _event = pubsub.consume_event(&session_id).await;

        // Should not have event anymore
        assert!(!pubsub.has_event(&session_id).await);
    }

    #[tokio::test]
    async fn correct_pubsub_behavior_subscribe_after_publish() {
        let pubsub = PubSub::new();
        let session_id = SessionId::new();

        // Publish first (no subscribers yet) - this is correct behavior
        let message = SessionMessage::CreateSession(kiko::data::CreateSession {
            name: "Early Message".to_string(),
            duration: Duration::from_secs(1800),
        });

        pubsub.publish(session_id.clone(), message).await;

        // Then subscribe
        let _notifier = pubsub.subscribe(session_id.clone()).await;

        // Message should not be available since we subscribed after publishing
        // This is CORRECT pub/sub behavior
        let event = pubsub.get_event(&session_id).await;
        assert!(
            event.is_none(),
            "Should not receive messages published before subscription"
        );
    }

    #[tokio::test]
    async fn event_processing_loop_with_consume() {
        let pubsub = Arc::new(PubSub::new());
        let session_id = SessionId::new();

        let notifier = pubsub.subscribe(session_id.clone()).await;

        let pubsub_clone = pubsub.clone();
        let session_id_clone = session_id.clone();

        // Publisher task
        let publisher = tokio::spawn(async move {
            for i in 0..3 {
                let message = SessionMessage::RemoveParticipant(kiko::data::RemoveParticipant {
                    session_id: session_id_clone.to_string(),
                    participant_id: format!("participant_{i}"),
                });
                pubsub_clone
                    .publish(session_id_clone.clone(), message)
                    .await;
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
                if let Some(event) = pubsub.consume_event(&session_id).await {
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
        assert!(!pubsub.has_event(&session_id).await);
    }

    // Keep your original working tests too
    #[tokio::test]
    async fn multiple_subscribers() {
        let pubsub = PubSub::new();
        let session_id = SessionId::new();
        let _notifier1 = pubsub.subscribe(session_id.clone()).await;
        let _notifier2 = pubsub.subscribe(session_id.clone()).await;

        let message = SessionMessage::AddParticipant(kiko::data::AddParticipant {
            session_id: session_id.clone().to_string(),
            participant_name: "participant1".to_string(),
        });

        pubsub.publish(session_id.clone(), message.clone()).await;
        let event1 = pubsub.get_event(&session_id).await;
        let event2 = pubsub.get_event(&session_id).await;

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
