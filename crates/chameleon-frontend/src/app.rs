use yew::prelude::*;

use crate::contexts::{current_user, theme};

#[function_component]
pub fn App() -> Html {
    html! {
        <theme::Provider>
            <current_user::Provider>
                { "app" }
            </current_user::Provider>
        </theme::Provider>
    }
}
