use chameleon_protocol::{attributes, jsonapi};
use wasm_bindgen_futures::spawn_local;
use yew::{
    prelude::*,
    suspense::{Suspension, SuspensionResult},
};

use crate::contexts::network::NetworkContext;

#[hook]
pub fn use_user(
    id: AttrValue,
) -> SuspensionResult<jsonapi::ResourcesDocument<attributes::UserAttributes>> {
    let network = use_context::<NetworkContext>().unwrap();
    let state = use_state(|| None::<jsonapi::ResourcesDocument<attributes::UserAttributes>>);

    if let Some(state) = state.as_ref() {
        return Ok(state.clone());
    }

    let (suspension, handle) = Suspension::new();

    spawn_local(async move {
        let document = network
            .get_user(&id)
            .await
            .unwrap_or_else(|_| jsonapi::ResourcesDocument::internal_server_error());
        state.set(Some(document));
        handle.resume();
    });

    Err(suspension)
}
