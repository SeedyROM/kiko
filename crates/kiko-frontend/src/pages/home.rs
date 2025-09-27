use yew::prelude::*;

use crate::components::CreateSession;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    html! {
        <div class="min-h-screen p-6">
            <div class="max-w-7xl mx-auto">
                // Create Session Section
                <div class="mb-6">
                    <CreateSession />
                </div>
            </div>
        </div>
    }
}
