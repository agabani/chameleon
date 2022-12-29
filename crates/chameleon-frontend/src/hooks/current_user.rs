use chameleon_protocol::{attributes::UserAttributes, jsonapi::ResourcesDocument};
use wasm_bindgen_futures::spawn_local;
use yew::{
    prelude::*,
    suspense::{Suspension, SuspensionResult},
};

use crate::contexts::network::NetworkContext;

#[derive(Default, PartialEq)]
pub struct CurrentUserState {
    response: Option<ResourcesDocument<UserAttributes>>,
}

#[hook]
pub fn use_current_user() -> SuspensionResult<ResourcesDocument<UserAttributes>> {
    let network = use_context::<NetworkContext>().unwrap();
    let state = use_reducer(CurrentUserState::default);

    if let Some(document) = &state.response {
        return Ok(document.clone());
    }

    let (suspension, handle) = Suspension::new();

    spawn_local(async move {
        let Ok(userinfo) = network.get_userinfo().await else {
            state.dispatch(CurrentUserState {
                response: Some(ResourcesDocument::internal_server_error())
            });
            return handle.resume();
        };

        let Some(userinfo) = userinfo else {
            state.dispatch(CurrentUserState {
                response: Some(ResourcesDocument::unauthorized())
            });
            return handle.resume();
        };

        let user = network.get_user(&userinfo.sub).await.unwrap();
        state.dispatch(CurrentUserState {
            response: Some(user),
        });
        handle.resume();
    });

    Err(suspension)
}

impl Reducible for CurrentUserState {
    type Action = Self;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        action.into()
    }
}
