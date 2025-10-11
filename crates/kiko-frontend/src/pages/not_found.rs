use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::Route;

#[function_component(NotFoundPage)]
pub fn not_found_page() -> Html {
    html! {
        <div class="min-h-screen p-4 bg-white dark:bg-gray-900">
            <div class="max-w-7xl mx-auto">
                <div class="flex flex-col items-center justify-center min-h-[80vh]">
                    <div class="text-center space-y-6">
                        <div class="text-9xl font-bold text-gray-900 dark:text-gray-100">
                            {"404"}
                        </div>
                        <div class="text-2xl font-semibold text-gray-700 dark:text-gray-300">
                            {"Page Not Found"}
                        </div>
                        <div class="text-gray-600 dark:text-gray-400 max-w-md mx-auto">
                            {"The page you're looking for doesn't exist or has been moved."}
                        </div>
                        <div class="pt-4">
                            <Link<Route> to={Route::Home}
                                classes="inline-block px-6 py-3 bg-blue-600 hover:bg-blue-700 dark:bg-blue-500 dark:hover:bg-blue-600 text-white font-semibold rounded-lg transition-colors duration-200"
                            >
                                {"Go Home"}
                            </Link<Route>>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
