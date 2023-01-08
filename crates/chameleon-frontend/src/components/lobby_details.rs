use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component]
pub fn LobbyDetails(props: &Props) -> Html {
    let node_ref = use_node_ref();

    let onsubmit = {
        use_callback(
            |event, (callback, node_ref)| handle_onsubmit(&event, callback, node_ref),
            (props.onsubmit.clone(), node_ref.clone()),
        )
    };

    html! {
        <div class="lobby-details">
            <div class="lobby-details--name">{ &props.name }</div>
            <div class="lobby-details--host-name">{ "host name: " }{ &props.host_name }</div>
            <form {onsubmit}>
                if props.require_passcode {
                    <div class="lobby-details--passcode">
                        <label>{ "passcode: " }</label>
                        <input ref={node_ref} type="password" />
                    </div>
                }
                <button type="submit">{ "join" }</button>
            </form>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,

    pub host_name: AttrValue,

    pub name: AttrValue,

    pub require_passcode: bool,

    #[prop_or_default]
    pub onsubmit: Callback<OnsubmitEvent>,
}

fn handle_onsubmit(event: &SubmitEvent, callback: &Callback<OnsubmitEvent>, node_ref: &NodeRef) {
    event.prevent_default();

    let passcode = node_ref
        .cast::<HtmlInputElement>()
        .map(|element| element.value().into());

    callback.emit(OnsubmitEvent { passcode });
}

pub struct OnsubmitEvent {
    pub passcode: Option<AttrValue>,
}
