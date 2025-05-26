#[macro_export]
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
