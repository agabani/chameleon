use web_sys::HtmlInputElement;
use yew::prelude::*;

#[hook]
pub fn use_input(initial: AttrValue) -> Input {
    let node_ref = use_node_ref();
    let state = use_state(|| initial);
    let callback = use_callback(
        move |_: Event, (node_ref, state)| {
            let value = node_ref.cast::<HtmlInputElement>().unwrap().value();
            state.set(value.into());
        },
        (node_ref.clone(), state.clone()),
    );

    Input {
        callback,
        node_ref,
        state,
    }
}

pub struct Input {
    pub callback: Callback<Event>,
    pub node_ref: NodeRef,
    pub state: UseStateHandle<AttrValue>,
}
