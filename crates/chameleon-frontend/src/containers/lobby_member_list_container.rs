use chameleon_protocol::jsonapi::Resources;
use yew::prelude::*;

use crate::{
    components::{lobby_member_list::LobbyMemberList, lobby_member_list_item::LobbyMemberListItem},
    hooks::lobby::use_lobby_members,
};

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,
}

#[function_component]
pub fn LobbyMemberListContainer(props: &Props) -> Html {
    html! {
        <Suspense fallback={html! { <Fallback /> }}>
            <Content id={props.id.clone()} key={props.id.as_ref()} />
        </Suspense>
    }
}

#[function_component]
fn Content(props: &Props) -> HtmlResult {
    let document = use_lobby_members(&props.id, None)?;
    let members = use_state(|| {
        document
            .try_get_resources()
            .and_then(Resources::try_get_collection)
            .cloned()
            .unwrap()
    });

    Ok(html! {
        <LobbyMemberList>
            { members.iter().map(|member| {
                let id = member.try_get_field(|a| a.id.as_ref(), "id", "Id").unwrap();
                let name = member.try_get_attribute(|a| a.name.as_ref(), "name", "Name").unwrap();

                html! {
                    <LobbyMemberListItem id={id.clone()} name={name.clone()} key={id.clone()} />
                }
            }).collect::<Html>()}
        </LobbyMemberList>
    })
}

#[function_component]
fn Fallback() -> Html {
    html! {}
}
