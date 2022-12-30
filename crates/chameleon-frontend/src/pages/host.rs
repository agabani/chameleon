use chameleon_protocol::{
    attributes::LobbyAttributes,
    jsonapi::{Resource, Resources, ResourcesDocument},
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    components::lobby_creation_form::LobbyCreationForm, contexts::network::NetworkContext,
};

#[function_component]
pub fn Host() -> Html {
    let disabled = use_state(|| false);
    let network = use_context::<NetworkContext>().unwrap();
    let onsubmit = use_callback(
        |name: AttrValue, (network, disabled)| {
            disabled.set(true);

            let disabled = disabled.clone();
            let network = network.clone();
            spawn_local(async move {
                let _result = network
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
                    .await;

                disabled.set(false);
            });
        },
        (network, disabled.clone()),
    );

    html! {
        <div class="host">
            <div>{ "Host" }</div>
            <LobbyCreationForm disabled={*disabled} onsubmit={onsubmit} />
        </div>
    }
}
