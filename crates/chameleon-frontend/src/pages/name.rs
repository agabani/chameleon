use chameleon_protocol::jsonapi::Resources;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{
    contexts::current_user::{CurrentUserContext, CurrentUserState},
    hooks::current_user::use_current_user,
};

#[function_component]
pub fn Name() -> Html {
    let fallback = html! { <Fallback /> };

    html! {
        <Suspense {fallback}>
            <Content />
        </Suspense>
    }
}

#[function_component]
fn Content() -> HtmlResult {
    let context = use_context::<CurrentUserContext>().unwrap();
    let node_ref = use_node_ref();
    let current_user = use_current_user()?; // TODO: handle network error...

    let name = current_user
        .try_get_resources()
        .and_then(Resources::try_get_individual)
        .and_then(|r| r.try_get_attribute(|a| a.name.as_ref(), "name", "Name"))
        .map_or_else(|_| String::default(), String::clone);

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

    Ok(html! {
        <div class="name">
            <div>{ "welcome" }</div>
            <form onsubmit={onsubmit}>
                <div>{ "name" }</div>
                <div><input ref={node_ref} value={name}/></div>
                <div><button>{ "continue" }</button></div>
            </form>
        </div>
    })
}

#[function_component]
fn Fallback() -> Html {
    html! {
        <div> { "Loading..." }</div>
    }
}
