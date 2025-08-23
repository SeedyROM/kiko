//! WebSocket hook for Yew components.
//!
//! This module provides a hook for managing WebSocket connections in Yew applications.
//! It handles connection state, message sending/receiving, and automatic cleanup.

use std::cell::RefCell;
use std::rc::Rc;

use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{Message, futures::WebSocket};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use kiko::async_callback;

/// Represents the current state of a WebSocket connection.
#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionState {
    /// WebSocket is not connected
    Disconnected,
    /// WebSocket is in the process of connecting
    Connecting,
    /// WebSocket is connected and ready to send/receive messages
    Connected,
    /// WebSocket connection failed or encountered an error
    Error(String),
}

/// Callback type for handling incoming WebSocket messages.
pub type MessageCallback = Callback<String>;

/// Internal type for managing the WebSocket sender.
type WebSocketSender = Rc<RefCell<Option<SplitSink<WebSocket, Message>>>>;

/// Internal type for managing the abort handle for the read task
type AbortHandle = Rc<RefCell<Option<futures::future::AbortHandle>>>;

/// Handle returned by the `use_websocket` hook, providing control over the WebSocket connection.
pub struct WebSocketHandle {
    /// Current connection state
    pub state: ConnectionState,
    /// Callback to initiate a WebSocket connection
    pub connect: Callback<()>,
    /// Callback to close the WebSocket connection
    pub disconnect: Callback<()>,
    /// Callback to send a text message through the WebSocket
    pub send: Callback<String>,
    /// Callback to set the message handler for incoming messages
    pub set_on_message: Callback<MessageCallback>,
}

