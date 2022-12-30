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

#[derive(PartialEq, Properties)]
pub struct Props {
    pub onclick: Callback<String>,
}

#[function_component]
pub fn LobbyListInfiniteScrolling(props: &Props) -> Html {
    let network = use_context::<NetworkContext>().unwrap();
    let state = use_reducer(State::default);

    let onclick = {
        let network = network.clone();
        let state = state.clone();
        Callback::from(move |_| {
            let network = network.clone();
            let state = state.clone();
            fetch(network, state);
        })
    };

    {
        let state = state.clone();
        use_effect(|| initial_fetch(network, state));
    };

    html! {
        <div class="lobby-list-infinite-scrolling">
            <LobbyList>
                { state.lobbies.iter().map(|lobby| {
                    let id = lobby.try_get_field(|r| r.id.as_ref(), "id", "Id").unwrap();
                    let name = lobby.try_get_attribute(|a| a.name.as_ref(), "name", "Name").unwrap();
                    let onclick = {
                        let id = id.clone();
                        let onclick = props.onclick.clone();
                        move |_| onclick.clone().emit(id.clone())
                    };
                    html! {
                        <LobbyListItem id={id.clone()} name={name.clone()} onclick={onclick} />
                    }
                }).collect::<Html>() }
            </LobbyList>
            <button onclick={onclick}>{ "Load More" }</button>
        </div>
    }
}

#[derive(Default)]
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

fn fetch(network: NetworkContext, state: UseReducerHandle<State>) {
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
}

fn initial_fetch(network: NetworkContext, state: UseReducerHandle<State>) {
    if state.document.is_none() {
        fetch(network, state);
    }
}
