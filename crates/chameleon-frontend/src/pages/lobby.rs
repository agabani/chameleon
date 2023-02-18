use std::{collections::HashMap, rc::Rc};

use chameleon_protocol::{attributes, frames, jsonapi, jsonrpc};
use futures::{channel::mpsc::Sender, SinkExt, StreamExt};
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::prelude::{use_navigator, Navigator};

use crate::{
    app::Route,
    components::{
        lobby_chat_input::LobbyChatInput, lobby_chat_list::LobbyChatList,
        lobby_chat_list_item::LobbyChatListItem, lobby_member_list::LobbyMemberList,
        lobby_member_list_item::LobbyMemberListItem,
    },
    contexts::network::{NetworkContext, NetworkState},
};

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,
}

#[function_component]
pub fn Lobby(props: &Props) -> Html {
    let navigator = use_navigator().unwrap();
    let network = use_context::<NetworkContext>().unwrap();
    let state = use_reducer(State::default);

    let onclick = {
        let id = props.id.clone();
        let network = network.clone();
        use_callback(move |_, _| action_leave(&network, &navigator, &id), ())
    };

    let onsubmit = {
        let authenticated = state.authenticated;

        let id = props.id.clone();
        let network = network.clone();
        let state = state.clone();
        use_callback(
            move |content: AttrValue, _| {
                action_chat_message(&network, &state, &id, &content);
            },
            authenticated,
        )
    };

    load_data(&network, &state, props);
    web_socket(network, state.clone(), props);

    html! {
        <div class="lobby">
            <div class="lobby--grid-item-settings">
                <div>{ "=== lobby ===" }</div>
                <div>{ "name: "} { present_lobby_name(&state) }</div>
                <div><button onclick={onclick}>{ "leave" }</button></div>
            </div>
            <div class="lobby--grid-item-members">
                <div>{ "=== lobby members ===" }</div>
                <LobbyMemberList>
                {
                    present_members(&state).iter().map(|(id, name)| html! {
                        <LobbyMemberListItem key={id.as_str()} id={id} name={name} />
                    }).collect::<Html>()
                }
                </LobbyMemberList>
            </div>
            <div class="lobby--grid-item-chat">
                <LobbyChatList>
                {
                    present_messages(&state).iter().map(|(key, _, name, content)| html! {
                        <LobbyChatListItem key={key.to_string()} name={name} message={content} />
                    }).collect::<Html>()
                }
                </LobbyChatList>
            </div>
            <div class="lobby--grid-item-chat-input">
                <LobbyChatInput disabled={!state.authenticated} onsubmit={onsubmit} />
            </div>
        </div>
    }
}

#[derive(Default)]
struct State {
    authenticated: bool,

    lobby: Option<jsonapi::Resource<attributes::LobbyAttributes>>,

    sender: Option<Sender<String>>,

    // members: user_id, name
    members: HashMap<AttrValue, AttrValue>,

    // messages: key, user_id, content
    messages: Vec<(Uuid, AttrValue, AttrValue)>,

