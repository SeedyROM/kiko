use yew::prelude::*;

use crate::components::ThemeToggle;

#[function_component(Header)]
pub fn header() -> Html {
    html! {
        <header class="px-4 md:px-6 py-3 bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100">
            <div class="max-w-7xl mx-auto flex justify-between items-center">
                <a href="/" alt="Back to homepage" class="text-lg font-bold flex gap-x-2">
                    <div>{"🫵"}</div>
                    <div>{"Kiko"}</div>
                </a>
                <ThemeToggle />
            </div>
        </header>
    }
}
