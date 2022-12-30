use std::rc::Rc;

use chameleon_protocol::{
    attributes::LobbyAttributes,
    jsonapi::{Resource, Resources, ResourcesDocument},
};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    components::{lobby_list::LobbyList, lobby_list_item::LobbyListItem},
    contexts::network::NetworkContext,
};

#[function_component]
pub fn LobbyListInfiniteScrolling() -> Html {
    let network = use_context::<NetworkContext>().unwrap();
    let state = use_reducer(|| State {
        document: None,
        lobbies: Vec::new(),
    });

    let onclick = {
        let state = state.clone();
        Callback::from(move |_| {
            let network = network.clone();
            let state = state.clone();

            let next = state
                .document
                .as_ref()
                .and_then(|document| document.try_get_link("next", "Next").ok().cloned());

            spawn_local(async move {
                let document = network
                    .query_lobby(next)
                    .await
                    .unwrap_or_else(|_| ResourcesDocument::internal_server_error());
                state.dispatch(document);
            });
        })
    };

    html! {
        <div class="lobby-list-infinite-scrolling">
            <LobbyList>
                { state.lobbies.iter().map(|lobby|{
                    let id = lobby.try_get_field(|r| r.id.as_ref(), "id", "Id").unwrap();
                    let name = lobby.try_get_attribute(|a| a.name.as_ref(), "name", "Name").unwrap();
                    html! {
                        <LobbyListItem id={id.clone()} name={name.clone()} />
                    }
                }).collect::<Html>() }
            </LobbyList>
            <button onclick={onclick}>{ "Load More" }</button>
        </div>
    }
}

struct State {
    document: Option<ResourcesDocument<LobbyAttributes>>,
    lobbies: Vec<Resource<LobbyAttributes>>,
}

impl Reducible for State {
    type Action = ResourcesDocument<LobbyAttributes>;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut lobbies = self.lobbies.clone();
        lobbies.extend(
            action
                .try_get_resources()
                .and_then(Resources::try_get_collection)
                .cloned()
                .unwrap_or_default(),
        );

        Rc::new(Self {
            document: Some(action),
            lobbies,
        })
    }
}
