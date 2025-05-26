mod api;

use std::rc::Rc;

use yew::prelude::*;

use kiko::data::HelloWorld;

macro_rules! async_callback {
    // Version without event parameter
    ([$($var:ident),* $(,)?] $body:expr) => {
        {
            $(let $var = $var.clone();)*
            Callback::from(move |_| {
                $(let $var = $var.clone();)*
                wasm_bindgen_futures::spawn_local(async move {
                    $body
                });
            })
        }
    };

    // Version with event parameter
    ([$($var:ident),* $(,)?] |$event:ident| $body:expr) => {
        {
            $(let $var = $var.clone();)*
            Callback::from(move |$event| {
                $(let $var = $var.clone();)*
                wasm_bindgen_futures::spawn_local(async move {
                    $body
                });
            })
        }
    };
}

#[function_component(App)]
fn app() -> Html {
    let api = use_memo((), |_| Rc::new(api::create()));
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
                error_msg.set(Some(format!("Error fetching data: {}", err)));
            }
        }
    });

    html! {
        <div class="p-8">
            <h1 class="text-2xl font-bold mb-4">{ "Kiko Pointing Poker" }</h1>

            <button
                class="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600 disabled:opacity-50"
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
    }
}

fn main() {
    kiko::log::setup().expect("Failed to setup logging");
    yew::Renderer::<App>::new().render();
}
