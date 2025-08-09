use yew::prelude::*;

use kiko::{async_callback, data::Session, log::info};

use crate::{components::SessionView, providers::api};

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

    let session_id = props.id.clone();
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
                <h1 class="text-3xl font-bold text-gray-900">{ "Session Details" }</h1>
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