    status: Status,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum Status {
    #[default]
    Requested,
    Processing,
    Completed,
}

impl Reducible for State {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Action::Authenticated => Rc::new(Self {
                authenticated: true,
                lobby: self.lobby.clone(),
                members: self.members.clone(),
                messages: self.messages.clone(),
                sender: self.sender.clone(),
                status: self.status,
            }),
            Action::ChatMessage(id, content) => {
                let mut messages = self.messages.clone();
                messages.push((Uuid::new_v4(), id, content));
                Rc::new(Self {
                    authenticated: self.authenticated,
                    lobby: self.lobby.clone(),
                    members: self.members.clone(),
                    messages,
                    sender: self.sender.clone(),
                    status: self.status,
                })
            }
            Action::Connected(sender) => Rc::new(Self {
                authenticated: self.authenticated,
                lobby: self.lobby.clone(),
                members: self.members.clone(),
                messages: self.messages.clone(),
                sender: Some(sender),
                status: self.status,
            }),
            Action::Disconnected => Rc::new(Self {
                authenticated: false,
                lobby: self.lobby.clone(),
                members: self.members.clone(),
                messages: self.messages.clone(),
                sender: None,
                status: self.status,
            }),
            Action::UserJoined(_) | Action::UserLeft(_) => self,
            Action::Status(status) => Rc::new(Self {
                authenticated: self.authenticated,
                lobby: self.lobby.clone(),
                members: self.members.clone(),
                messages: self.messages.clone(),
                sender: self.sender.clone(),
                status,
            }),
            Action::LoadedLobby(lobby) => Rc::new(Self {
                authenticated: self.authenticated,
                lobby,
                members: self.members.clone(),
                messages: self.messages.clone(),
                sender: self.sender.clone(),
                status: self.status,
            }),
            Action::LoadedMembers(members) => {
                let members = members
                    .map(|members| {
                        members
                            .into_iter()
                            .collect::<HashMap<AttrValue, AttrValue>>()
                    })
                    .unwrap_or_default();
                Rc::new(Self {
                    authenticated: self.authenticated,
                    lobby: self.lobby.clone(),
                    members,
                    messages: self.messages.clone(),
                    sender: self.sender.clone(),
                    status: self.status,
                })
            }
        }
    }
}

enum Action {
    Authenticated,
    // user id, content
    ChatMessage(AttrValue, AttrValue),
    Connected(Sender<String>),
    UserJoined(AttrValue),
    UserLeft(AttrValue),
    Disconnected,
    Status(Status),
    LoadedLobby(Option<jsonapi::Resource<attributes::LobbyAttributes>>),
    LoadedMembers(Option<Vec<(AttrValue, AttrValue)>>),
}

fn action_chat_message(
    network: &UseReducerHandle<NetworkState>,
    state: &UseReducerHandle<State>,
    id: &AttrValue,
    content: &AttrValue,
) {
    if !state.authenticated {
        return;
    }

    let document = jsonapi::ResourcesDocument {
        data: Some(jsonapi::Resources::Individual(jsonapi::Resource {
            id: None,
            type_: Some("chat_message".to_string()),
            attributes: Some(attributes::ChatMessageAttributes {
                message: Some(content.to_string()),
            }),
            links: None,
            relationships: None,
        })),
        errors: None,
        links: None,
    };

    let id = id.clone();
    let network = network.clone();
    spawn_local(async move {
        network
            .action_lobby_chat_message(&id, &document)
            .await
            .unwrap();
    });
}

fn action_leave(network: &UseReducerHandle<NetworkState>, navigator: &Navigator, id: &AttrValue) {
    let id = id.clone();
    let navigator = navigator.clone();
    let network = network.clone();
    spawn_local(async move {
        network.action_lobby_leave(&id).await.unwrap();
        navigator.push(&Route::Browse);
    });
}

fn load_data(
    network: &UseReducerHandle<NetworkState>,
    state: &UseReducerHandle<State>,
    props: &Props,
) {
    if state.status != Status::Requested {
        return;
    }

    state.dispatch(Action::Status(Status::Processing));

    let id = props.id.clone();
    let network = network.clone();
    let state = state.clone();
    spawn_local(async move {
        let lobby = network
            .get_lobby(&id)
            .await
            .unwrap_or_else(|_| jsonapi::ResourcesDocument::internal_server_error());

        let lobby = lobby
            .try_get_resources()
            .and_then(jsonapi::Resources::try_get_individual)
            .ok()
            .cloned();

        state.dispatch(Action::LoadedLobby(lobby));

        let members = network
            .get_lobby_members(&id, None)
            .await
            .unwrap_or_else(|_| jsonapi::ResourcesDocument::internal_server_error());

        let members = members
            .try_get_resources()
            .and_then(jsonapi::Resources::try_get_collection)
            .ok()
            .map(|members| {
                members
                    .iter()
                    .map(|member| {
                        let id = member.try_get_field(|a| a.id.as_ref(), "id", "Id").unwrap();

                        let name = member
                            .try_get_attribute(|a| a.name.as_ref(), "name", "Name")
                            .unwrap();

                        (id.to_string().into(), name.to_string().into())
                    })
                    .collect()
            });

        state.dispatch(Action::LoadedMembers(members));

        state.dispatch(Action::Status(Status::Completed));
    });
}

