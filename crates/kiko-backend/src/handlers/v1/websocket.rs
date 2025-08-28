use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{
        ConnectInfo, State,
        ws::{self, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use tokio::sync::mpsc;

use kiko::{data::SessionMessage, errors::WebSocketError};
use kiko::{id::SessionId, tracing};
use kiko::{log, serde_json};

use crate::services::SessionService;

#[derive(Debug)]
pub enum WebSocketResponse {
    Success(String),
    SubscriptionStarted,
    None,
}

pub struct ConnectionState {
    session_id: Option<SessionId>,
    participant_id: Option<kiko::id::ParticipantId>,
    task_handle: Option<tokio::task::JoinHandle<()>>,
    outbound_rx: Option<mpsc::UnboundedReceiver<String>>,
}

impl ConnectionState {
    pub fn new() -> Self {
        Self {
            session_id: None,
            participant_id: None,
            task_handle: None,
            outbound_rx: None,
        }
    }

    pub fn is_subscribed(&self) -> bool {
        self.task_handle.is_some()
    }

    pub fn cleanup_subscription(&mut self) {
        if let Some(handle) = &self.task_handle {
            if !handle.is_finished() {
                handle.abort();
            }
        }
        self.task_handle = None;
        self.outbound_rx = None;
        self.participant_id = None;
    }
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::new()
    }
}

async fn handle_create_session(
    _create: &kiko::data::CreateSession,
    _state: &Arc<crate::AppState>,
) -> Result<WebSocketResponse, WebSocketError> {
    log::info!("Creating session: {:?}", _create);
    // TODO: Implement session creation logic
    Ok(WebSocketResponse::None)
}

async fn setup_subscription(
    session_id: SessionId,
    state: &Arc<crate::AppState>,
    conn_state: &mut ConnectionState,
) -> Result<(), WebSocketError> {
    if conn_state.is_subscribed() {
        return Err(WebSocketError::AlreadySubscribed);
    }

    // Check if the session exists
    if state.sessions.get(&session_id).await.is_err() {
        return Err(WebSocketError::SessionNotFound(session_id.to_string()));
    }

    conn_state.session_id = Some(session_id.clone());

    // Subscribe to the session and get the notifier
    let notifier = state.pub_sub.subscribe(session_id.clone()).await;

    // Create a channel for sending messages to the WebSocket
    let (outbound_tx, rx) = mpsc::unbounded_channel::<String>();
    conn_state.outbound_rx = Some(rx);

    // Spawn a task to listen for messages and notify the WebSocket
    let task_handle = tokio::spawn({
        let state = state.clone();
        let session_id = session_id.clone();
        async move {
            log::debug!(
                "Starting PubSub listener task for session: {:?}",
                session_id
            );
            let mut notified = notifier.notified(); // Create first notification listener
            loop {
                notified.await; // Wait for notification
                log::debug!("Received notification for session: {:?}", session_id);
                // Create the next notification listener BEFORE processing the current message
                notified = notifier.notified();

                if let Some(msg) = state.pub_sub.get_event(&session_id).await {
                    match serde_json::to_string(&*msg) {
                        Ok(json) => {
                            log::debug!("Sending message to WebSocket: {}", json);
                            if outbound_tx.send(json).is_err() {
                                log::debug!(
                                    "WebSocket channel closed, ending PubSub task for session: {:?}",
                                    session_id
                                );
                                break; // Channel closed
                            }
                        }
                        Err(e) => {
                            log::error!(
                                "Failed to serialize message for session {:?}: {}",
                                session_id,
                                e
                            );
                        }
                    }
                } else {
                    log::debug!(
                        "No message found after notification for session: {:?}",
                        session_id
                    );
                }
            }
            log::debug!("PubSub listener task ended for session: {:?}", session_id);
        }
    });

    conn_state.task_handle = Some(task_handle);
    Ok(())
}

async fn handle_join_session(
    join: &kiko::data::JoinSession,
    state: &Arc<crate::AppState>,
    conn_state: &mut ConnectionState,
) -> Result<WebSocketResponse, WebSocketError> {
    log::info!("Joining session: {:?}", join);

    let session_id: SessionId = join.session_id.clone().into();

    // Check if the session exists
    let mut session = state
        .sessions
        .get(&session_id)
        .await
        .map_err(|_| WebSocketError::SessionNotFound(join.session_id.clone()))?;

    // Add participant to the session
    let participant_id = kiko::id::ParticipantId::new();
    let participant =
        kiko::data::Participant::new(participant_id.clone(), join.participant_name.clone());
    session.add_participant(participant);

    // Store the participant ID in the connection state for cleanup
    conn_state.participant_id = Some(participant_id);

    // Update the session in storage
    state
        .sessions
        .update(&session_id, &session)
        .await
        .map_err(|_| WebSocketError::InvalidMessage("Failed to update session".to_string()))?;

    // Broadcast the updated session to all subscribers
    let update_message = SessionMessage::SessionUpdate(session);
    state.pub_sub.publish(session_id, update_message).await;

    Ok(WebSocketResponse::None)
}

async fn handle_subscribe_to_session(
    subscribe: &kiko::data::SubscribeToSession,
    state: &Arc<crate::AppState>,
    conn_state: &mut ConnectionState,
) -> Result<WebSocketResponse, WebSocketError> {
    log::info!("Subscribing to session: {:?}", subscribe);

    let session_id: SessionId = subscribe.session_id.clone().into();

    // Just setup subscription without joining
    setup_subscription(session_id, state, conn_state).await?;

    Ok(WebSocketResponse::SubscriptionStarted)
}

async fn handle_add_participant(
    add: &kiko::data::AddParticipant,
    _state: &Arc<crate::AppState>,
) -> Result<WebSocketResponse, WebSocketError> {
    log::info!("Adding participant: {:?}", add);
    // TODO: Implement participant addition logic
    Ok(WebSocketResponse::None)
}

async fn handle_remove_participant(
    remove: &kiko::data::RemoveParticipant,
    state: &Arc<crate::AppState>,
) -> Result<WebSocketResponse, WebSocketError> {
    log::info!("Removing participant: {:?}", remove);

    let session_id: SessionId = remove.session_id.clone().into();
    let participant_id: kiko::id::ParticipantId = remove.participant_id.clone().into();

    // Get the current session
    let mut session = state
        .sessions
        .get(&session_id)
        .await
        .map_err(|_| WebSocketError::SessionNotFound(remove.session_id.clone()))?;

    // Remove participant from the session
    session.remove_participant(&participant_id);

    // Update the session in storage
    state
        .sessions
        .update(&session_id, &session)
        .await
        .map_err(|_| WebSocketError::InvalidMessage("Failed to update session".to_string()))?;

    // Broadcast the updated session to all subscribers
    let update_message = SessionMessage::SessionUpdate(session);
    state.pub_sub.publish(session_id, update_message).await;

    Ok(WebSocketResponse::None)
}

async fn handle_session_update(
    update: &kiko::data::Session,
    _state: &Arc<crate::AppState>,
) -> Result<WebSocketResponse, WebSocketError> {
    log::info!("Session update: {:?}", update);
    // TODO: Implement session update logic
    Ok(WebSocketResponse::None)
}

async fn handle_point_session(
    point: &kiko::data::PointSession,
    state: &Arc<crate::AppState>,
    _conn_state: &mut ConnectionState,
) -> Result<WebSocketResponse, WebSocketError> {
    log::info!("Pointing session: {:?}", point);

    let session_id: SessionId = point.session_id.clone().into();
    let participant_id: kiko::id::ParticipantId = point.participant_id.clone().into();

    // Get the current session
    let mut session = state
        .sessions
        .get(&session_id)
        .await
        .map_err(|_| WebSocketError::SessionNotFound(point.session_id.clone()))?;

    // Add points for the participant
    session.point(&participant_id, point.points);

    // Update the session in storage
    state
        .sessions
        .update(&session_id, &session)
        .await
        .map_err(|_| WebSocketError::InvalidMessage("Failed to update session".to_string()))?;

    // Broadcast the updated session to all subscribers
    let update_message = SessionMessage::SessionUpdate(session);
    state.pub_sub.publish(session_id, update_message).await;

    Ok(WebSocketResponse::None)
}

async fn handle_set_topic(
    topic: &String,
    state: &Arc<crate::AppState>,
    conn_state: &mut ConnectionState,
) -> Result<WebSocketResponse, WebSocketError> {
    log::info!("Setting topic: {:?}", topic);

    let session_id = match &conn_state.session_id {
        Some(id) => id.clone(),
        None => return Err(WebSocketError::NotSubscribed),
    };

    // Get the current session
    let mut session = state
        .sessions
        .get(&session_id)
        .await
        .map_err(|_| WebSocketError::SessionNotFound(session_id.to_string()))?;

    // Set the new topic
    session.set_topic(topic.clone());

    // Update the session in storage
    state
        .sessions
        .update(&session_id, &session)
        .await
        .map_err(|_| WebSocketError::InvalidMessage("Failed to update session".to_string()))?;

    // Broadcast the updated session to all subscribers
    let update_message = SessionMessage::SessionUpdate(session);
    state.pub_sub.publish(session_id, update_message).await;

    Ok(WebSocketResponse::None)
}

async fn handle_clear_points(
    state: &Arc<crate::AppState>,
    conn_state: &mut ConnectionState,
) -> Result<WebSocketResponse, WebSocketError> {
    log::info!("Clearing points");

    let session_id = match &conn_state.session_id {
        Some(id) => id.clone(),
        None => return Err(WebSocketError::NotSubscribed),
    };

    // Get the current session
    let mut session = state
        .sessions
        .get(&session_id)
        .await
        .map_err(|_| WebSocketError::SessionNotFound(session_id.to_string()))?;

    // Clear all points
    session.clear_points();

    // Update the session in storage
    state
        .sessions
        .update(&session_id, &session)
        .await
        .map_err(|_| WebSocketError::InvalidMessage("Failed to update session".to_string()))?;

    // Broadcast the updated session to all subscribers
    let update_message = SessionMessage::SessionUpdate(session);
    state.pub_sub.publish(session_id, update_message).await;

    Ok(WebSocketResponse::None)
}

async fn handle_toggle_hide_points(
    state: &Arc<crate::AppState>,
    conn_state: &mut ConnectionState,
) -> Result<WebSocketResponse, WebSocketError> {
    log::info!("Toggling hide points");

    let session_id = match &conn_state.session_id {
        Some(id) => id.clone(),
        None => return Err(WebSocketError::NotSubscribed),
    };

    // Get the current session
    let mut session = state
        .sessions
        .get(&session_id)
        .await
        .map_err(|_| WebSocketError::SessionNotFound(session_id.to_string()))?;

    session.toggle_hide_points();

    // Update the session in storage
    state
        .sessions
        .update(&session_id, &session)
        .await
        .map_err(|_| WebSocketError::InvalidMessage("Failed to update session".to_string()))?;

    // Broadcast the updated session to all subscribers
    let update_message = SessionMessage::SessionUpdate(session);
    state.pub_sub.publish(session_id, update_message).await;

    Ok(WebSocketResponse::None)
}

async fn send_error(socket: &mut WebSocket, error: &WebSocketError) -> bool {
    let error_msg = error.to_string();
    log::error!("{}", error_msg);

    if let Err(e) = socket.send(ws::Message::Text(error_msg.into())).await {
        log::error!("Failed to send error message: {}", e);
        return false; // Connection broken
    }
    true
}

async fn send_response(socket: &mut WebSocket, response: WebSocketResponse) -> bool {
    match response {
        WebSocketResponse::Success(msg) => {
            if let Err(e) = socket.send(ws::Message::Text(msg.into())).await {
                log::error!("Failed to send success message: {}", e);
                return false;
            }
        }
        WebSocketResponse::SubscriptionStarted => {
            log::debug!("Subscription started successfully");
            // No message sent to client for this response type
        }
        WebSocketResponse::None => {
            // No response needed
        }
    }
    true
}

/// Handler to upgrade HTTP connection to WebSocket
pub async fn upgrade(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<crate::AppState>>,
) -> Response {
    // Pass the client address to the handler
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

/// Handle WebSocket connection
#[tracing::instrument(name = "websocket", skip(socket, state))]
async fn handle_socket(
    mut socket: WebSocket,
    client_addr: SocketAddr,
    state: Arc<crate::AppState>,
) {
    log::debug!("Connection established");
    let mut conn_state = ConnectionState::new();

    loop {
        tokio::select! {
            // Handle incoming WebSocket messages
            msg = socket.recv() => {
                if !handle_incoming_message(msg, &mut socket, &mut conn_state, &state, client_addr).await {
                    break;
                }
            }

            // Handle outbound messages from the subscription task
            outbound_msg = async {
                match &mut conn_state.outbound_rx {
                    Some(rx) => rx.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                if !handle_outbound_message(outbound_msg, &mut socket).await {
                    break;
                }
            }
        }
    }

    log::debug!("Cleaning up WebSocket connection");
    cleanup_connection(&mut conn_state, &state).await;
    log::debug!("Connection ended");
}

async fn handle_incoming_message(
    msg: Option<Result<ws::Message, axum::Error>>,
    socket: &mut WebSocket,
    conn_state: &mut ConnectionState,
    state: &Arc<crate::AppState>,
    client_addr: SocketAddr,
) -> bool {
    match msg {
        Some(Ok(ws::Message::Text(text))) => {
            handle_text_message(text.to_string(), socket, conn_state, state, client_addr).await
        }
        Some(Ok(ws::Message::Close(_))) => {
            log::debug!("Connection closed by client");
            false
        }
        Some(Err(e)) => {
            log::error!("WebSocket error: {}", e);
            false
        }
        None => {
            log::debug!("WebSocket connection closed");
            false
        }
        _ => {
            // Handle other message types (Binary, Ping, Pong) if needed
            true
        }
    }
}

async fn handle_text_message(
    text: String,
    socket: &mut WebSocket,
    conn_state: &mut ConnectionState,
    state: &Arc<crate::AppState>,
    client_addr: SocketAddr,
) -> bool {
    let session_msg = match serde_json::from_str::<SessionMessage>(&text) {
        Ok(msg) => msg,
        Err(e) => {
            let error = WebSocketError::InvalidMessage(e.to_string());
            return send_error(socket, &error).await;
        }
    };

    log::debug!("Received message from {}: {:?}", client_addr, session_msg);

    let result = match &session_msg {
        // Session management
        SessionMessage::CreateSession(create) => handle_create_session(create, state).await,
        SessionMessage::JoinSession(join) => handle_join_session(join, state, conn_state).await,
        SessionMessage::SubscribeToSession(subscribe) => {
            handle_subscribe_to_session(subscribe, state, conn_state).await
        }

        // Participant management
        SessionMessage::AddParticipant(add) => handle_add_participant(add, state).await,
        SessionMessage::RemoveParticipant(remove) => handle_remove_participant(remove, state).await,

        // Session actions
        SessionMessage::SetTopic(topic) => handle_set_topic(topic, state, conn_state).await,
        SessionMessage::PointSession(point) => handle_point_session(point, state, conn_state).await,
        SessionMessage::ClearPoints => handle_clear_points(state, conn_state).await,
        SessionMessage::ToggleHidePoints => handle_toggle_hide_points(state, conn_state).await,

        // Session updates (usually from server-side, but handled here for completeness)
        SessionMessage::SessionUpdate(update) => handle_session_update(update, state).await,
    };

    match result {
        Ok(response) => send_response(socket, response).await,
        Err(error) => send_error(socket, &error).await,
    }
}

async fn handle_outbound_message(outbound_msg: Option<String>, socket: &mut WebSocket) -> bool {
    match outbound_msg {
        Some(json) => {
            if let Err(e) = socket.send(ws::Message::Text(json.into())).await {
                log::error!("Failed to send message to WebSocket: {}", e);
                false
            } else {
                true
            }
        }
        None => {
            log::debug!("Outbound channel closed");
            false
        }
    }
}

async fn cleanup_connection(conn_state: &mut ConnectionState, state: &Arc<crate::AppState>) {
    // Remove participant from session if they were joined
    if let (Some(session_id), Some(participant_id)) =
        (&conn_state.session_id, &conn_state.participant_id)
    {
        log::info!(
            "Cleaning up participant {:?} from session {:?}",
            participant_id,
            session_id
        );

        // Remove participant from session
        if let Ok(mut session) = state.sessions.get(session_id).await {
            session.remove_participant(participant_id);

            if (state.sessions.update(session_id, &session).await).is_ok() {
                // Broadcast the updated session to all remaining subscribers
                let update_message = SessionMessage::SessionUpdate(session);
                state
                    .pub_sub
                    .publish(session_id.clone(), update_message)
                    .await;
            }
        }
    }

    conn_state.cleanup_subscription();

    // Note: We don't call pub_sub.cleanup_session here because other subscribers
    // might still be connected to this session. The PubSub system will handle
    // cleanup when there are no more subscribers.
}
