use std::ops::Deref;

use yew::prelude::*;

use crate::hooks::input::use_input;

#[derive(PartialEq, Properties)]
pub struct Props {
    /// current user name
    pub current_user_name: AttrValue,

    /// require passcode
    pub require_passcode: bool,

    /// onsubmit callback
    pub onsubmit: Callback<OnsubmitEvent>,
}

#[function_component]
pub fn LobbyLandingForm(props: &Props) -> Html {
    // resources
    let user_name = use_input(props.current_user_name.clone());
    let lobby_passcode = use_input(AttrValue::default());

    // callback
    let onsubmit = {
        let lobby_passcode = lobby_passcode.state.clone();
        let user_name = user_name.state.clone();
        let require_passcode = props.require_passcode;

        use_callback(
            move |event: SubmitEvent, (callback, lobby_passcode, user_name)| {
                event.prevent_default();

                callback.emit(OnsubmitEvent {
                    lobby_passcode: if require_passcode {
                        Some(lobby_passcode.deref().clone())
                    } else {
                        None
                    },
                    user_name: user_name.deref().clone(),
                });
            },
            (props.onsubmit.clone(), lobby_passcode, user_name),
        )
    };

    // layout
    html! {
        <form class="lobby-landing-form" onsubmit={onsubmit}>
            <div>
                <label>{ "enter your name: " }</label>
                <input
                    onchange={user_name.callback}
                    ref={user_name.node_ref}
                    value={user_name.state.to_string()}
                    type="text" />
            </div>
            { if props.require_passcode { html! {
                <div>
                    <label>{ "enter lobby passcode: "}</label>
                    <input
                        onchange={lobby_passcode.callback}
                        ref={lobby_passcode.node_ref}
                        type="password" />
                </div>
            } } else { html! {
                // render nothing
            } } }
            <div>
                <button type="submit">{ "join" }</button>
            </div>
        </form>
    }
}

pub struct OnsubmitEvent {
    /// lobby passcode
    pub lobby_passcode: Option<AttrValue>,

    /// user name
    pub user_name: AttrValue,
}
