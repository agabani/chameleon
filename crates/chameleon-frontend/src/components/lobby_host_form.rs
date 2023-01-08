use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component]
pub fn LobbyHostForm(props: &Props) -> Html {
    let state = use_state(State::default);

    let name = use_node_ref();
    let passcode = use_node_ref();
    let require_passcode = use_node_ref();

    let onchange = {
        let state = state.clone();
        use_callback(
            move |_, node_ref| handle_require_passcode_onchange(&state, node_ref),
            require_passcode.clone(),
        )
    };

    let onsubmit = use_callback(
        move |event, (callback, name, passcode, require_passcode)| {
            handle_form_submit(&event, callback, name, passcode, require_passcode);
        },
        (
            props.onsubmit.clone(),
            name.clone(),
            passcode.clone(),
            require_passcode.clone(),
        ),
    );

    html! {
        <div class="lobby-host-form">
            <form class="lobby-host-form--form" disabled={props.disabled} {onsubmit}>
                <div class="lobby-host-form--input-group">
                    <label class="lobby-host-form--label">{ "name:" }</label>
                    <input class="lobby-host-form--input" disabled={props.disabled} type="text" ref={name} />
                </div>
                <div class="lobby-host-form--input-group">
                    <label class="lobby-host-form--label">{ "require passcode:" }</label>
                    <input
                        class="lobby-host-form--input"
                        disabled={props.disabled}
                        type="checkbox"
                        value="require_passcode"
                        onclick={onchange}
                        ref={require_passcode} />
                </div>
                if state.require_passcode {
                    <div class="lobby-host-form--input-group">
                        <label class="lobby-host-form--label">{ "passcode:" }</label>
                        <input class="lobby-host-form--input" disabled={props.disabled} type="password" ref={passcode} />
                    </div>
                }
                <div class="lobby-host-form--input-group">
                    <button class="lobby-host-form--button" disabled={props.disabled} type="submit">{ "host" }</button>
                </div>
            </form>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub disabled: bool,

    #[prop_or_default]
    pub onsubmit: Callback<OnsubmitEvent>,
}

#[derive(Default)]
struct State {
    require_passcode: bool,
}

fn handle_require_passcode_onchange(state: &UseStateHandle<State>, require_passcode: &NodeRef) {
    if let Some(require_passcode) = require_passcode
        .cast::<HtmlInputElement>()
        .map(|element| element.checked())
    {
        state.set(State { require_passcode });
    }
}

fn handle_form_submit(
    event: &SubmitEvent,
    callback: &Callback<OnsubmitEvent>,
    name: &NodeRef,
    passcode: &NodeRef,
    require_passcode: &NodeRef,
) {
    event.prevent_default();

    let name = name.cast::<HtmlInputElement>().unwrap().value().into();
    let require_passcode = require_passcode
        .cast::<HtmlInputElement>()
        .unwrap()
        .checked();
    let passcode = if require_passcode {
        Some(passcode.cast::<HtmlInputElement>().unwrap().value().into())
    } else {
        None
    };

    callback.emit(OnsubmitEvent {
        name,
        passcode,
        require_passcode,
    });
}

pub struct OnsubmitEvent {
    pub name: AttrValue,
    pub passcode: Option<AttrValue>,
    pub require_passcode: bool,
}
