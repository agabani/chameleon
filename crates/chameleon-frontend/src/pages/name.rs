use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::contexts::current_user::{CurrentUserContext, CurrentUserState};

#[function_component]
pub fn Name() -> Html {
    let context = use_context::<CurrentUserContext>().unwrap();
    let node_ref = use_node_ref();

    let onsubmit = {
        let context = context.clone();
        let node_ref = node_ref.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let _value = node_ref.cast::<HtmlInputElement>().unwrap().value();
            context.dispatch(CurrentUserState {
                authenticated: true,
            });
        })
    };

    html! {
        <div class="name">
            <div>{ "welcome" }</div>
            <form onsubmit={onsubmit}>
                <div>{ "name" }</div>
                <div><input ref={node_ref} /></div>
                <div><button>{ "continue" }</button></div>
            </form>
        </div>
    }
}
