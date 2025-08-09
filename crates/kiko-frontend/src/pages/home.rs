use yew::prelude::*;

use kiko::{async_callback, data::HelloWorld};

use crate::components::{CreateSession, WebSocketChat};
use crate::providers::api;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let api = use_memo((), |_| api::create());
    let hello_data = use_state(|| None::<HelloWorld>);
    let loading = use_state(|| false);
    let error_msg = use_state(|| None::<String>);

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

            // Create Session Section
            <div class="mb-8">
                <CreateSession />
            </div>

            // WebSocket Section
            <WebSocketChat url="ws://127.0.0.1:3030/api/v1/ws" />
        </div>
    }
}
