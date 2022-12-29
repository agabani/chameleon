use yew::prelude::*;

#[function_component]
pub fn MainMenu() -> Html {
    html! {
        <div class="main-menu">
            <div>{ "Browse" }</div>
            <div>{ "Host" }</div>
        </div>
    }
}
