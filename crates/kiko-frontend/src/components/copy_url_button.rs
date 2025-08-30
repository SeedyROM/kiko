use yew::prelude::*;

#[function_component(CopyUrlButton)]
pub fn copy_url_button() -> Html {
    let is_copied = use_state(|| false);

    let copy_url = {
        let is_copied = is_copied.clone();
        Callback::from(move |_| {
            let window = web_sys::window().unwrap();
            let location = window.location();
            let url = location.href().unwrap();
            let navigator = window.navigator();
            let clipboard = navigator.clipboard();
            let is_copied = is_copied.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let result = wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&url)).await;
                if result.is_ok() {
                    is_copied.set(true);

                    // Reset after 2 seconds
                    let is_copied_reset = is_copied.clone();
                    gloo_timers::callback::Timeout::new(2000, move || {
                        is_copied_reset.set(false);
                    })
                    .forget();
                }
            });
        })
    };

    let button_classes = if *is_copied {
        "mt-3 px-3 py-1.5 bg-green-600 hover:bg-green-700 dark:bg-green-700 dark:hover:bg-green-600 text-white text-xs rounded-md focus:outline-none focus:ring-2 focus:ring-green-500 transition-all duration-200"
    } else {
        "mt-3 px-3 py-1.5 bg-blue-600 hover:bg-blue-700 dark:bg-blue-700 dark:hover:bg-blue-600 text-white text-xs rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all duration-200"
    };

    html! {
        <button
            class={button_classes}
            onclick={copy_url}
        >
            {
                if *is_copied {
                    html! {
                        <span class="flex items-center space-x-1">
                            <svg class="h-3 w-3" fill="currentColor" viewBox="0 0 20 20">
                                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                            </svg>
                            <span>{ "Copied!" }</span>
                        </span>
                    }
                } else {
                    html! {
                        <span class="flex items-center space-x-1">
                            <svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                            </svg>
                            <span>{ "Copy URL" }</span>
                        </span>
                    }
                }
            }
        </button>
    }
}
