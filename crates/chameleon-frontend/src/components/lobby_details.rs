use chameleon_protocol::jsonapi::Resources;
use yew::prelude::*;

use crate::hooks::lobby::{use_lobby, use_lobby_host};

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,
}

#[function_component]
pub fn LobbyDetails(props: &Props) -> Html {
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

    Ok(html! {
        <div class="lobby-details">
            <div>{ id }</div>
            <div>{ name }</div>
            <div>{ host }</div>
        </div>
    })
}

#[function_component]
fn Fallback() -> Html {
    html! {
        <div>{ "Loading..." }</div>
    }
}
