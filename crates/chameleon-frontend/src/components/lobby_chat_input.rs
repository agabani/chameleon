use yew::prelude::*;

use crate::hooks::input::use_input;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub onsubmit: Callback<AttrValue>,
}

#[function_component]
pub fn LobbyChatInput(props: &Props) -> Html {
    let message = use_input(String::new());
    let onsubmit = use_callback(
        |event: SubmitEvent, (state, callback)| {
            event.prevent_default();
            callback.emit(state.to_string().into());
        },
        (message.state.clone(), props.onsubmit.clone()),
    );

    html! {
        <div class="lobby-chat-input">
            <form onsubmit={onsubmit}>
                <input ref={message.node_ref} onchange={message.callback} value={message.state.to_string()} />
                <button type="submit">{ "send" }</button>
            </form>
        </div>
    }
}
