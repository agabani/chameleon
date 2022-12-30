use yew::prelude::*;

use crate::components::lobby_creation_form::LobbyCreationForm;

#[function_component]
pub fn Host() -> Html {
    let disabled = use_state(|| false);
    let onsubmit = use_callback(
        |name: AttrValue, disabled| {
            gloo::console::log!(name.as_str());
            disabled.set(true);
        },
        disabled.clone(),
    );

    html! {
        <div class="host">
            <div>{ "Host" }</div>
            <LobbyCreationForm disabled={*disabled} onsubmit={onsubmit} />
        </div>
    }
}
