use chameleon_protocol::{
    attributes::UserAttributes,
    jsonapi::{Resources, ResourcesDocument},
};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::{
    contexts::{
        current_user::{CurrentUserContext, CurrentUserState},
        network::NetworkContext,
    },
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
    let current_user = use_current_user()?;
    let current_user_context = use_context::<CurrentUserContext>().unwrap();
    let network_context = use_context::<NetworkContext>().unwrap();
    let node_ref = use_node_ref();

    let id = current_user
        .try_get_resources()
        .and_then(Resources::try_get_individual)
        .and_then(|r| r.try_get_field(|r| r.id.as_ref(), "id", "Id"))
        .ok()
        .cloned();

    let name = current_user
        .try_get_resources()
        .and_then(Resources::try_get_individual)
        .and_then(|r| r.try_get_attribute(|r| r.name.as_ref(), "name", "Name"))
        .ok()
        .cloned();

    let onsubmit = {
        let node_ref = node_ref.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();

            let id = id.clone();
            let current_user_context = current_user_context.clone();
            let network_context = network_context.clone();

            let name = node_ref.cast::<HtmlInputElement>().unwrap().value();

            let document = ResourcesDocument {
                data: Some(Resources::Individual(
                    chameleon_protocol::jsonapi::Resource {
                        id: id.clone(),
                        type_: Some("user".to_string()),
                        attributes: Some(UserAttributes { name: Some(name) }),
                        links: None,
                        relationships: None,
                    },
                )),
                errors: None,
                links: None,
            };

            spawn_local(async move {
                let response = if let Some(id) = id {
                    network_context.update_user(&id, &document).await
                } else {
                    network_context.create_user(&document).await
                };

                response
                    .expect("TODO: network request failed")
                    .try_get_resources()
                    .expect("TODO: user create or update failed");

                current_user_context.dispatch(CurrentUserState {
                    authenticated: true,
                });
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
