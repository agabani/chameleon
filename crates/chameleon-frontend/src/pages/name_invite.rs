use chameleon_protocol::{attributes, jsonapi};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    components::lobby_invite_form::{LobbyLandingForm, OnsubmitEvent},
    contexts::{
        current_user::{CurrentUserContext, CurrentUserState},
        network::{NetworkContext, NetworkState},
    },
    hooks::{
        current_user::use_current_user,
        lobby::{use_lobby, use_lobby_host},
    },
};

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,
}

#[function_component]
pub fn NameInvite(props: &Props) -> Html {
    html! {
        <Suspense fallback={html! { <Fallback /> }}>
            <Content id={&props.id} />
        </Suspense>
    }
}

#[function_component]
fn Content(props: &Props) -> HtmlResult {
    // context
    let current_user_context = use_context::<CurrentUserContext>().unwrap();
    let network_context = use_context::<NetworkContext>().unwrap();

    // resources
    let current_user = use_current_user()?;
    let lobby = use_lobby(&props.id)?;
    let host = use_lobby_host(&props.id)?;

    // callback
    let onsubmit = {
        let current_user_id = current_user
            .try_get_field(|accessor| accessor.id.as_ref(), "id", "Id")
            .cloned()
            .ok();

        let lobby_id = props.id.clone();
        use_callback(
            move |event: OnsubmitEvent, _| {
                let current_user_id = current_user_id.clone();
                let current_user_context = current_user_context.clone();
                let lobby_id = lobby_id.clone();
                let network_context = network_context.clone();
                spawn_local(async move {
                    network_update_user(&network_context, &current_user_id, &event).await;
                    if network_join_lobby(&network_context, &lobby_id, &event).await {
                        current_user_context.dispatch(CurrentUserState {
                            authenticated: true,
                        });
                    }
                });
            },
            (),
        )
    };

    // presentation
    let current_user_name = present_user_name(&current_user);
    let host_user_name = present_user_name(&host);
    let lobby_name = present_lobby_name(&lobby);
    let lobby_require_passcode = present_lobby_require_passcode(&lobby);

    // layout
    Ok(html! {
        <div class="name-invite">
            <div>
                { "You're joining lobby '" } { lobby_name } {"'"}
            </div>
            <div>
                { "Hosted by '" } { host_user_name }  { "'" }
            </div>
            <LobbyLandingForm
                current_user_name={current_user_name.unwrap_or_default()}
                require_passcode={lobby_require_passcode}
                onsubmit={onsubmit} />
        </div>
    })
}

#[function_component]
fn Fallback() -> Html {
    html! {
        <div class="name-invite">
            { "Loading..." }
        </div>
    }
}

async fn network_join_lobby(
    network_context: &UseReducerHandle<NetworkState>,
    lobby_id: &AttrValue,
    event: &OnsubmitEvent,
) -> bool {
    let lobby_document = jsonapi::ResourcesDocument {
        data: Some(jsonapi::Resources::Individual(jsonapi::Resource {
            id: Some(lobby_id.to_string()),
            type_: Some("lobby".to_string()),
            attributes: Some(attributes::LobbyAttributes {
                name: None,
                passcode: event
                    .lobby_passcode
                    .as_ref()
                    .map(std::string::ToString::to_string),
                require_passcode: None,
            }),
            links: None,
            relationships: None,
        })),
        errors: None,
        links: None,
    };

    let response = network_context
        .action_lobby_join(lobby_id, &lobby_document)
        .await
        .expect("TODO: network request failed");

    if let Some(errors) = response.errors {
        gloo::console::error!(format!("{errors:?}"));
        false
    } else {
        true
    }
}

async fn network_update_user(
    network_context: &UseReducerHandle<NetworkState>,
    current_user_id: &Option<String>,
    event: &OnsubmitEvent,
) {
    let user_document = jsonapi::ResourcesDocument {
        data: Some(jsonapi::Resources::Individual(jsonapi::Resource {
            id: current_user_id.clone(),
            type_: Some("user".to_string()),
            attributes: Some(attributes::UserAttributes {
                name: Some(event.user_name.to_string()),
            }),
            links: None,
            relationships: None,
        })),
        errors: None,
        links: None,
    };

    let response = if let Some(current_user_id) = current_user_id {
        network_context
            .update_user(current_user_id, &user_document)
            .await
    } else {
        network_context.create_user(&user_document).await
    };

    response
        .expect("TODO: network request failed")
        .try_get_resources()
        .expect("TODO: user create or update failed");
}

fn present_lobby_name(
    lobby: &jsonapi::ResourcesDocument<attributes::LobbyAttributes>,
) -> AttrValue {
    lobby
        .try_get_attribute(|accessor| accessor.name.as_ref(), "name", "Name")
        .map(|attribute| attribute.clone().into())
        .expect("TODO: lobby name does not exist")
}

fn present_lobby_require_passcode(
    lobby: &jsonapi::ResourcesDocument<attributes::LobbyAttributes>,
) -> bool {
    lobby
        .try_get_attribute(
            |accessor| accessor.require_passcode.as_ref(),
            "require_passcode",
            "Require Passcode",
        )
        .map(|attribute| *attribute)
        .expect("TODO: expected")
}

fn present_user_name(
    user: &jsonapi::ResourcesDocument<attributes::UserAttributes>,
) -> Option<AttrValue> {
    user.try_get_attribute(|accessor| accessor.name.as_ref(), "name", "Name")
        .map(|attribute| attribute.clone().into())
        .ok()
}
