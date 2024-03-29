use std::rc::Rc;

use chameleon_protocol::{
    attributes::{ChatMessageAttributes, LobbyAttributes, UserAttributes},
    jsonapi::{ResourceIdentifiersDocument, ResourcesDocument},
    openid_connect,
};
use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    FutureExt, SinkExt, StreamExt,
};
use gloo::{
    net::{
        http::Request,
        websocket::{futures::WebSocket, Message},
    },
    storage::{errors::StorageError, LocalStorage, Storage},
    utils::document,
};
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Default, PartialEq)]
pub struct NetworkState {}

pub type NetworkContext = UseReducerHandle<NetworkState>;

#[derive(PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

#[function_component]
pub fn NetworkProvider(props: &Props) -> Html {
    let context = use_reducer(NetworkState::default);

    html! {
        <ContextProvider<NetworkContext> context={context}>
            { props.children.clone() }
        </ContextProvider<NetworkContext>>
    }
}

impl Reducible for NetworkState {
    type Action = Self;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        action.into()
    }
}

impl NetworkState {
    pub async fn action_lobby_chat_message(
        &self,
        id: &str,
        document: &ResourcesDocument<ChatMessageAttributes>,
    ) -> Result<ResourcesDocument<ChatMessageAttributes>, gloo::net::Error> {
        Request::post(&format!("/api/v1/lobbies/{id}/actions/chat_message"))
            .authentication_headers()
            .json(document)?
            .send()
            .await?
            .json()
            .await
    }

    pub async fn action_lobby_join(
        &self,
        id: &str,
        document: &ResourcesDocument<LobbyAttributes>,
    ) -> Result<ResourceIdentifiersDocument, gloo::net::Error> {
        Request::post(&format!("/api/v1/lobbies/{id}/actions/join"))
            .authentication_headers()
            .json(document)?
            .send()
            .await?
            .json()
            .await
    }

    pub async fn action_lobby_leave(
        &self,
        id: &str,
    ) -> Result<ResourceIdentifiersDocument, gloo::net::Error> {
        Request::post(&format!("/api/v1/lobbies/{id}/actions/leave"))
            .authentication_headers()
            .send()
            .await?
            .json()
            .await
    }

    pub async fn create_lobby(
        &self,
        document: &ResourcesDocument<LobbyAttributes>,
    ) -> Result<ResourcesDocument<LobbyAttributes>, gloo::net::Error> {
        Request::post("/api/v1/lobbies")
            .authentication_headers()
            .json(document)?
            .send()
            .await?
            .json()
            .await
    }

