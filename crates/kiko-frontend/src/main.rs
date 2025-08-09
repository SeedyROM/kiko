mod components;
mod hooks;
mod pages;
mod providers;
mod routes;

use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<routes::Route> render={routes::switch} />
        </BrowserRouter>
    }
}

fn main() {
    kiko::log::setup().expect("Failed to setup logging");
    yew::Renderer::<App>::new().render();
}
