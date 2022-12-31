use chameleon_protocol::jsonapi::{ResourceIdentifiersDocument, Resources};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    app::Route,
    components::lobby_details::LobbyDetails,
    contexts::network::NetworkContext,
    hooks::lobby::{use_lobby, use_lobby_host},
};

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,
}

#[function_component]
pub fn LobbyDetailsContainer(props: &Props) -> Html {
    html! {
        <Suspense fallback={html! { <Fallback /> }}>
            <Content id={props.id.clone()} key={props.id.as_ref()} />
        </Suspense>
    }
}

#[function_component]
fn Content(props: &Props) -> HtmlResult {
    let network = use_context::<NetworkContext>().unwrap();
    let navigator = use_navigator().unwrap();
    let lobby = use_lobby(&props.id)?;
    let host = use_lobby_host(&props.id)?;
    let onclick = {
        use_callback(
            move |_: MouseEvent, id| {
                let id = id.clone();
                let navigator = navigator.clone();
                let network = network.clone();
                spawn_local(async move {
                    let document = network
                        .action_lobby_join(&id)
                        .await
                        .unwrap_or_else(|_| ResourceIdentifiersDocument::internal_server_error());

                    if let Some(errors) = document.errors {
                        gloo::console::error!(format!("{errors:?}"));
                    } else {
                        navigator.push(&Route::Lobby { id: id.to_string() });
                    }
                });
            },
            props.id.clone(),
        )
    };

    let id = lobby
        .try_get_resources()
        .and_then(Resources::try_get_individual)
        .and_then(|r| r.try_get_field(|a| a.id.as_ref(), "id", "Id"))
        .cloned()
        .unwrap();

    let name = lobby
        .try_get_resources()
        .and_then(Resources::try_get_individual)
        .and_then(|r| r.try_get_attribute(|a| a.name.as_ref(), "name", "Name"))
        .cloned()
        .unwrap();

    let host = host
        .try_get_resources()
        .and_then(Resources::try_get_individual)
        .and_then(|r| r.try_get_attribute(|a| a.name.as_ref(), "name", "Name"))
        .cloned()
        .unwrap();

    Ok(html! {
       <LobbyDetails id={id} name={name} host={host} onclick={onclick} />
    })
}

#[function_component]
fn Fallback() -> Html {
    html! {
        <div>{ "Loading..." }</div>
    }
}
