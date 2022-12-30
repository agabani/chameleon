use chameleon_protocol::{
    attributes::{LobbyAttributes, UserAttributes},
    jsonapi::ResourcesDocument,
};
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
