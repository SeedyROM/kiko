//! Kiko Frontend Application
//!
//! A Yew-based WebAssembly frontend for real-time session management.
//! Provides a web interface for creating, joining, and managing sessions with live updates via WebSockets.

mod components;
mod hooks;
mod pages;
mod providers;
mod routes;

use providers::ThemeProvider;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <ThemeProvider>
            <BrowserRouter>
                <Switch<routes::Route> render={routes::switch} />
            </BrowserRouter>
        </ThemeProvider>
    }
}

fn main() {
    kiko::log::setup().expect("Failed to setup logging");
    yew::Renderer::<App>::new().render();
}
