use yew::prelude::*;

use crate::components::CreateSession;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    html! {
        <div class="p-8">
            <h1 class="text-2xl font-bold mb-4">{ "Kiko Pointing Poker" }</h1>

            // Create Session Section
            <div class="mb-8">
                <CreateSession />
            </div>
        </div>
    }
}