fn present_lobby_name(state: &State) -> AttrValue {
    state
        .lobby
        .as_ref()
        .and_then(|lobby| {
            lobby
                .try_get_attribute(|a| a.name.as_ref(), "name", "Name")
                .ok()
                .map(|v| v.to_string().into())
        })
        .unwrap_or_else(|| String::new().into())
}

/// presents members: user id, user name
fn present_members(state: &State) -> Vec<(AttrValue, AttrValue)> {
    let mut members = state
        .members
        .iter()
        .map(|(a, b)| (a.clone(), b.clone()))
        .collect::<Vec<_>>();
    members.sort_by(|a, b| a.1.cmp(&b.1));
    members
}

/// presents messages: key, user id, user name, message
fn present_messages(state: &State) -> Vec<(Uuid, AttrValue, AttrValue, AttrValue)> {
    state
        .messages
        .iter()
        .cloned()
        .map(|(key, id, content)| {
            let name = state
                .members
                .get(&id)
                .cloned()
                .unwrap_or_else(|| "???".to_string().into());
            (key, id, name, content)
        })
        .collect()
}

fn web_socket(
    network: UseReducerHandle<NetworkState>,
    state: UseReducerHandle<State>,
    props: &Props,
) {
    if state.sender.is_some() {
        return;
    }

    let web_socket = network
        .subscribe_lobby(&props.id)
        .expect("TODO: Failed to open web socket connection to lobby");

    let (mut sender, mut receiver) = network.web_socket_to_channels(web_socket);

    state.dispatch(Action::Connected(sender.clone()));

    let props = Props {
        id: props.id.clone(),
    };
    spawn_local(async move {
        let frame = frames::LobbyFrame::new_request(
            None,
            frames::LobbyRequest::Authenticate(frames::LobbyAuthenticate {
                local_id: Some(network.local_id().expect("TODO: Failed to read local id")),
            }),
        );

        sender
            .send(frame.to_string().expect("TODO: Failed to serialize frame"))
            .await
            .expect("TODO: Failed to send frame");

        while let Some(message) = receiver.next().await {
            let frame = frames::LobbyFrame::try_from_str(&message)
                .expect("TODO: Failed to deserialize frame");

            match frame.type_ {
                jsonrpc::FrameType::Request(request) => match request.data {
                    frames::LobbyRequest::Authenticate(_) => {}
                    frames::LobbyRequest::ChatMessage(data) => {
                        state.dispatch(Action::ChatMessage(
                            data.user_id.unwrap().into(),
                            data.message.unwrap().into(),
                        ));
                    }
                    frames::LobbyRequest::UserJoined(data) => {
                        state.dispatch(Action::UserJoined(data.user_id.unwrap().into()));
                        state.dispatch(Action::Status(Status::Requested));
                    }
                    frames::LobbyRequest::UserLeft(data) => {
                        state.dispatch(Action::UserLeft(data.user_id.unwrap().into()));
                        state.dispatch(Action::Status(Status::Requested));
                    }
                },
                jsonrpc::FrameType::Response(response) => match response.result {
                    Some(result) => match result {
                        frames::LobbyResponse::Authenticate(authenticated) => {
                            if authenticated {
                                state.dispatch(Action::Authenticated);
                                load_data(&network, &state, &props);
                            }
                        }
                    },
                    None => {}
                },
                jsonrpc::FrameType::RequestMethodNotFound(_) => {}
            };
        }

        state.dispatch(Action::Disconnected);
    });
}
