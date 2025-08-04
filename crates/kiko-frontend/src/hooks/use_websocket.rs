use std::cell::RefCell;
use std::rc::Rc;

use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{Message, futures::WebSocket};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use kiko::async_callback;

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

pub type MessageCallback = Callback<String>;

type WebSocketSender = Rc<RefCell<Option<SplitSink<WebSocket, Message>>>>;

pub struct WebSocketHandle {
    pub state: ConnectionState,
    pub connect: Callback<()>,
    pub disconnect: Callback<()>,
    pub send: Callback<String>,
    pub set_on_message: Callback<MessageCallback>,
}

#[hook]
pub fn use_websocket(url: &str) -> WebSocketHandle {
    let state = use_state(|| ConnectionState::Disconnected);
    let sender: WebSocketSender = use_mut_ref(|| None);
    let on_message_callback: Rc<RefCell<Option<MessageCallback>>> = use_mut_ref(|| None);

    let url = url.to_string();

    let connect = async_callback!([state, sender, on_message_callback, url] {
        state.set(ConnectionState::Connecting);

        match WebSocket::open(&url) {
            Ok(ws) => {
                state.set(ConnectionState::Connected);

                let (write, mut read) = ws.split();
                *sender.borrow_mut() = Some(write);

                // Handle incoming messages
                spawn_local(async move {
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                if let Some(callback) =
                                    on_message_callback.borrow().as_ref()
                                {
                                    callback.emit(text);
                                }
                            }
                            Ok(Message::Bytes(_)) => {
                                // Handle binary messages if needed
                            }
                            Err(e) => {
                                state.set(ConnectionState::Error(format!(
                                    "WebSocket error: {e:?}"
                                )));
                                break;
                            }
                        }
                    }

                    *sender.borrow_mut() = None;
                    state.set(ConnectionState::Disconnected);
                });
            }
            Err(e) => {
                state.set(ConnectionState::Error(format!("Failed to connect: {e:?}")));
            }
        }
    });

    let disconnect = {
        let state = state.clone();
        let sender = sender.clone();

        Callback::from(move |_: ()| {
            *sender.borrow_mut() = None;
            state.set(ConnectionState::Disconnected);
        })
    };

    let send = {
        let sender = sender.clone();

        Callback::from(move |text: String| {
            let sender = sender.clone();

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
