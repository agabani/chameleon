use yew::prelude::*;

use crate::components::navigation::Navigation;

#[function_component]
pub fn MainMenu() -> Html {
    html! {
        <div class="main-menu">
            <div class="main-menu--header">
                <Navigation />
            </div>
            <div class="main-menu--content">
            </div>
        </div>
    }
}
