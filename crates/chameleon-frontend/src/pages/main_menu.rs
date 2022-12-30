use yew::prelude::*;
use yew_router::prelude::*;

use crate::app::Route;

#[function_component]
pub fn MainMenu() -> Html {
    html! {
        <div class="main-menu">
            <div>
                <Link<Route> to={Route::Browse}>{ "Browse" }</Link<Route>>
            </div>
            <div>
                <Link<Route> to={Route::Host}>{ "Host" }</Link<Route>>
            </div>
        </div>
    }
}
