use std::str::FromStr;

use yew::prelude::*;

use crate::hooks::input::use_input;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub disabled: bool,
    pub onsubmit: Callback<(AttrValue, AttrValue, bool)>,
}

#[function_component]
pub fn LobbyCreationForm(props: &Props) -> Html {
    let name = use_input(String::new());
    let passcode = use_input(String::new());
    let require_passcode = use_input(String::new());

    let onsubmit = use_callback(
        |event: SubmitEvent, (name, passcode, require_passcode, callback)| {
            event.prevent_default();

            let require_passcode = bool::from_str(require_passcode.as_str()).unwrap_or_default();

            callback.emit((
                name.to_string().into(),
                passcode.to_string().into(),
                require_passcode,
            ));
        },
        (
            name.state.clone(),
            passcode.state.clone(),
            require_passcode.state.clone(),
            props.onsubmit.clone(),
        ),
    );

    html! {
        <div class="lobby-creation-form">
            <form onsubmit={onsubmit}>
                <div>
                    <label>{ "name: " }</label>
                    <input
                        disabled={props.disabled}
                        onchange={name.callback}
                        ref={name.node_ref}
                        type="text"
                        value={name.state.to_string()} />
                </div>
                <div>
                    <label>{ "require passcode: " }</label>
                    <input
                        disabled={props.disabled}
                        onchange={require_passcode.callback}
                        ref={require_passcode.node_ref}
                        type="checkbox"
                        value="true" />
                </div>
                <div>
                    <label>{ "passcode: " }</label>
                    <input
                        disabled={props.disabled}
                        onchange={passcode.callback}
                        ref={passcode.node_ref}
                        type="password"
                        value={passcode.state.to_string()} />
                </div>
                <div>
                    <button disabled={props.disabled} type="submit">{ "host" }</button>
                </div>
            </form>
        </div>
    }
}
