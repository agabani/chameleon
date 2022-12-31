use chameleon_protocol::jsonapi::Resources;
use yew::prelude::*;

use crate::{
    components::lobby_details::LobbyDetails,
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
    let lobby = use_lobby(&props.id)?;
    let host = use_lobby_host(&props.id)?;

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
       <LobbyDetails id={id} name={name} host={host} />
    })
}

#[function_component]
fn Fallback() -> Html {
    html! {
        <div>{ "Loading..." }</div>
    }
}
