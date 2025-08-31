use yew::prelude::*;

use crate::components::ThemeToggle;

#[function_component(Header)]
pub fn header() -> Html {
    html! {
        <header class="px-6 py-3 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100">
            <div class="max-w-7xl mx-auto flex justify-between items-center">
                <h1 class="text-lg font-bold">{ "Kiko Pointing Poker" }</h1>
                <ThemeToggle />
            </div>
        </header>
    }
}
