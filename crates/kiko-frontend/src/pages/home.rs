use yew::prelude::*;

use crate::components::{CreateSession, ThemeToggle};

#[function_component(HomePage)]
pub fn home_page() -> Html {
    html! {
        <div class="min-h-screen bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100">
            <div class="p-8">
                <div class="flex justify-between items-center mb-8">
                    <h1 class="text-2xl font-bold">{ "Kiko Pointing Poker" }</h1>
                    <ThemeToggle />
                </div>

                // Create Session Section
                <div class="mb-8">
                    <CreateSession />
                </div>
            </div>
        </div>
    }
}
