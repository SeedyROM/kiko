use std::collections::VecDeque;

use wasm_bindgen::JsValue;
use yew::prelude::*;

use crate::hooks::{ConnectionState, use_websocket};

#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
    pub content: String,
    pub timestamp: String,
    pub is_outgoing: bool,
}

#[derive(Properties, PartialEq)]
pub struct WebSocketChatProps {
    pub url: String,
}

#[function_component(WebSocketChat)]
pub fn websocket_chat(props: &WebSocketChatProps) -> Html {
    let messages = use_state(VecDeque::<ChatMessage>::new);
    let message_input = use_state(String::new);
    let messages_container_ref = use_node_ref();

    // WebSocket hook - pure transport layer
    let ws = use_websocket(&props.url);

    // Auto-connect on mount
    {
        let ws_connect = ws.connect.clone();
        use_effect_with((), move |_| {
            ws_connect.emit(());
        });
    }

    // Set up message handler - refreshes callback to avoid stale closures
    {
        let messages = messages.clone();
        let set_on_message = ws.set_on_message.clone();

        use_effect(move || {
            let message_callback = Callback::from({
                let messages = messages.clone();
                move |text: String| {
                    let chat_msg = ChatMessage {
                        content: text,
                        timestamp: js_sys::Date::new_0()
                            .to_locale_string("en-US", &JsValue::undefined())
                            .into(),
                        is_outgoing: false,
                    };

                    messages.set({
                        let mut current_msgs = (*messages).clone();
                        current_msgs.push_back(chat_msg);
                        if current_msgs.len() > 100 {
                            current_msgs.pop_front();
                        }
                        current_msgs
                    });
                }
            });

            set_on_message.emit(message_callback);
        });
    }

    // Auto-scroll to bottom when messages change
    {
        let messages_container_ref = messages_container_ref.clone();
        let messages_len = messages.len();

        use_effect_with(messages_len, move |_| {
            if let Some(container) = messages_container_ref.cast::<web_sys::HtmlElement>() {
                container.set_scroll_top(container.scroll_height());
            }
        });
    }

    // Send message helper
    let send_message = {
        let message_input = message_input.clone();
        let messages = messages.clone();
        let ws_send = ws.send.clone();

        Callback::from(move |text: String| {
            if !text.is_empty() {
                // Add to our message history as outgoing
                let chat_msg = ChatMessage {
                    content: text.clone(),
                    // Format timestamp as needed human-readable
                    timestamp: js_sys::Date::new_0()
                        .to_locale_string("en-US", &JsValue::undefined())
                        .into(),
                    is_outgoing: true,
                };

                messages.set({
                    let mut current_msgs = (*messages).clone();
                    current_msgs.push_back(chat_msg);
                    if current_msgs.len() > 100 {
                        current_msgs.pop_front();
                    }
                    current_msgs
                });

                // Send via WebSocket
                ws_send.emit(text);
                message_input.set(String::new());
            }
        })
    };

    // Button click handler
    let on_send_click = {
        let message_input = message_input.clone();
        let send_message = send_message.clone();

        Callback::from(move |_| {
            let msg = (*message_input).clone();
            send_message.emit(msg);
        })
    };

    // Input handlers
    let on_input = {
        let message_input = message_input.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                message_input.set(input.value());
            }
        })
    };

    let on_keypress = {
        let message_input = message_input.clone();
        let send_message = send_message.clone();

        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                let msg = (*message_input).clone();
                send_message.emit(msg);
            }
        })
    };

    html! {
        <div class="p-4 border border-gray-200 rounded">
            <h2 class="text-xl font-semibold mb-4">{ "WebSocket Connection" }</h2>

            // Connection status
            <div class="mb-4">
                <span class="font-medium">{ "Status: " }</span>
                <span class={match ws.state {
                    ConnectionState::Connected => "text-green-600",
                    ConnectionState::Connecting => "text-yellow-600",
                    ConnectionState::Disconnected => "text-gray-600",
                    ConnectionState::Error(_) => "text-red-600",
                }}>
                    { match &ws.state {
                        ConnectionState::Connected => "Connected".to_string(),
                        ConnectionState::Connecting => "Connecting...".to_string(),
                        ConnectionState::Disconnected => "Disconnected".to_string(),
                        ConnectionState::Error(err) => format!("Error: {err}"),
                    }}
                </span>
            </div>

            // Connection controls
            <div class="mb-4 space-x-2">
                <button
                    class="bg-green-600 text-white px-4 py-2 rounded hover:bg-green-700 disabled:opacity-50"
                    onclick={ws.connect.reform(|_| ())}
                    disabled={matches!(ws.state, ConnectionState::Connected)}
                >
                    { "Connect" }
                </button>

                <button
                    class="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700 disabled:opacity-50"
                    onclick={ws.disconnect.reform(|_| ())}
                    disabled={!matches!(ws.state, ConnectionState::Connected)}
                >
                    { "Disconnect" }
                </button>
            </div>

            // Message input
            {
                if matches!(ws.state, ConnectionState::Connected) {
                    html! {
                        <div class="mb-4">
                            <div class="flex space-x-2">
                                <input
                                    type="text"
                                    class="flex-1 px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    placeholder="Enter message..."
                                    value={(*message_input).clone()}
                                    oninput={on_input}
                                    onkeypress={on_keypress}
                                />
                                <button
                                    class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700"
                                    onclick={on_send_click}
                                >
                                    { "Send" }
                                </button>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            // Messages display
            <div
                ref={messages_container_ref}
                class="border border-gray-300 rounded p-4 h-64 overflow-y-auto bg-gray-50"
            >
                {
                    messages.iter().map(|msg| {
                        html! {
                            <div class={format!("mb-2 {}", if msg.is_outgoing { "text-right" } else { "text-left" })}>
                                <div class={format!("inline-block px-3 py-2 rounded max-w-xs {}",
                                    if msg.is_outgoing {
                                        "bg-blue-600 text-white"
                                    } else {
                                        "bg-white border border-gray-300"
                                    }
                                )}>
                                    <div class="font-medium">{ &msg.content }</div>
                                    <div class="text-xs opacity-70 mt-1">{ &msg.timestamp }</div>
                                </div>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>
        </div>
    }
}
