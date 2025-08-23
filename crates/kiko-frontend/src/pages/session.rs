use yew::prelude::*;
use web_sys::{InputEvent, KeyboardEvent, MouseEvent};

use kiko::{
    async_callback,
    data::{JoinSession, Session, SessionMessage},
    log::info,
    serde_json,
};

use crate::{
    components::SessionView,
    hooks::{ConnectionState, use_websocket},
    providers::api,
};

#[derive(Properties, PartialEq)]
pub struct SessionProps {
    pub id: String,
}

#[function_component(SessionPage)]
pub fn session_page(props: &SessionProps) -> Html {
    let api = use_memo((), |_| api::create());
    let session_data = use_state(|| None::<Session>);
    let loading = use_state(|| true);
    let error_msg = use_state(|| None::<String>);
    let ws_error = use_state(|| None::<String>);
    let join_attempted = use_state(|| false);
    let session_exists = use_state(|| false);
    let participant_name = use_state(String::new);
    let is_joined = use_state(|| false);

    // WebSocket connection
    let ws = use_websocket("ws://localhost:3030/api/v1/ws");

    let session_id = props.id.clone();

    // Initial session load
    use_effect_with(session_id.clone(), {
        let api = api.clone();
        let session_data = session_data.clone();
        let loading = loading.clone();
        let error_msg = error_msg.clone();
        let session_exists = session_exists.clone();

        move |id: &String| {
            let api = api.clone();
            let session_data = session_data.clone();
            let loading = loading.clone();
            let error_msg = error_msg.clone();
            let session_exists = session_exists.clone();
            let id = id.clone();

            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                error_msg.set(None);
                session_exists.set(false);

                match api.fetch_session(&id).await {
                    Ok(session) => {
                        info!("✅ Session loaded: {:?}", session);
                        session_data.set(Some(session));
                        session_exists.set(true);
                        loading.set(false);
                    }
                    Err(err) => {
                        info!("❌ Session error: {:?}", err);
                        loading.set(false);
                        session_exists.set(false);
                        error_msg.set(Some(format!("Failed to load session: {err}")));
                    }
                }
            });
        }
    });

    // Set up WebSocket message handler
    {
        let session_data = session_data.clone();
        let ws_error = ws_error.clone();
        let ws_set_on_message = ws.set_on_message.clone();

        use_effect(move || {
            let message_callback = Callback::from({
                let session_data = session_data.clone();
                let ws_error = ws_error.clone();
                move |text: String| {
                    info!("📨 Received WebSocket message: {}", text);

                    // Clear any previous WebSocket errors when we receive a message
                    if ws_error.is_some() {
                        ws_error.set(None);
                    }

                    // Check if this is an error message from the server
                    if text.starts_with("Session ") && text.contains("not found") {
                        ws_error.set(Some(text));
                        return;
                    }

                    if text.contains("Invalid message format")
                        || text.contains("Already subscribed")
                    {
                        ws_error.set(Some(text));
                        return;
                    }

                    // Try to parse as SessionMessage
                    match serde_json::from_str::<SessionMessage>(&text) {
                        Ok(SessionMessage::SessionUpdate(updated_session)) => {
                            info!("🔄 Session update received");
                            session_data.set(Some(updated_session));
                        }
                        Ok(other_msg) => {
                            info!("📥 Other message type received: {:?}", other_msg);
                        }
                        Err(err) => {
                            info!("❌ Failed to parse session message: {err:?}");
                            ws_error.set(Some(format!("Failed to parse server message: {err}")));
                        }
                    }
                }
            });

            ws_set_on_message.emit(message_callback);
        });
    }

    // Auto-connect to WebSocket for session updates
    {
        let ws_connect = ws.connect.clone();
        let ws_state = ws.state.clone();
        let session_exists = *session_exists;

        use_effect_with((ws_state, session_exists), move |(state, exists)| {
            // Only attempt websocket connection if session exists
            if !*exists {
                return;
            }

            if state == &ConnectionState::Disconnected {
                info!("🔌 Auto-connecting to WebSocket for session updates...");
                ws_connect.emit(());
            }
        });
    }

    // Join session callback (for participation)
    let join_session = {
        let ws_send = ws.send.clone();
        let ws_state = ws.state.clone();
        let session_id = session_id.clone();
        let participant_name = participant_name.clone();
        let is_joined = is_joined.clone();
        let ws_error = ws_error.clone();

        async_callback!([ws_send, ws_state, session_id, participant_name, is_joined, ws_error] {
            if participant_name.trim().is_empty() {
                ws_error.set(Some("Please enter your name".to_string()));
                return;
            }

            if *is_joined {
                ws_error.set(Some("Already joined this session".to_string()));
                return;
            }

            // Check if WebSocket is connected
            if !matches!(ws_state, ConnectionState::Connected) {
                ws_error.set(Some("WebSocket not connected. Please wait and try again.".to_string()));
                return;
            }

            ws_error.set(None);
            
            // Send join message for participation
            info!("📤 Joining session as participant...");
            let join_message = SessionMessage::JoinSession(JoinSession {
                session_id: session_id.clone(),
                participant_name: participant_name.trim().to_string(),
            });

            if let Ok(message_text) = serde_json::to_string(&join_message) {
                ws_send.emit(message_text);
                info!("📤 Sent join message for session: {}", session_id);
                is_joined.set(true);
            } else {
                ws_error.set(Some("Failed to serialize join message".to_string()));
            }
        })
    };

    let refresh_session = async_callback!([api, session_data, loading, error_msg, session_exists, session_id] {
        loading.set(true);
        error_msg.set(None);
        session_exists.set(false);

        match api.fetch_session(&session_id).await {
            Ok(session) => {
                session_data.set(Some(session));
                session_exists.set(true);
                loading.set(false);
            }
            Err(err) => {
                loading.set(false);
                session_exists.set(false);
                error_msg.set(Some(format!("Failed to refresh session: {err}")));
            }
        }
    });

    html! {
        <div class="p-8 max-w-4xl mx-auto">
            <div class="mb-6">
                <div class="flex justify-between items-center">
                    <h1 class="text-3xl font-bold text-gray-900">{ "Session Details" }</h1>
                    <div class="flex items-center space-x-2">
                        <div class={classes!("w-3", "h-3", "rounded-full", match ws.state {
                            ConnectionState::Connected => "bg-green-500",
                            ConnectionState::Connecting => "bg-yellow-500",
                            ConnectionState::Disconnected => "bg-gray-500",
                            ConnectionState::Error(_) => "bg-red-500",
                        })}></div>
                        <span class={classes!(
                            "text-sm",
                            match &ws.state {
                                ConnectionState::Connected => "text-green-600",
                                ConnectionState::Connecting => "text-yellow-600",
                                ConnectionState::Disconnected => "text-gray-600",
                                ConnectionState::Error(_) => "text-red-600",
                            }
                        )}>{
                            match &ws.state {
                                ConnectionState::Connected => "Connected",
                                ConnectionState::Connecting => "Connecting...",
                                ConnectionState::Disconnected => "Disconnected",
                                ConnectionState::Error(_) => "Connection Error",
                            }
                        }</span>
                    </div>
                </div>
            </div>

            // Show WebSocket error if present
            {
                if let Some(ws_err) = ws_error.as_ref() {
                    html! {
                        <div class="mb-4 bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                            <div class="flex items-start">
                                <div class="flex-shrink-0">
                                    <svg class="h-5 w-5 text-yellow-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                                        <path fill-rule="evenodd" d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.19-1.458-1.515-2.625L8.485 2.495zM10 5a.75.75 0 01.75.75v3.5a.75.75 0 01-1.5 0v-3.5A.75.75 0 0110 5zm0 9a1 1 0 100-2 1 1 0 000 2z" clip-rule="evenodd" />
                                    </svg>
                                </div>
                                <div class="ml-3">
                                    <h3 class="text-sm font-medium text-yellow-800">{ "WebSocket Issue" }</h3>
                                    <p class="text-sm text-yellow-700 mt-1">{ ws_err }</p>
                                    <div class="mt-2">
                                        <button
                                            class="text-sm bg-yellow-100 hover:bg-yellow-200 text-yellow-800 px-3 py-1 rounded-md transition-colors"
                                            onclick={{
                                                let ws_connect = ws.connect.clone();
                                                let ws_error = ws_error.clone();
                                                let join_attempted = join_attempted.clone();
                                                Callback::from(move |_| {
                                                    ws_error.set(None);
                                                    join_attempted.set(false);
                                                    ws_connect.emit(());
                                                })
                                            }}
                                        >
                                            { "Retry Connection" }
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            // Show connection error details for Error state
            {
                if let ConnectionState::Error(err) = &ws.state {
                    html! {
                        <div class="mb-4 bg-red-50 border border-red-200 rounded-lg p-4">
                            <div class="flex items-start">
                                <div class="flex-shrink-0">
                                    <svg class="h-5 w-5 text-red-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.28 7.22a.75.75 0 00-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 101.06 1.06L10 11.06l1.72 1.72a.75.75 0 101.06-1.06L11.06 10l1.72-1.72a.75.75 0 00-1.06-1.06L10 8.94 8.28 7.22z" clip-rule="evenodd" />
                                    </svg>
                                </div>
                                <div class="ml-3">
                                    <h3 class="text-sm font-medium text-red-800">{ "Connection Failed" }</h3>
                                    <p class="text-sm text-red-700 mt-1">{ err }</p>
                                    <div class="mt-2">
                                        <button
                                            class="text-sm bg-red-100 hover:bg-red-200 text-red-800 px-3 py-1 rounded-md transition-colors"
                                            onclick={{
                                                let ws_connect = ws.connect.clone();
                                                let join_attempted = join_attempted.clone();
                                                Callback::from(move |_| {
                                                    join_attempted.set(false);
                                                    ws_connect.emit(());
                                                })
                                            }}
                                        >
                                            { "Retry Connection" }
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            {
                if *loading {
                    html! {
                        <div class="flex items-center justify-center py-12">
                            <div class="flex items-center space-x-2">
                                <svg class="animate-spin h-6 w-6 text-blue-600" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                </svg>
                                <span class="text-gray-600">{ "Loading session..." }</span>
                            </div>
                        </div>
                    }
                } else if let Some(error) = error_msg.as_ref() {
                    html! {
                        <div class="bg-red-50 border border-red-200 rounded-lg p-6">
                            <div>
                                <h2 class="text-lg font-medium text-red-800 mb-2">{ "⚠️ Error Loading Session" }</h2>
                                <p class="text-red-700 mb-4">{ error }</p>
                                <button
                                    class="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500"
                                    onclick={refresh_session}
                                >
                                    { "Retry" }
                                </button>
                            </div>
                        </div>
                    }
                } else if let Some(session) = session_data.as_ref() {
                    html! {
                        <div>
                            // Join session form (only show if not joined)
                            {
                                if !*is_joined {
                                    html! {
                                        <div class="mb-6 bg-blue-50 border border-blue-200 rounded-lg p-6">
                                            <div class="flex items-start">
                                                <div class="flex-shrink-0">
                                                    <svg class="h-5 w-5 text-blue-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                                                        <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a.75.75 0 000 1.5h.253a.25.25 0 01.244.304l-.459 2.066A1.75 1.75 0 0010.747 15H11a.75.75 0 000-1.5h-.253a.25.25 0 01-.244-.304l.459-2.066A1.75 1.75 0 009.253 9H9z" clip-rule="evenodd" />
                                                    </svg>
                                                </div>
                                                <div class="ml-3 flex-1">
                                                    <h3 class="text-sm font-medium text-blue-800">{ "Join as Participant" }</h3>
                                                    <p class="text-sm text-blue-700 mt-1 mb-4">{ "Enter your name to actively participate in this session. You'll receive updates automatically." }</p>
                                                    <div class="flex items-end space-x-3">
                                                        <div class="flex-1">
                                                            <label for="participant-name" class="block text-xs font-medium text-blue-700 mb-1">{ "Your Name" }</label>
                                                            <input
                                                                id="participant-name"
                                                                type="text"
                                                                class="w-full px-3 py-2 border border-blue-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                                placeholder="Enter your name..."
                                                                value={(*participant_name).clone()}
                                                                oninput={{
                                                                    let participant_name = participant_name.clone();
                                                                    Callback::from(move |e: InputEvent| {
                                                                        if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                                            participant_name.set(input.value());
                                                                        }
                                                                    })
                                                                }}
                                                                onkeypress={{
                                                                    let join_session = join_session.clone();
                                                                    Callback::from(move |e: KeyboardEvent| {
                                                                        if e.key() == "Enter" {
                                                                            e.prevent_default();
                                                                            join_session.emit(());
                                                                        }
                                                                    })
                                                                }}
                                                            />
                                                        </div>
                                                        <button
                                                            type="button"
                                                            class={classes!(
                                                                "px-4", "py-2", "rounded-md", "text-sm", "font-medium", "transition-colors",
                                                                if participant_name.trim().is_empty() {
                                                                    "bg-gray-300 text-gray-500 cursor-not-allowed"
                                                                } else {
                                                                    "bg-blue-600 text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                                                }
                                                            )}
                                                            disabled={participant_name.trim().is_empty()}
                                                            onclick={{
                                                                let join_session = join_session.clone();
                                                                Callback::from(move |_: MouseEvent| {
                                                                    join_session.emit(());
                                                                })
                                                            }}
                                                        >
                                                            { "Join as Participant" }
                                                        </button>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    }
                                } else {
                                    html! {
                                        <div class="mb-6 bg-green-50 border border-green-200 rounded-lg p-4">
                                            <div class="flex items-center">
                                                <div class="flex-shrink-0">
                                                    <svg class="h-5 w-5 text-green-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                                                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.857-9.809a.75.75 0 00-1.214-.882l-3.236 4.53L7.53 10.347a.75.75 0 00-1.06 1.061l2.5 2.5a.75.75 0 001.137-.089l4-5.5z" clip-rule="evenodd" />
                                                    </svg>
                                                </div>
                                                <div class="ml-3">
                                                    <p class="text-sm font-medium text-green-800">
                                                        { format!("Participating as {}", participant_name.trim()) }
                                                    </p>
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }
                            }
                            
                            <SessionView session={session.clone()} on_refresh={refresh_session.clone()} />
                        </div>
                    }
                } else {
                    html! {
                        <div class="text-center py-12">
                            <p class="text-gray-500">{ "No session data available" }</p>
                        </div>
                    }
                }
            }
        </div>
    }
}
