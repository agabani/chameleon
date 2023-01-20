use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component]
pub fn UserForm(props: &Props) -> Html {
    let name = use_node_ref();

    let onsubmit = use_callback(
        move |event, (callback, name)| {
            handle_form_submit(&event, callback, name);
        },
        (props.onsubmit.clone(), name.clone()),
    );

    html! {
        <div class="user-form">
            <form class="user-form--form" {onsubmit}>
                <div class="user-form--input-group">
                    <label class="user-form--label">{ "name:" }</label>
                    <input class="user-form--input" disabled={props.disabled} type="text" value={&props.name} ref={name} />
                </div>
                <div class="user-form--input-group">
                    <button class="user-form--button" disabled={props.disabled} type="submit">{ "continue" }</button>
                </div>
            </form>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub disabled: bool,

    pub name: Option<AttrValue>,

    #[prop_or_default]
    pub onsubmit: Callback<OnsubmitEvent>,
}

fn handle_form_submit(event: &SubmitEvent, callback: &Callback<OnsubmitEvent>, name: &NodeRef) {
    event.prevent_default();

    let name = name.cast::<HtmlInputElement>().unwrap().value().into();

    callback.emit(OnsubmitEvent { name });
}

pub struct OnsubmitEvent {
    pub name: AttrValue,
}