/// A Yew hook for managing WebSocket connections.
///
/// This hook provides a complete WebSocket client implementation with automatic
/// connection management, message handling, and state tracking.
///
/// # Arguments
///
/// * `url` - The WebSocket URL to connect to (e.g., "ws://localhost:3030/ws")
///
/// # Returns
///
/// Returns a `WebSocketHandle` with methods to control the connection:
/// - `state`: Current connection state
/// - `connect()`: Initiate connection
/// - `disconnect()`: Close connection
/// - `send(message)`: Send text message
/// - `set_on_message(callback)`: Set message handler
///
/// # Example
///
/// ```rust
/// use yew::prelude::*;
/// use crate::hooks::use_websocket;
///
/// #[function_component(WebSocketExample)]
/// fn websocket_example() -> Html {
///     let ws = use_websocket("ws://localhost:3030/ws");
///     let messages = use_state(Vec::<String>::new);
///
///     // Set up message handler
///     {
///         let messages = messages.clone();
///         use_effect_with((), move |_| {
///             let messages = messages.clone();
///             ws.set_on_message.emit(Callback::from(move |msg: String| {
///                 let mut msgs = (*messages).clone();
///                 msgs.push(msg);
///                 messages.set(msgs);
///             }));
///         });
///     }
///
///     let on_connect = {
///         let ws = ws.clone();
///         Callback::from(move |_| ws.connect.emit(()))
///     };
///
///     let on_send = {
///         let ws = ws.clone();
///         Callback::from(move |_| ws.send.emit("Hello WebSocket!".to_string()))
///     };
///
///     html! {
///         <div>
///             <p>{ format!("Status: {:?}", ws.state) }</p>
///             <button onclick={on_connect}>{ "Connect" }</button>
///             <button onclick={on_send}>{ "Send Message" }</button>
///             <ul>
///                 { for messages.iter().map(|msg| html! { <li>{ msg }</li> }) }
///             </ul>
///         </div>
///     }
/// }
/// ```
///
/// # Connection Lifecycle
///
/// 1. **Disconnected**: Initial state, no connection active
/// 2. **Connecting**: Connection attempt in progress
/// 3. **Connected**: WebSocket ready for sending/receiving messages
/// 4. **Error**: Connection failed or encountered an error
///
/// The hook automatically handles connection cleanup when the component unmounts.
#[hook]
pub fn use_websocket(url: &str) -> WebSocketHandle {
    use futures::FutureExt;

    let state = use_state(|| ConnectionState::Disconnected);
    let sender: WebSocketSender = use_mut_ref(|| None);
    let on_message_callback: Rc<RefCell<Option<MessageCallback>>> = use_mut_ref(|| None);
    let abort_handle: AbortHandle = use_mut_ref(|| None);

    let url = url.to_string();

    // Clean up on unmount
    {
        let abort_handle = abort_handle.clone();
        let sender = sender.clone();
        use_effect_with((), move |_| {
            move || {
                // Abort the read task if it exists
                if let Some(handle) = abort_handle.borrow_mut().take() {
                    handle.abort();
                }
                // Close the sender if it exists
                if let Some(mut write) = sender.borrow_mut().take() {
                    spawn_local(async move {
                        let _ = write.close().await;
                    });
                }
            }
        });
    }

    let connect = async_callback!([state, sender, on_message_callback, abort_handle, url] {
        // Don't connect if already connected or connecting
        if matches!(*state, ConnectionState::Connected | ConnectionState::Connecting) {
            return;
        }

        state.set(ConnectionState::Connecting);

        // Abort any existing read task
        if let Some(handle) = abort_handle.borrow_mut().take() {
            handle.abort();
        }

        match WebSocket::open(&url) {
            Ok(ws) => {
                // Keep in Connecting state until we confirm the connection works

                let (write, read) = ws.split();
                *sender.borrow_mut() = Some(write);

                // Create an abortable future for the read task
                let (abort_handle_new, abort_registration) = futures::future::AbortHandle::new_pair();
                *abort_handle.borrow_mut() = Some(abort_handle_new);

                // Handle incoming messages with ping health checks
                let read_future = async move {
                    let mut read = read;
                    // Start with connection as Connected since WebSocket.open() succeeded
                    // We'll detect failures through the read loop ending or send failures
                    state.set(ConnectionState::Connected);

                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                // All text messages go to the callback
                                if let Some(callback) = on_message_callback.borrow().as_ref() {
                                    callback.emit(text);
                                }
                            }
                            Ok(Message::Bytes(_)) => {
                                // Handle binary messages if needed
                            }
                            Err(e) => {
                                state.set(ConnectionState::Error(format!("WebSocket error: {e:?}")));
                                break;
                            }
                        }

                        // Connection health is now monitored through the read stream
                        // If the read stream closes, we'll detect it and update state accordingly
                    }

                    // Clean up when the read loop ends
                    *sender.borrow_mut() = None;
                    *abort_handle.borrow_mut() = None;
                    state.set(ConnectionState::Disconnected);
                };

                // Spawn the abortable future
                spawn_local(futures::future::Abortable::new(read_future, abort_registration).map(|_| ()));
            }
            Err(e) => {
                state.set(ConnectionState::Error(format!("Failed to connect: {e:?}")));
            }
        }
    });

    let disconnect = {
        let state = state.clone();
        let sender = sender.clone();
        let abort_handle = abort_handle.clone();

        Callback::from(move |_: ()| {
            // First, abort the read task to stop receiving messages
            if let Some(handle) = abort_handle.borrow_mut().take() {
                handle.abort();
            }

            // Then close the write half
            if let Some(mut write) = sender.borrow_mut().take() {
                let state = state.clone();
                spawn_local(async move {
                    let _ = write.close().await;
                    state.set(ConnectionState::Disconnected);
                });
            } else {
                // Already disconnected
                state.set(ConnectionState::Disconnected);
            }
        })
    };

    let send = {
        let sender = sender.clone();
        let state = state.clone();

        Callback::from(move |text: String| {
            // Only send if connected
            if !matches!(*state, ConnectionState::Connected) {
                use kiko::log::warn;
                warn!("Cannot send message: WebSocket is not connected");
                return;
            }

            let sender = sender.clone();
            if let Some(mut write) = sender.borrow_mut().take() {
                let sender_clone = sender.clone();
                spawn_local(async move {
                    if (write.send(Message::Text(text)).await).is_err() {
                        // Handle send error - connection might be closed
                        use kiko::log::error;
                        error!("Failed to send message through WebSocket");
                    }
                    *sender_clone.borrow_mut() = Some(write);
                });
            }
        })
    };

    let set_on_message = {
        let on_message_callback = on_message_callback.clone();

        Callback::from(move |callback: MessageCallback| {
            *on_message_callback.borrow_mut() = Some(callback);
        })
    };

    WebSocketHandle {
        state: (*state).clone(),
        connect,
        disconnect,
        send,
        set_on_message,
    }
}
