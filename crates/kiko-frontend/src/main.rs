mod api;
mod websocket;

use yew::prelude::*;

use kiko::{async_callback, data::HelloWorld};
use websocket::{ConnectionState, use_websocket};

#[function_component(App)]
fn app() -> Html {
    let api = use_memo((), |_| api::create());
    let hello_data = use_state(|| None::<HelloWorld>);
    let loading = use_state(|| false);
    let error_msg = use_state(|| None::<String>);

    // WebSocket hook
    let (ws_state, ws_connect, ws_disconnect, ws_send) = use_websocket();
    let message_input = use_state(String::new);
    
    // Ref for messages container to enable auto-scrolling
    let messages_container_ref = use_node_ref();
    
    // Auto-scroll to bottom when messages change
    {
        let messages_container_ref = messages_container_ref.clone();
        let messages_len = ws_state.messages.len();
        
        use_effect_with(
            messages_len,
            move |_| {
                if let Some(container) = messages_container_ref.cast::<web_sys::HtmlElement>() {
                    container.set_scroll_top(container.scroll_height());
                }
            },
        );
    }

    let fetch_data = async_callback!([api, hello_data, loading, error_msg] {
        loading.set(true);
        error_msg.set(None);

        match api.fetch_hello().await {
            Ok(data) => {
                hello_data.set(Some(data));
                loading.set(false);
            }
            Err(err) => {
                loading.set(false);
                error_msg.set(Some(format!("Error fetching data: {err}")));
            }
        }
    });

    html! {
        <div class="p-8">
            <h1 class="text-2xl font-bold mb-4">{ "Kiko Pointing Poker" }</h1>

            // HTTP API Section
            <div class="mb-8 p-4 border border-gray-200 rounded">
                <h2 class="text-xl font-semibold mb-4">{ "HTTP API" }</h2>

                <button
                    class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 disabled:opacity-50 cursor-pointer"
                    onclick={fetch_data}
                    disabled={*loading}
                >
                    { if *loading { "Loading..." } else { "Fetch Hello" } }
                </button>

                // Error display
                {
                    if let Some(error) = error_msg.as_ref() {
                        html! {
                            <div class="mt-4 p-4 bg-red-100 text-red-700 rounded">
                                <p>{ error }</p>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }

                // Success display
                {
                    if let Some(hello) = hello_data.as_ref() {
                        html! {
                            <div class="mt-4 p-4 bg-green-100 rounded">
                                <p>{ &hello.message }</p>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>

            // WebSocket Section
            <div class="p-4 border border-gray-200 rounded">
                <h2 class="text-xl font-semibold mb-4">{ "WebSocket Connection" }</h2>

                // Connection status
                <div class="mb-4">
                    <span class="font-medium">{ "Status: " }</span>
                    <span class={match ws_state.connection_state {
                        ConnectionState::Connected => "text-green-600",
                        ConnectionState::Connecting => "text-yellow-600",
                        ConnectionState::Disconnected => "text-gray-600",
                        ConnectionState::Error(_) => "text-red-600",
                    }}>
                        { match &ws_state.connection_state {
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
                        onclick={ws_connect}
                        disabled={matches!(ws_state.connection_state, ConnectionState::Connected)}
                    >
                        { "Connect" }
                    </button>

                    <button
                        class="bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700 disabled:opacity-50"
                        onclick={ws_disconnect}
                        disabled={!matches!(ws_state.connection_state, ConnectionState::Connected)}
                    >
                        { "Disconnect" }
                    </button>
                </div>

                // Message input
                {
                    if matches!(ws_state.connection_state, ConnectionState::Connected) {
                        let message_input_clone = message_input.clone();
                        let ws_send_clone = ws_send.clone();

                        let send_message = Callback::from(move |_| {
                            let msg = (*message_input_clone).clone();
                            if !msg.is_empty() {
                                ws_send_clone.emit(msg);
                                message_input_clone.set(String::new());
                            }
                        });

                        let on_input = {
                            let message_input = message_input.clone();
                            Callback::from(move |e: InputEvent| {
                                if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                    message_input.set(input.value());
                                }
                            })
                        };
                        
                        let on_keypress = {
                            let message_input_clone = message_input.clone();
                            let ws_send_clone = ws_send.clone();
                            
                            Callback::from(move |e: KeyboardEvent| {
                                if e.key() == "Enter" {
                                    let msg = (*message_input_clone).clone();
                                    if !msg.is_empty() {
                                        ws_send_clone.emit(msg);
                                        message_input_clone.set(String::new());
                                    }
                                }
                            })
                        };

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
                                        onclick={send_message}
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
                        ws_state.messages.iter().map(|msg| {
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
        </div>
    }
}

fn main() {
    kiko::log::setup().expect("Failed to setup logging");
    yew::Renderer::<App>::new().render();
}
