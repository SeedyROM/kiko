use yew::prelude::*;

use crate::components::{CreateSession, WebSocketChat};

#[function_component(HomePage)]
pub fn home_page() -> Html {
    html! {
        <div class="p-8">
            <h1 class="text-2xl font-bold mb-4">{ "Kiko Pointing Poker" }</h1>

            // Create Session Section
            <div class="mb-8">
                <CreateSession />
            </div>

            // WebSocket Section
            <WebSocketChat url="ws://127.0.0.1:3030/api/v1/ws" />
        </div>
    }
}
