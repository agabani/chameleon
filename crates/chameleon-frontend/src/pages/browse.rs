use std::rc::Rc;

use chameleon_protocol::{attributes, jsonapi};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    app::Route,
    components::{
        infinite_scrolling::{InfiniteScrolling, OnscrollEvent},
        lobby_details::{LobbyDetails, OnsubmitEvent},
        lobby_list::LobbyList,
        lobby_list_item::LobbyListItem,
        navigation::Navigation,
    },
    contexts::network::{NetworkContext, NetworkState},
};

#[function_component]
pub fn Browse() -> Html {
    let state = use_reducer(State::default);
    let navigator = use_navigator().unwrap();
    let network = use_context::<NetworkContext>().unwrap();

    {
        let network = network.clone();
        let state = state.clone();
        let rendered = state.rendered;
        use_effect_with_deps(move |_| handle_first_render(&network, &state), rendered);
    }

    let lobby_details_onsubmit = {
        let network = network.clone();
        let state = state.clone();
        let lobby_id = state.lobby.as_ref().map(|s| s.id.clone());
        use_callback(
            move |event, id| {
                handle_lobby_details_onsubmit(&navigator, &network, id.as_ref().unwrap(), &event);
            },
            lobby_id,
        )
    };

    let lobby_list_item_onclick = {
        let network = network.clone();
        let state = state.clone();
        use_callback(
            move |id, _| handle_lobby_list_item_onclick(&network, &state, &id),
            (),
        )
    };

    let infinite_scrolling_onclick = {
        let network = network.clone();
        let state = state.clone();
        let next = state.next_lobby_link.clone();
        use_callback(
            move |_, _| handle_infinite_scrolling_onclick(&network, &state),
            next,
        )
    };

    let infinite_scrolling_onscroll = {
        let network = network;
        let state = state.clone();
        let next = state.next_lobby_link.clone();
        use_callback(
            move |event, _| handle_infinite_scrolling_onscroll(&network, &state, &event),
            next,
        )
    };

    html! {
        <div class="browse">
            <div class="browse--grid-item-header">
                <Navigation />
            </div>
            <div class="browse--grid-item-browse">
                <InfiniteScrolling
                    onclick={infinite_scrolling_onclick}
                    onscroll={infinite_scrolling_onscroll}>
                    <LobbyList>
                    {
                        state.lobbies.iter().map(|item| {
                            html! {
                                <LobbyListItem
                                    key={item.id.as_str()}
                                    id={&item.id}
                                    name={&item.name}
                                    require_passcode={item.require_passcode}
                                    onclick={lobby_list_item_onclick.clone()} />
                            }
                        }).collect::<Html>()
                    }
                    </LobbyList>
                </InfiniteScrolling>
            </div>
            <div class="browse--grid-item-details">
                if let Some(item) = &state.lobby {
                    <LobbyDetails
                        key={item.id.as_str()}
                        id={&item.id}
                        host_name={&item.host_name}
                        name={&item.name}
                        require_passcode={item.require_passcode}
                        onsubmit={lobby_details_onsubmit} />
                }
            </div>
        </div>
    }
}

struct State {
    lobbies: Vec<StateLobby>,
    lobby: Option<StateLobbyDetails>,
    networking: bool,
    next_lobby_link: Option<AttrValue>,
    rendered: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            lobbies: Vec::new(),
            lobby: None,
            networking: false,
            next_lobby_link: None,
            rendered: true,
        }
    }
}

#[derive(Clone)]
struct StateLobby {
    id: AttrValue,
    name: AttrValue,
    require_passcode: bool,
}

#[derive(Clone)]
struct StateLobbyDetails {
    id: AttrValue,
    host_name: AttrValue,
    name: AttrValue,
    require_passcode: bool,
}

impl Reducible for State {
    type Action = Action;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        match action {
            Action::Lobbies((new_lobbies, next_link)) => {
                let mut lobbies = self.lobbies.clone();
                lobbies.extend(new_lobbies);

                Rc::new(Self {
                    lobbies,
                    lobby: self.lobby.clone(),
                    networking: self.networking,
                    next_lobby_link: Some(next_link),
                    rendered: true,
                })
            }
            Action::LobbyDetails(lobby_details) => Rc::new(Self {
                lobbies: self.lobbies.clone(),
                lobby: Some(lobby_details),
                networking: self.networking,
                next_lobby_link: self.next_lobby_link.clone(),
                rendered: self.rendered,
            }),
            Action::Networking(networking) => Rc::new(Self {
                lobbies: self.lobbies.clone(),
                lobby: self.lobby.clone(),
                networking,
                next_lobby_link: self.next_lobby_link.clone(),
                rendered: self.rendered,
            }),
            Action::Rendered => Rc::new(Self {
                lobbies: self.lobbies.clone(),
                lobby: self.lobby.clone(),
                networking: self.networking,
                next_lobby_link: self.next_lobby_link.clone(),
                rendered: true,
            }),
        }
    }
}

enum Action {
    Lobbies((Vec<StateLobby>, AttrValue)),
    LobbyDetails(StateLobbyDetails),
    Networking(bool),
    Rendered,
}

fn handle_first_render(network: &UseReducerHandle<NetworkState>, state: &UseReducerHandle<State>) {
    if !state.rendered {
        return;
    }

    state.dispatch(Action::Rendered);

    fetch_lobbies(network, state);
}

fn handle_lobby_details_onsubmit(
    navigator: &Navigator,
    network: &UseReducerHandle<NetworkState>,
    id: &AttrValue,
    event: &OnsubmitEvent,
) {
    action_join_lobby(navigator, network, id, event);
}

