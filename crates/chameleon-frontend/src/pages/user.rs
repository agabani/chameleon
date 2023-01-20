use chameleon_protocol::{attributes, jsonapi};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    components::user_form::{OnsubmitEvent, UserForm},
    contexts::{
        current_user::{CurrentUserContext, CurrentUserState},
        network::{NetworkContext, NetworkState},
    },
    hooks::current_user::use_current_user,
};

#[function_component]
pub fn User() -> Html {
    let fallback = html! {<Fallback />};

    html! {
        <Suspense {fallback}>
            <Content />
        </Suspense>
    }
}

#[function_component]
fn Content() -> HtmlResult {
    let state = use_state(State::default);

    let current_user = use_context::<CurrentUserContext>().unwrap();
    let network = use_context::<NetworkContext>().unwrap();

    let user = use_current_user()?;

    let user_id = format_user_id(&user);
    let user_name = format_user_name(&user);

    let onsubmit = {
        let networking = state.networking;
        let state = state.clone();
        use_callback(
            move |event, (user_id, _)| {
                handle_onsubmit(&network, &current_user, &state, user_id, event);
            },
            (user_id, networking),
        )
    };

    Ok(html! {
        <div class="user">
            <UserForm name={user_name} onsubmit={onsubmit} disabled={state.networking} />
        </div>
    })
}

#[function_component]
fn Fallback() -> Html {
    html! {}
}

#[derive(Default)]
struct State {
    networking: bool,
}

fn format_user_id(
    user: &jsonapi::ResourcesDocument<attributes::UserAttributes>,
) -> Option<AttrValue> {
    user.try_get_field(|accessor| accessor.id.as_ref(), "id", "Id")
        .ok()
        .cloned()
        .map(Into::into)
}

fn format_user_name(
    user: &jsonapi::ResourcesDocument<attributes::UserAttributes>,
) -> Option<AttrValue> {
    user.try_get_attribute(|accessor| accessor.name.as_ref(), "name", "Name")
        .ok()
        .cloned()
        .map(Into::into)
}

fn handle_onsubmit(
    network: &UseReducerHandle<NetworkState>,
    current_user: &UseReducerHandle<CurrentUserState>,
    state: &UseStateHandle<State>,
    user_id: &Option<AttrValue>,
    event: OnsubmitEvent,
) {
    if state.networking {
        return;
    }

    state.set(State { networking: true });

    let current_user = current_user.clone();
    let network = network.clone();
    let state = state.clone();
    let user_id = user_id.clone();
    spawn_local(async move {
        let document = jsonapi::ResourcesDocument {
            data: Some(jsonapi::Resources::Individual(jsonapi::Resource {
                id: user_id.as_ref().map(ToString::to_string),
                type_: Some("user".to_string()),
                attributes: Some(attributes::UserAttributes {
                    name: Some(event.name.to_string()),
                }),
                links: None,
                relationships: None,
            })),
            errors: None,
            links: None,
        };

        let response = if let Some(user_id) = user_id {
            network.update_user(&user_id, &document).await
        } else {
            network.create_user(&document).await
        };

        let response = match response {
            Ok(response) => response,
            Err(errors) => {
                gloo::console::error!(format!("{errors:?}"));
                return;
            }
        };

        if let Some(errors) = response.errors {
            gloo::console::error!(format!("{errors:?}"));
            return;
        }

        current_user.dispatch(CurrentUserState {
            authenticated: true,
        });

        state.set(State { networking: false });
    });
}
