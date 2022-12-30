use yew::prelude::*;

use crate::hooks::input::use_input;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub disabled: bool,
    pub onsubmit: Callback<AttrValue>,
}

#[function_component]
pub fn LobbyCreationForm(props: &Props) -> Html {
    let name = use_input(String::new());
    let onsubmit = use_callback(
        |event: SubmitEvent, (state, callback)| {
            event.prevent_default();
            callback.emit(state.to_string().into());
        },
        (name.state.clone(), props.onsubmit.clone()),
    );

    html! {
        <div class="lobby-creation-form">
            <form onsubmit={onsubmit}>
                <input disabled={props.disabled} ref={name.node_ref} onchange={name.callback} value={name.state.to_string()} />
                <button disabled={props.disabled} type="submit">{ "host" }</button>
            </form>
        </div>
    }
}
