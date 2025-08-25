use yew::prelude::*;

#[function_component(CopyUrlButton)]
pub fn copy_url_button() -> Html {
    let copy_url = Callback::from(|_| {
        let window = web_sys::window().unwrap();
        let location = window.location();
        let url = location.href().unwrap();
        let navigator = window.navigator();
        let clipboard = navigator.clipboard();

        wasm_bindgen_futures::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&url)).await;
        });
    });

    html! {
        <button
            class="mt-3 px-3 py-1.5 bg-blue-600 text-white text-xs rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
            onclick={copy_url}
        >
            { "Copy URL" }
        </button>
    }
}
