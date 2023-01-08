use chameleon_protocol::{attributes, jsonapi};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    app::Route,
    components::{
        lobby_host_form::{LobbyHostForm, OnsubmitEvent},
        navigation::Navigation,
    },
    contexts::network::{NetworkContext, NetworkState},
};

#[function_component]
pub fn Host() -> Html {
    let state = use_state(State::default);

    let navigator = use_navigator().unwrap();
    let network = use_context::<NetworkContext>().unwrap();

    let lobby_host_form_onsubmit = {
        let state = state.clone();
        use_callback(
            move |event, network| {
                handle_lobby_host_form_onsubmit(&navigator, network, &state, &event);
            },
            network,
        )
    };

    html! {
        <div class="host">
            <div class="host--grid-item-header">
                <Navigation />
            </div>
            <div class="host--grid-item-content">
                <LobbyHostForm disabled={state.networking} onsubmit={lobby_host_form_onsubmit} />
            </div>
        </div>
    }
}

#[derive(Default)]
struct State {
    networking: bool,
}

fn handle_lobby_host_form_onsubmit(
    navigator: &Navigator,
    network: &UseReducerHandle<NetworkState>,
    state: &UseStateHandle<State>,
    event: &OnsubmitEvent,
) {
    gloo::console::log!(format!(
        "{:?} {:?} {:?}",
        event.name, event.require_passcode, event.passcode
    ));
    action_create_lobby(navigator, network, state, event);
}

fn action_create_lobby(
    navigator: &Navigator,
    network: &UseReducerHandle<NetworkState>,
    state: &UseStateHandle<State>,
    event: &OnsubmitEvent,
) {
    state.set(State { networking: true });

    let document = jsonapi::ResourcesDocument {
        data: Some(jsonapi::Resources::Individual(jsonapi::Resource {
            id: None,
            type_: Some("lobby".to_string()),
            attributes: Some(attributes::LobbyAttributes {
                name: Some(event.name.to_string()),
                passcode: event.passcode.as_ref().map(ToString::to_string),
                require_passcode: Some(event.require_passcode),
            }),
            links: None,
            relationships: None,
        })),
        errors: None,
        links: None,
    };

    let navigator = navigator.clone();
    let network = network.clone();
    let state = state.clone();
    spawn_local(async move {
        let resource = match network.create_lobby(&document).await {
            Ok(resource) => resource,
            Err(error) => {
                state.set(State { networking: false });
                gloo::console::error!(error.to_string());
                return;
            }
        };

        if let Some(errors) = resource.errors {
            state.set(State { networking: false });
            gloo::console::error!(format!("{errors:?}"));
            return;
        }

        let id = resource
            .try_get_field(|a| a.id.as_ref(), "id", "Id")
            .expect("lobby to have id")
            .clone();

        navigator.push(&Route::Lobby { id });
    });
}
