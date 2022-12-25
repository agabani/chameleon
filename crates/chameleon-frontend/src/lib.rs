#![deny(clippy::pedantic)]

mod components;
mod pages;
mod services;

use std::rc::Rc;

use pages::{browse::Browse, host::Host};
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    pages::{home::Home, not_found::NotFound, test::Test},
    services::Service,
};

#[derive(Clone, PartialEq, Eq, Routable)]
enum Route {
    #[at("/")]
    Home,
    #[at("/browse")]
    Browse,
    #[at("/host")]
    Host,
    #[at("/test")]
    Test,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[allow(clippy::needless_pass_by_value)] // reason = "required by `yew_router::switch::Switch`"
fn render(route: Route) -> Html {
    match route {
        Route::Home => html! {<Home />},
        Route::Browse => html! {<Browse />},
        Route::Host => html! {<Host />},
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
                <div class={classes!("app", "theme-dark")}>
                    <Switch<Route> render={render} />
                </div>
            </BrowserRouter>
        </ContextProvider<Rc<Service>>>
    }
}
