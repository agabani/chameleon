use std::{collections::HashMap, rc::Rc};

use chameleon_protocol::{
    attributes,
    frames::{self, LobbyFrame},
    jsonapi,
    jsonrpc::FrameType,
};
use futures::{channel::mpsc::Sender, SinkExt, StreamExt};
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
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
    let network = use_context::<NetworkContext>().unwrap();
    let state = use_reducer(State::default);

    let onsubmit = {
        let authenticated = state.authenticated;

        let id = props.id.clone();
        let network = network.clone();
        let state = state.clone();
        use_callback(
            move |content: AttrValue, _| {
                send_chat_message(&network, &state, &id, &content);
            },
            authenticated,
        )
    };

    web_socket(network, state.clone(), props);

    html! {
        <div class="lobby">
            <div>{ "lobby members" }</div>
            <LobbyMemberList>
            {
                present_members(&state).iter().map(|(id, name)| html! {
                    <LobbyMemberListItem key={id.as_str()} id={id} name={name} />
                }).collect::<Html>()
            }
            </LobbyMemberList>

            <div>{ "lobby chat" }</div>
            <LobbyChatList>
            {
                present_messages(&state).iter().map(|(key, _, name, content)| html! {
                    <LobbyChatListItem key={key.to_string()} name={name} message={content} />
                }).collect::<Html>()
            }
            </LobbyChatList>
            <LobbyChatInput disabled={!state.authenticated} onsubmit={onsubmit} />
        </div>
    }
}

#[derive(Default)]
struct State {
    authenticated: bool,

    sender: Option<Sender<String>>,

    // members: user_id, name
    members: HashMap<AttrValue, AttrValue>,

    // messages: key, user_id, content
    messages: Vec<(Uuid, AttrValue, AttrValue)>,
}

impl Reducible for State {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            Action::Authenticated => Rc::new(Self {
                authenticated: true,
                sender: self.sender.clone(),
                members: self.members.clone(),
                messages: self.messages.clone(),
            }),
            Action::ChatMessage(id, content) => {
                let mut messages = self.messages.clone();
                messages.push((Uuid::new_v4(), id, content));
                Rc::new(Self {
                    authenticated: self.authenticated,
                    sender: self.sender.clone(),
                    members: self.members.clone(),
                    messages,
                })
            }
            Action::Connected(sender) => Rc::new(Self {
                authenticated: self.authenticated,
                sender: Some(sender),
                members: self.members.clone(),
                messages: self.messages.clone(),
            }),
            Action::Disconnected => Rc::new(Self {
                authenticated: false,
                sender: None,
                members: self.members.clone(),
                messages: self.messages.clone(),
            }),
            Action::UserJoined(_) | Action::UserLeft(_) => self,
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

    spawn_local(async move {
        let frame = LobbyFrame::new_request(
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
            let frame =
                LobbyFrame::try_from_str(&message).expect("TODO: Failed to deserialize frame");

            match frame.type_ {
                FrameType::Request(request) => match request.data {
                    frames::LobbyRequest::Authenticate(_) => {}
                    frames::LobbyRequest::ChatMessage(data) => {
                        state.dispatch(Action::ChatMessage(
                            data.user_id.unwrap().into(),
                            data.message.unwrap().into(),
                        ));
                    }
                    frames::LobbyRequest::UserJoined(data) => {
                        state.dispatch(Action::UserJoined(data.user_id.unwrap().into()));
                    }
                    frames::LobbyRequest::UserLeft(data) => {
                        state.dispatch(Action::UserLeft(data.user_id.unwrap().into()));
                    }
                },
                FrameType::Response(response) => match response.result {
                    Some(result) => match result {
                        frames::LobbyResponse::Authenticate(authenticated) => {
                            if authenticated {
                                state.dispatch(Action::Authenticated);
                            }
                        }
                    },
                    None => {}
                },
                FrameType::RequestMethodNotFound(_) => {}
            };
        }

        state.dispatch(Action::Disconnected);
    });
}

/// presents members: user id, user name
fn present_members(state: &State) -> Vec<(AttrValue, AttrValue)> {
    let mut members = state
        .members
        .iter()
        .map(|(a, b)| (a.clone(), b.clone()))
        .collect::<Vec<_>>();
    members.sort_by(|a, b| a.0.cmp(&b.0));
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

fn send_chat_message(
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
