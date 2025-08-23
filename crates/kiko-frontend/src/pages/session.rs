use yew::prelude::*;

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

    // WebSocket connection
    let ws = use_websocket("ws://localhost:3030/api/v1/ws");

    let session_id = props.id.clone();

    // Initial session load
    use_effect_with(session_id.clone(), {
        let api = api.clone();
        let session_data = session_data.clone();
        let loading = loading.clone();
        let error_msg = error_msg.clone();

        move |id: &String| {
            let api = api.clone();
            let session_data = session_data.clone();
            let loading = loading.clone();
            let error_msg = error_msg.clone();
            let id = id.clone();

            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                error_msg.set(None);

                match api.fetch_session(&id).await {
                    Ok(session) => {
                        info!("✅ Session loaded: {:?}", session);
                        session_data.set(Some(session));
                        loading.set(false);
                    }
                    Err(err) => {
                        info!("❌ Session error: {:?}", err);
                        loading.set(false);
                        error_msg.set(Some(format!("Failed to load session: {err}")));
                    }
                }
            });
        }
    });

    // Set up WebSocket message handler
    {
        let session_data = session_data.clone();
        let ws_set_on_message = ws.set_on_message.clone();

        use_effect(move || {
            let message_callback = Callback::from({
                let session_data = session_data.clone();
                move |text: String| {
                    info!("📨 Received WebSocket message: {}", text);

                    // Try to parse as SessionMessage
                    match serde_json::from_str::<SessionMessage>(&text) {
                        Ok(SessionMessage::SessionUpdate(updated_session)) => {
                            info!("🔄 Session update received");
                            session_data.set(Some(updated_session));
                        }
                        Ok(other_msg) => {
                            info!("📥 Other message type received: {:?}", other_msg);
                        }
                        Err(e) => {
                            info!("❌ Failed to parse session message: {:?}", e);
                        }
                    }
                }
            });

            ws_set_on_message.emit(message_callback);
        });
    }

    // Auto-connect and join session when WebSocket is ready
    {
        let session_id = session_id.clone();
        let ws_connect = ws.connect.clone();
        let ws_send = ws.send.clone();
        let ws_state = ws.state.clone();

        use_effect_with((session_id.clone(), ws_state), move |(id, state)| {
            match state {
                ConnectionState::Disconnected => {
                    info!("🔌 Connecting to WebSocket...");
                    ws_connect.emit(());
                }
                ConnectionState::Connected => {
                    info!("✅ WebSocket connected, joining session...");
                    let join_message = SessionMessage::JoinSession(JoinSession {
                        session_id: id.clone(),
                        participant_name: "Anonymous User".to_string(), // TODO: Get from user input
                    });

                    if let Ok(message_text) = serde_json::to_string(&join_message) {
                        ws_send.emit(message_text);
                        info!("📤 Sent join message for session: {}", id);
                    }
                }
                ConnectionState::Error(err) => {
                    info!("❌ WebSocket error: {}", err);
                }
                _ => {}
            }
        });
    };

    let refresh_session = async_callback!([api, session_data, loading, error_msg, session_id] {
        loading.set(true);
        error_msg.set(None);

        match api.fetch_session(&session_id).await {
            Ok(session) => {
                session_data.set(Some(session));
                loading.set(false);
            }
            Err(err) => {
                loading.set(false);
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
                        <span class="text-sm text-gray-600">{
                            match &ws.state {
                                ConnectionState::Connected => "Connected",
                                ConnectionState::Connecting => "Connecting...",
                                ConnectionState::Disconnected => "Disconnected",
                                ConnectionState::Error(_) => "Error",
                            }
                        }</span>
                    </div>
                </div>
            </div>

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
                    html! { <SessionView session={session.clone()} on_refresh={refresh_session.clone()} /> }
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
