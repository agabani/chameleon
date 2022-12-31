use std::sync::Mutex;

use chameleon_protocol::{
    attributes::{LobbyAttributes, UserAttributes},
    jsonapi::ResourcesDocument,
};
use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    FutureExt, SinkExt, StreamExt,
};
use gloo::net::websocket::Message;
use wasm_bindgen_futures::spawn_local;
use yew::{
    prelude::*,
    suspense::{Suspension, SuspensionResult},
};

use crate::contexts::network::NetworkContext;

#[hook]
pub fn use_lobby(id: &str) -> SuspensionResult<ResourcesDocument<LobbyAttributes>> {
    let network = use_context::<NetworkContext>().unwrap();
    let state = use_state(|| None::<ResourcesDocument<LobbyAttributes>>);

    if let Some(state) = state.as_ref() {
        return Ok(state.clone());
    }

    let (suspension, handle) = Suspension::new();

    let id = id.to_string();
    spawn_local(async move {
        let document = network
            .get_lobby(&id)
            .await
            .unwrap_or_else(|_| ResourcesDocument::internal_server_error());
        state.set(Some(document));
        handle.resume();
    });

    Err(suspension)
}

#[hook]
pub fn use_lobby_host(id: &str) -> SuspensionResult<ResourcesDocument<UserAttributes>> {
    let network = use_context::<NetworkContext>().unwrap();
    let state = use_state(|| None::<ResourcesDocument<UserAttributes>>);

    if let Some(state) = state.as_ref() {
        return Ok(state.clone());
    }

    let (suspension, handle) = Suspension::new();

    let id = id.to_string();
    spawn_local(async move {
        let document = network
            .get_lobby_host(&id)
            .await
            .unwrap_or_else(|_| ResourcesDocument::internal_server_error());
        state.set(Some(document));
        handle.resume();
    });

    Err(suspension)
}

#[hook]
pub fn use_lobby_members(
    id: &str,
    next: Option<String>,
) -> SuspensionResult<ResourcesDocument<UserAttributes>> {
    let network = use_context::<NetworkContext>().unwrap();
    let state = use_state(|| None::<ResourcesDocument<UserAttributes>>);

    if let Some(state) = state.as_ref() {
        return Ok(state.clone());
    }

    let (suspension, handle) = Suspension::new();

    let id = id.to_string();
    spawn_local(async move {
        let document = network
            .get_lobby_members(&id, next)
            .await
            .unwrap_or_else(|_| ResourcesDocument::internal_server_error());
        state.set(Some(document));
        handle.resume();
    });

    Err(suspension)
}

#[allow(clippy::type_complexity)]
#[hook]
pub fn use_lobby_subscription(
    id: &str,
) -> UseStateHandle<(Mutex<Sender<String>>, Mutex<Option<Receiver<String>>>)> {
    let network = use_context::<NetworkContext>().unwrap();
    let state = use_state(|| {
        let websocket = network.subscribe_lobby(id).unwrap();

        let (tx_send, mut tx_recv) = channel::<String>(1000);
        let (mut rx_send, rx_recv) = channel::<String>(1000);

        spawn_local(async move {
            let (mut sink, mut stream) = websocket.split();

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

            let websocket = sink.reunite(stream).expect("Failed to reunite web socket");

            websocket
                .close(None, None)
                .expect("Failed to close web socket");
        });

        (Mutex::new(tx_send), Mutex::new(Some(rx_recv)))
    });

    state
}