    pub async fn create_user(
        &self,
        document: &ResourcesDocument<UserAttributes>,
    ) -> Result<ResourcesDocument<UserAttributes>, gloo::net::Error> {
        Request::post("/api/v1/users")
            .authentication_headers()
            .json(document)?
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_lobby(
        &self,
        id: &str,
    ) -> Result<ResourcesDocument<LobbyAttributes>, gloo::net::Error> {
        Request::get(&format!("/api/v1/lobbies/{id}"))
            .authentication_headers()
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_lobby_host(
        &self,
        id: &str,
    ) -> Result<ResourcesDocument<UserAttributes>, gloo::net::Error> {
        Request::get(&format!("/api/v1/lobbies/{id}/host"))
            .authentication_headers()
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_lobby_members(
        &self,
        id: &str,
        next: Option<String>,
    ) -> Result<ResourcesDocument<UserAttributes>, gloo::net::Error> {
        Request::get(&next.unwrap_or_else(|| format!("/api/v1/lobbies/{id}/members")))
            .authentication_headers()
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_user(
        &self,
        id: &str,
    ) -> Result<ResourcesDocument<UserAttributes>, gloo::net::Error> {
        Request::get(&format!("/api/v1/users/{id}"))
            .authentication_headers()
            .send()
            .await?
            .json()
            .await
    }

    pub async fn get_userinfo(&self) -> Result<Option<openid_connect::UserInfo>, gloo::net::Error> {
        let response = Request::get("/api/v1/userinfo")
            .authentication_headers()
            .send()
            .await?;

        match response.status() {
            200 => Ok(Some(response.json().await?)),
            401 => Ok(None),
            status => todo!("Unexpected status code: {status}"),
        }
    }

    pub async fn query_lobby(
        &self,
        next: Option<String>,
    ) -> Result<ResourcesDocument<LobbyAttributes>, gloo::net::Error> {
        Request::get(&next.unwrap_or_else(|| "/api/v1/lobbies".to_string()))
            .authentication_headers()
            .send()
            .await?
            .json()
            .await
    }

    #[allow(clippy::unused_self)]
    pub fn subscribe_lobby(&self, id: &str) -> Result<WebSocket, gloo::utils::errors::JsError> {
        let location = document().location().expect("Failed to read location");
        let host = location.host().expect("Failed to read location host");
        let protocol = location
            .protocol()
            .expect("Failed to read location protocol");

        let url = format!(
            "{}//{host}/ws/v1/lobbies/{id}",
            if protocol == "https:" { "wss:" } else { "ws:" }
        );

        WebSocket::open(&url)
    }

    pub async fn update_user(
        &self,
        id: &str,
        document: &ResourcesDocument<UserAttributes>,
    ) -> Result<ResourcesDocument<UserAttributes>, gloo::net::Error> {
        Request::patch(&format!("/api/v1/users/{id}"))
            .authentication_headers()
            .json(document)?
            .send()
            .await?
            .json()
            .await
    }

    #[allow(clippy::unused_self)]
    pub fn local_id(&self) -> Result<String, StorageError> {
        local_id()
    }

    #[allow(clippy::unused_self)]
    pub fn web_socket_to_channels(
        &self,
        web_socket: WebSocket,
    ) -> (Sender<String>, Receiver<String>) {
        let (tx_send, mut tx_recv) = channel::<String>(1000);
        let (mut rx_send, rx_recv) = channel::<String>(1000);

        spawn_local(async move {
            let (mut sink, mut stream) = web_socket.split();

            while futures::select! {
                message = tx_recv.next() => {
                    match message {
                        Some(message) => {
                            match sink.send(Message::Text(message)).await {
                                Ok(_) => true,
                                Err(error) => {
                                    gloo::console::error!(format!("{error:?}"));
                                    false
                                }
                            }
                        },
                        None => false
                    }
                },
                message = stream.next().fuse() => {
                    match message {
                        Some(message) => {
                            match message {
                                Ok(message) => {
                                    match message {
                                        Message::Text(message) => {
                                            match rx_send.try_send(message) {
                                                Ok(_) => true,
                                                Err(error) => {
                                                    gloo::console::error!(format!("{error:?}"));
                                                    false
                                                }
                                            }
                                        },
                                        Message::Bytes(_) => true
                                    }
                                },
                                Err(error) => {
                                    gloo::console::error!(format!("{error:?}"));
                                    false
                                },
                            }
                        },
                        None => false
                    }
                },
            } {}

            let web_socket = sink.reunite(stream).expect("Failed to reunite web socket");

            web_socket
                .close(None, None)
                .expect("Failed to close web socket");
        });

        (tx_send, rx_recv)
    }
}

trait RequestExt {
    fn authentication_headers(self) -> Self;
}

impl RequestExt for gloo::net::http::Request {
    fn authentication_headers(self) -> Self {
        self.header("x-chameleon-local-id", &local_id().unwrap())
    }
}

fn local_id() -> Result<String, StorageError> {
    const KEY: &str = "local-id";

    match LocalStorage::get(KEY) {
        Ok(value) => Ok(value),
        Err(StorageError::KeyNotFound(_)) => {
            match LocalStorage::set(KEY, Uuid::new_v4().to_string()) {
                Ok(_) => LocalStorage::get(KEY),
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}
