use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::{HomePage, SessionPage};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/session/:id")]
    Session { id: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <HomePage /> },
        Route::Session { id } => html! { <SessionPage id={id} /> },
        Route::NotFound => html! { <div>{ "404 Not Found" }</div> },
    }
}
