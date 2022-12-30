use std::rc::Rc;

use chameleon_protocol::{
    attributes::{LobbyAttributes, UserAttributes},
    jsonapi::ResourcesDocument,
    openid_connect,
};
use gloo::{
    net::http::Request,
    storage::{errors::StorageError, LocalStorage, Storage},
};
use uuid::Uuid;
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
}

trait RequestExt {
    fn authentication_headers(self) -> Self;
}

impl RequestExt for gloo::net::http::Request {
    fn authentication_headers(self) -> Self {
        const KEY: &str = "local-id";

        let value: String = {
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
        .unwrap();

        self.header("x-chameleon-local-id", &value)
    }
}
