#[macro_export]
/// Allow defining an async callback that can be used in Yew components.
/// This macro simplifies the creation of async callbacks by automatically
/// handling the cloning of variables and the spawning of async tasks.
///
/// ## With the macro
/// The macro can be used in two forms:
///
/// 1. Without an event parameter:
/// ```compile_fail
/// let fetch_data = async_callback!([api, hello_data, loading, error_msg] {
///     loading.set(true);
///     error_msg.set(None);
///     match api.fetch_hello().await {
///         Ok(data) => {
///             hello_data.set(Some(data));
///             loading.set(false);
///         }
///         Err(err) => {
///             loading.set(false);
///             error_msg.set(Some(format!("Error fetching data: {}", err)));
///         }
///     }
/// });
/// ```
///
/// 2. With an event parameter:
/// ```compile_fail
/// let on_click = async_callback!([api, hello_data, loading, error_msg] |event| {
///     event.prevent_default();
///     loading.set(true);
///     error_msg.set(None);
///     match api.fetch_hello().await {
///         Ok(data) => {
///             hello_data.set(Some(data));
///             loading.set(false);
///         }
///         Err(err) => {
///             loading.set(false);
///             error_msg.set(Some(format!("Error fetching data: {}", err)));
///         }
///     }
/// });
/// ```
///
/// ## Without the macro
/// ```compile_fail
/// // Manual approach - verbose and error-prone
/// let api_clone = api.clone();
/// let hello_data_clone = hello_data.clone();
/// let loading_clone = loading.clone();
/// let error_msg_clone = error_msg.clone();
/// let fetch_data = Callback::from(move |_| {
///     let api = api_clone.clone();
///     let hello_data = hello_data_clone.clone();
///     let loading = loading_clone.clone();
///     let error_msg = error_msg_clone.clone();
///     wasm_bindgen_futures::spawn_local(async move {
///         loading.set(true);
///         error_msg.set(None);
///         match api.fetch_hello().await {
///             Ok(data) => {
///                 hello_data.set(Some(data));
///                 loading.set(false);
///             }
///             Err(err) => {
///                 loading.set(false);
///                 error_msg.set(Some(format!("Error fetching data: {}", err)));
///             }
///         }
///     });
/// });
/// ```
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
