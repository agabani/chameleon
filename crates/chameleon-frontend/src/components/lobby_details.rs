use yew::prelude::*;

use crate::hooks::input::use_input;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,
    pub name: AttrValue,
    pub host: AttrValue,
    pub require_passcode: bool,
    pub onclick: Callback<LobbyDetailsJoinEvent>,
}

#[function_component]
pub fn LobbyDetails(props: &Props) -> Html {
    let passcode = use_input(String::new().into());

    let onsubmit = use_callback(
        move |event: SubmitEvent, (callback, require_passcode, passcode)| {
            event.prevent_default();

            callback.emit(LobbyDetailsJoinEvent {
                passcode: if *require_passcode {
                    Some(passcode.to_string().into())
                } else {
                    None
                },
            });
        },
        (
            props.onclick.clone(),
            props.require_passcode,
            passcode.state.clone(),
        ),
    );

    html! {
        <div class="lobby-details">
            <div>{ &props.id }</div>
            <div>{ &props.name }</div>
            <div>{ &props.host }</div>
            <form onsubmit={onsubmit}>
                {
                    if props.require_passcode {
                        html! {
                            <div>
                                <label>{ "passcode: " }</label>
                                <input
                                    onchange={passcode.callback}
                                    ref={passcode.node_ref}
                                    type="password"
                                    value={passcode.state.to_string()} />
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                <div>
                    <button type="submit">{ "join" }</button>
                </div>
            </form>
        </div>
    }
}

pub struct LobbyDetailsJoinEvent {
    pub passcode: Option<AttrValue>,
}
