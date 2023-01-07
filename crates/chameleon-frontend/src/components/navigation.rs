use yew::prelude::*;
use yew_router::prelude::*;

use crate::{app::Route, components::theme_picker::ThemePicker};

#[function_component]
pub fn Navigation() -> Html {
    html! {
        <div class="navigation">
            <div class="navigation--item">
                <Link<Route> classes="navigation--link" to={Route::MainMenu}>
                    { "main menu" }
                </Link<Route>>
            </div>
            <div class="navigation--item">
                <Link<Route> classes="navigation--link" to={Route::Browse}>
                    { "browse" }
                </Link<Route>>
            </div>
            <div class="navigation--item">
                <Link<Route> classes="navigation--link" to={Route::Host}>
                    { "host" }
                </Link<Route>>
            </div>
            <div class="navigation--item navigation--spacer"></div>
            <div class="navigation--item">
                <div class="navigation--theme-picker">
                    <ThemePicker />
                </div>
            </div>
        </div>
    }
}
