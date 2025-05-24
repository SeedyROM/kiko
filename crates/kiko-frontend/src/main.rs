mod api;

use yew::prelude::*;

use kiko::data::HelloWorld;

#[function_component(App)]
fn app() -> Html {
    let hello_data = use_state(|| None::<HelloWorld>);
    let loading = use_state(|| false);

    let hello_data_clone = hello_data.clone();
    let loading_clone = loading.clone();

    let fetch_data = Callback::from(move |_| {
        let hello_data = hello_data_clone.clone();
        let loading = loading_clone.clone();

        loading.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            match api::fetch_hello().await {
                Ok(data) => {
                    hello_data.set(Some(data));
                    loading.set(false);
                }
                Err(_) => {
                    loading.set(false);
                    // Handle error
                }
            }
        });
    });

    html! {
        <div class="p-8">
            <h1 class="text-2xl font-bold mb-4">{ "Kiko Pointing Poker" }</h1>

            <button
                class="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600"
                onclick={fetch_data}
            >
                { if *loading { "Loading..." } else { "Fetch Hello" } }
            </button>

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
