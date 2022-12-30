use chameleon_protocol::{
    attributes::LobbyAttributes,
    jsonapi::{Resource, Resources, ResourcesDocument},
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::use_navigator;

use crate::{
    app::Route, components::lobby_creation_form::LobbyCreationForm,
    contexts::network::NetworkContext,
};

#[function_component]
pub fn Host() -> Html {
    let disabled = use_state(|| false);
    let navigator = use_navigator().unwrap();
    let network = use_context::<NetworkContext>().unwrap();
    let onsubmit = use_callback(
        |name: AttrValue, (network, navigator, disabled)| {
            disabled.set(true);

            let disabled = disabled.clone();
            let network = network.clone();
            let navigator = navigator.clone();
            spawn_local(async move {
                let document = network
                    .create_lobby(&ResourcesDocument {
                        data: Some(Resources::Individual(Resource {
                            id: None,
                            type_: Some("lobby".to_string()),
                            attributes: Some(LobbyAttributes {
                                name: Some(name.to_string()),
                            }),
                            links: None,
                            relationships: None,
                        })),
                        errors: None,
                        links: None,
                    })
                    .await
                    .unwrap_or_else(|_| ResourcesDocument::internal_server_error());

                let id = document
                    .try_get_resources()
                    .and_then(Resources::try_get_individual)
                    .and_then(|r| r.try_get_field(|a| a.id.as_ref(), "id", "Id"))
                    .cloned()
                    .ok();

                if let Some(id) = id {
                    navigator.push(&Route::Lobby { id });
                } else {
                    disabled.set(false);
                }
            });
        },
        (network, navigator, disabled.clone()),
    );

    html! {
        <div class="host">
            <div>{ "Host" }</div>
            <LobbyCreationForm disabled={*disabled} onsubmit={onsubmit} />
        </div>
    }
}
