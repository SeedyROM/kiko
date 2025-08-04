use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{Message, futures::WebSocket};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

#[derive(Clone, Debug)]
pub struct WebSocketMessage {
    pub content: String,
    pub timestamp: String,
    pub is_outgoing: bool,
}

#[derive(Clone)]
pub struct WebSocketState {
    pub connection_state: ConnectionState,
    pub messages: VecDeque<WebSocketMessage>,
}

impl Default for WebSocketState {
    fn default() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            messages: VecDeque::new(),
        }
    }
}

pub enum WebSocketAction {
    SetConnectionState(ConnectionState),
    AddMessage(WebSocketMessage),
    ClearMessages,
}

impl Reducible for WebSocketState {
    type Action = WebSocketAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut state = (*self).clone();
        match action {
            WebSocketAction::SetConnectionState(new_state) => {
                state.connection_state = new_state;
            }
            WebSocketAction::AddMessage(message) => {
                state.messages.push_back(message);
                if state.messages.len() > 100 {
                    state.messages.pop_front();
                }
            }
            WebSocketAction::ClearMessages => {
                state.messages.clear();
            }
        }
        Rc::new(state)
    }
}

type UseWebSocketReturn = (
    UseReducerHandle<WebSocketState>,
    Callback<web_sys::MouseEvent>,
    Callback<web_sys::MouseEvent>,
    Callback<String>,
);

#[hook]
pub fn use_websocket() -> UseWebSocketReturn {
    let ws_state = use_reducer(WebSocketState::default);
    let sender: Rc<RefCell<Option<futures::stream::SplitSink<WebSocket, Message>>>> =
        use_mut_ref(|| None);

    let connect = {
        let ws_state = ws_state.clone();
        let sender = sender.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            let ws_state = ws_state.clone();
            let sender = sender.clone();

            spawn_local(async move {
                ws_state.dispatch(WebSocketAction::SetConnectionState(ConnectionState::Connecting));

                match WebSocket::open("ws://127.0.0.1:3030/api/v1/ws") {
                    Ok(ws) => {
                        ws_state.dispatch(WebSocketAction::SetConnectionState(ConnectionState::Connected));

                        let (write, mut read) = ws.split();
                        *sender.borrow_mut() = Some(write);

                        // Handle incoming messages
                        spawn_local(async move {
                            while let Some(msg) = read.next().await {
                                match msg {
                                    Ok(Message::Text(text)) => {
                                        let ws_msg = WebSocketMessage {
                                            content: text,
                                            timestamp: js_sys::Date::new_0().to_iso_string().into(),
                                            is_outgoing: false,
                                        };

                                        ws_state.dispatch(WebSocketAction::AddMessage(ws_msg));
                                    }
                                    Ok(Message::Bytes(_)) => {
                                        // Handle binary messages if needed
                                    }
                                    Err(e) => {
                                        ws_state.dispatch(WebSocketAction::SetConnectionState(
                                            ConnectionState::Error(format!("WebSocket error: {e:?}"))
                                        ));
                                        break;
                                    }
                                }
                            }

                            *sender.borrow_mut() = None;
                            ws_state.dispatch(WebSocketAction::SetConnectionState(ConnectionState::Disconnected));
                        });
                    }
                    Err(e) => {
                        ws_state.dispatch(WebSocketAction::SetConnectionState(
                            ConnectionState::Error(format!("Failed to connect: {e:?}"))
                        ));
                    }
                }
            });
        })
    };

    let disconnect = {
        let ws_state = ws_state.clone();
        let sender = sender.clone();

        Callback::from(move |_: web_sys::MouseEvent| {
            *sender.borrow_mut() = None;
            ws_state.dispatch(WebSocketAction::SetConnectionState(ConnectionState::Disconnected));
        })
    };

    let send_message = {
        let ws_state = ws_state.clone();
        let sender = sender.clone();

        Callback::from(move |text: String| {
            let ws_state = ws_state.clone();
            let sender = sender.clone();

            // Add outgoing message to the list
            let ws_msg = WebSocketMessage {
                content: text.clone(),
                timestamp: js_sys::Date::new_0().to_iso_string().into(),
                is_outgoing: true,
            };

            ws_state.dispatch(WebSocketAction::AddMessage(ws_msg));

            // Send the actual message through the WebSocket
            if let Some(mut write) = sender.borrow_mut().take() {
                let sender_clone = sender.clone();
                spawn_local(async move {
                    if (write.send(Message::Text(text)).await).is_err() {
                        // Handle send error - connection might be closed
                    }
                    *sender_clone.borrow_mut() = Some(write);
                });
            }
        })
    };

    (ws_state, connect, disconnect, send_message)
}