fn handle_lobby_list_item_onclick(
    network: &UseReducerHandle<NetworkState>,
    state: &UseReducerHandle<State>,
    id: &AttrValue,
) {
    fetch_lobby_details(network, state, id);
}

fn handle_infinite_scrolling_onclick(
    network: &UseReducerHandle<NetworkState>,
    state: &UseReducerHandle<State>,
) {
    fetch_lobbies(network, state);
}

fn handle_infinite_scrolling_onscroll(
    network: &UseReducerHandle<NetworkState>,
    state: &UseReducerHandle<State>,
    event: &OnscrollEvent,
) {
    if event.scroll_height - event.client_height - event.scroll_top > 100 {
        return;
    }

    fetch_lobbies(network, state);
}

fn action_join_lobby(
    navigator: &Navigator,
    network: &UseReducerHandle<NetworkState>,
    id: &AttrValue,
    event: &OnsubmitEvent,
) {
    let id = id.clone();
    let passcode = event.passcode.as_ref().map(ToString::to_string);
    let navigator = navigator.clone();
    let network = network.clone();
    spawn_local(async move {
        let document = jsonapi::ResourcesDocument {
            data: Some(jsonapi::Resources::Individual(jsonapi::Resource {
                id: None,
                type_: Some("lobby".to_string()),
                attributes: Some(attributes::LobbyAttributes {
                    name: None,
                    passcode,
                    require_passcode: None,
                }),
                links: None,
                relationships: None,
            })),
            errors: None,
            links: None,
        };

        let response = match network.action_lobby_join(id.as_str(), &document).await {
            Ok(resource) => resource,
            Err(error) => {
                gloo::console::error!(error.to_string());
                return;
            }
        };

        if let Some(errors) = response.errors {
            gloo::console::error!(format!("{errors:?}"));
            return;
        }

        navigator.push(&Route::Lobby { id: id.to_string() });
    });
}

fn fetch_lobbies(network: &UseReducerHandle<NetworkState>, state: &UseReducerHandle<State>) {
    if state.networking {
        return;
    }

    state.dispatch(Action::Networking(true));

    let network = network.clone();
    let state = state.clone();
    spawn_local(async move {
        let next = state.next_lobby_link.as_ref().map(ToString::to_string);

        let response = match network.query_lobby(next).await {
            Ok(response) => response,
            Err(error) => {
                state.dispatch(Action::Networking(false));
                gloo::console::error!(error.to_string());
                return;
            }
        };

        if let Some(errors) = response.errors {
            state.dispatch(Action::Networking(false));
            gloo::console::error!(format!("{errors:?}"));
            return;
        }

        let next_link: AttrValue = response
            .try_get_link("next", "Next")
            .expect("next link to be present")
            .to_string()
            .into();

        let lobbies = response
            .try_get_collection_resources()
            .expect("collection to be present")
            .iter()
            .map(|resource| {
                let id = resource
                    .try_get_field(|a| a.id.as_ref(), "id", "Id")
                    .expect("Lobby to have ID")
                    .to_string()
                    .into();

                let name = resource
                    .try_get_attribute(|a| a.name.as_ref(), "name", "Name")
                    .map_or_else(
                        |_| "<< UNNAMED >>".to_string().into(),
                        |f| f.to_string().into(),
                    );

                let require_passcode = resource
                    .try_get_attribute(
                        |a| a.require_passcode.as_ref(),
                        "require_passcode",
                        "require_passcode",
                    )
                    .map_or(false, |f| *f);

                StateLobby {
                    id,
                    name,
                    require_passcode,
                }
            })
            .collect();

        state.dispatch(Action::Lobbies((lobbies, next_link)));

        state.dispatch(Action::Networking(false));
    });
}

fn fetch_lobby_details(
    network: &UseReducerHandle<NetworkState>,
    state: &UseReducerHandle<State>,
    id: &AttrValue,
) {
    let id = id.clone();
    let network = network.clone();
    let state = state.clone();
    spawn_local(async move {
        let id = id;

        let lobby_resource = match network.get_lobby(id.as_str()).await {
            Ok(resource) => resource,
            Err(error) => {
                state.dispatch(Action::Networking(false));
                gloo::console::error!(error.to_string());
                return;
            }
        };

        if let Some(errors) = lobby_resource.errors {
            state.dispatch(Action::Networking(false));
            gloo::console::error!(format!("{errors:?}"));
            return;
        }

        let host_resource = match network.get_lobby_host(id.as_str()).await {
            Ok(resource) => resource,
            Err(error) => {
                state.dispatch(Action::Networking(false));
                gloo::console::error!(error.to_string());
                return;
            }
        };

        if let Some(errors) = host_resource.errors {
            state.dispatch(Action::Networking(false));
            gloo::console::error!(format!("{errors:?}"));
            return;
        }

        let id: AttrValue = lobby_resource
            .try_get_field(|a| a.id.as_ref(), "id", "Id")
            .expect("Lobby to have ID")
            .to_string()
            .into();

        let host_name: AttrValue = host_resource
            .try_get_attribute(|a| a.name.as_ref(), "name", "Name")
            .map_or_else(
                |_| "<< UNNAMED >>".to_string().into(),
                |f| f.to_string().into(),
            );

        let name: AttrValue = lobby_resource
            .try_get_attribute(|a| a.name.as_ref(), "name", "Name")
            .map_or_else(
                |_| "<< UNNAMED >>".to_string().into(),
                |f| f.to_string().into(),
            );

        let require_passcode = lobby_resource
            .try_get_attribute(
                |a| a.require_passcode.as_ref(),
                "require_passcode",
                "require_passcode",
            )
            .map_or(false, |f| *f);

        state.dispatch(Action::LobbyDetails(StateLobbyDetails {
            id,
            host_name,
            name,
            require_passcode,
        }));
    });
}
