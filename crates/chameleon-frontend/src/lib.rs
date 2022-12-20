#![deny(clippy::pedantic)]

mod components;
mod pages;
mod services;

use std::rc::Rc;

use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    pages::{home::Home, lobby::Lobby, not_found::NotFound, test::Test},
    services::Service,
};

#[derive(Clone, PartialEq, Eq, Routable)]
enum Route {
    #[at("/")]
    Home,
    #[at("/lobby")]
    Lobby,
    #[at("/test")]
    Test,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[allow(clippy::needless_pass_by_value)] // reason = "required by `yew_router::switch::Switch`"
fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! {<Home />},
        Route::Lobby => html! {<Lobby />},
        Route::NotFound => html! {<NotFound />},
        Route::Test => html! {<Test />},
    }
}

#[function_component]
pub fn App() -> Html {
    let service = use_memo(|_| Service::default(), ());

    html! {
        <ContextProvider<Rc<Service>> context={service}>
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </ContextProvider<Rc<Service>>>
    }
}
