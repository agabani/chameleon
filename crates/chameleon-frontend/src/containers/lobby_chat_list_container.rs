use std::rc::Rc;

use chameleon_protocol::{
    frames::{LobbyAuthenticate, LobbyFrame, LobbyRequest},
    jsonrpc::FrameType,
};
use futures::StreamExt;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    components::{lobby_chat_list::LobbyChatList, lobby_chat_list_item::LobbyChatListItem},
    contexts::network::NetworkContext,
    hooks::lobby::use_lobby_subscription,
};

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,
}

#[function_component]
pub fn LobbyChatListContainer(props: &Props) -> Html {
    let state = use_reducer(State::default);
    let network = use_context::<NetworkContext>().unwrap();
    let channel = use_lobby_subscription(&props.id);

    {
        let mut sender = channel.0.lock().unwrap().clone();
        use_memo(
            move |_| {
                spawn_local(async move {
                    sender
                        .try_send(
                            LobbyFrame::new_request(
                                None,
                                LobbyRequest::Authenticate(LobbyAuthenticate {
                                    local_id: Some(network.local_id().unwrap()),
                                }),
                            )
                            .to_string()
                            .unwrap(),
                        )
                        .unwrap();
                });
            },
            (),
        );
    };

    {
        let state = state.clone();
        let receiver = channel.1.lock().unwrap().take();
        use_memo(
            |_| {
                if let Some(mut receiver) = receiver {
                    spawn_local(async move {
                        while let Some(message) = receiver.next().await {
                            match LobbyFrame::try_from_str(&message) {
                                Ok(frame) => {
                                    let FrameType::Request(request) = frame.type_ else {
                                        continue;
                                    };

                                    let LobbyRequest::ChatMessage(message) = request.data else {
                                        continue;
                                    };

                                    state.dispatch(Msg::ChatMessage(
                                        message.user_id.unwrap(),
                                        message.message.unwrap(),
                                    ));
                                }
                                Err(error) => gloo::console::error!(format!("{error:?}")),
                            }
                        }
                    });
                }
            },
            (),
        );
    };

    html! {
        <LobbyChatList>
            {
                state.messages.iter().map(|(name, message)| html! {
                    <LobbyChatListItem name={name} message={message} />
                }).collect::<Html>()
            }
        </LobbyChatList>
    }
}

#[derive(Default)]
struct State {
    messages: Vec<(AttrValue, AttrValue)>,
}

impl Reducible for State {
    type Action = Msg;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        match action {
            Msg::ChatMessage(name, message) => {
                let mut messages = self.messages.clone();
                messages.push((name.into(), message.into()));
                Rc::new(Self { messages })
            }
        }
    }
}

enum Msg {
    ChatMessage(String, String),
}
