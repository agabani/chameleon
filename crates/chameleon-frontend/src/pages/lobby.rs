use chameleon_protocol::jsonapi::Resources;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    containers::{
        lobby_chat_list_container::LobbyChatListContainer,
        lobby_member_list_container::LobbyMemberListContainer,
    },
    contexts::network::NetworkContext,
    hooks::lobby::{use_lobby, use_lobby_host},
};

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,
}

#[function_component]
pub fn Lobby(props: &Props) -> Html {
    html! {
        <Suspense fallback={html! { <Fallback /> }}>
            <Content id={props.id.clone()} key={props.id.as_ref()} />
        </Suspense>
    }
}

#[function_component]
fn Content(props: &Props) -> HtmlResult {
    let network = use_context::<NetworkContext>().unwrap();
    let lobby = use_lobby(&props.id)?;
    let host = use_lobby_host(&props.id)?;

    let id = lobby
        .try_get_resources()
        .and_then(Resources::try_get_individual)
        .and_then(|r| r.try_get_field(|a| a.id.as_ref(), "id", "Id"))
        .unwrap();

    let name = lobby
        .try_get_resources()
        .and_then(Resources::try_get_individual)
        .and_then(|r| r.try_get_attribute(|a| a.name.as_ref(), "name", "Name"))
        .unwrap();

    let host = host
        .try_get_resources()
        .and_then(Resources::try_get_individual)
        .and_then(|r| r.try_get_attribute(|a| a.name.as_ref(), "name", "Name"))
        .unwrap();

    let onclick = {
        use_callback(
            move |_, id| {
                let network = network.clone();
                let id = id.clone();
                spawn_local(async move {
                    network.action_lobby_leave(&id).await.unwrap();
                });
            },
            props.id.clone(),
        )
    };

    Ok(html! {
        <div class="lobby">
            <div>{ id }</div>
            <div>{ name }</div>
            <div>{ host }</div>
            <div><button onclick={onclick}>{ "leave" }</button></div>
            <div>{ "=== members ===" }</div>
            <LobbyMemberListContainer id={props.id.clone()} />
            <div>{ "=== chat ===" }</div>
            <LobbyChatListContainer id={props.id.clone()} />
        </div>
    })
}

#[function_component]
fn Fallback() -> Html {
    html! {
        <div>{ "Loading..." }</div>
    }
}
