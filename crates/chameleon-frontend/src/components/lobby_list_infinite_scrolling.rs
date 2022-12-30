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
        let state = state.clone();
        move |_| {
            state.dispatch(Action::Clicked);

            let network = network.clone();

            let next = state
                .document
                .as_ref()
                .and_then(|d| d.try_get_link("next", "Next").ok())
                .cloned();

            let state = state.clone();
            spawn_local(async move {
                let document = network
                    .query_lobby(next)
                    .await
                    .unwrap_or_else(|_| ResourcesDocument::internal_server_error());

                state.dispatch(Action::Document(Box::new(document)));
            });
        }
    };

    html! {
        <div class="lobby-list-infinite-scrolling">
            <LobbyList>
            {
                state.resources.iter().map(|resource| {
                    let id = resource
                        .try_get_field(|a| a.id.as_ref(), "id", "Id")
                        .cloned()
                        .unwrap();

                    let name = resource
                        .try_get_attribute(|a| a.name.as_ref(), "name", "Name")
                        .cloned()
                        .unwrap();

                    let onclick = {
                        let id = id.clone();
                        let onclick = props.onclick.clone();
                        move |_| {onclick.emit(id.clone())}
                    };

                    html! { <LobbyListItem id={id.clone()} name={name} onclick={onclick} key={id} /> }
                }).collect::<Html>()
            }
            </LobbyList>
            <button disabled={state.disabled} onclick={onclick}>{ "Load More" }</button>
        </div>
    }
}

#[derive(Default)]
struct State {
    disabled: bool,
    document: Option<ResourcesDocument<LobbyAttributes>>,
    resources: Vec<Resource<LobbyAttributes>>,
}

impl Reducible for State {
    type Action = Action;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        match action {
            Action::Clicked => Rc::new(Self {
                disabled: true,
                document: self.document.clone(),
                resources: self.resources.clone(),
            }),
            Action::Document(document) => {
                let mut resources = self.resources.clone();
                resources.extend(
                    document
                        .try_get_resources()
                        .and_then(Resources::try_get_collection)
                        .cloned()
                        .unwrap_or_default(),
                );

                Rc::new(Self {
                    disabled: false,
                    document: Some(*document),
                    resources,
                })
            }
        }
    }
}

enum Action {
    Clicked,
    Document(Box<ResourcesDocument<LobbyAttributes>>),
}
