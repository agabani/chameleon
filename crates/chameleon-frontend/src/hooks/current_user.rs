use chameleon_protocol::{attributes::UserAttributes, jsonapi::ResourcesDocument};
use wasm_bindgen_futures::spawn_local;
use yew::{
    prelude::*,
    suspense::{Suspension, SuspensionResult},
};

use crate::contexts::network::NetworkContext;

#[hook]
pub fn use_current_user() -> SuspensionResult<ResourcesDocument<UserAttributes>> {
    let network = use_context::<NetworkContext>().unwrap();
    let state = use_state(|| None::<ResourcesDocument<UserAttributes>>);

    if let Some(state) = state.as_ref() {
        return Ok(state.clone());
    }

    let (suspension, handle) = Suspension::new();

    spawn_local(async move {
        let Ok(userinfo) = network.get_userinfo().await else {
            state.set(Some(ResourcesDocument::internal_server_error()));
            return handle.resume();
        };

        let Some(userinfo) = userinfo else {
            state.set(Some(ResourcesDocument::unauthorized()));
            return handle.resume();
        };

        let Ok(user) = network.get_user(&userinfo.sub).await else {
            state.set(Some(ResourcesDocument::internal_server_error()));
            return handle.resume();
        };

        state.set(Some(user));
        handle.resume();
    });

    Err(suspension)
}
