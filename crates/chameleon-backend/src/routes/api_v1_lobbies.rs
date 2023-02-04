use axum::{
    extract::{Path, Query, State},
    http::header::LOCATION,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use chameleon_protocol::{
    attributes::{ChatMessageAttributes, LobbyAttributes},
    jsonapi::{
        self, Links, Pagination, Relationship, Relationships, ResourceIdentifiers,
        ResourceIdentifiersDocument, Resources, ResourcesDocument,
    },
};

use crate::{
    app::AppState,
    database::Database,
    domain::{lobby, lobby_id, local_id, user_id},
    error::ApiError,
};

use super::{ToResource, ToResourceIdentifier, Variation};

pub const PATH: &str = "/api/v1/lobbies";
const TYPE: &str = "lobby";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_many))
        .route("/", post(create_one))
        .route("/:id", get(get_one))
        .route("/:id", patch(update_one))
        // relationships: host
        .route("/:id/relationships/host", get(get_relationships_host))
        .route("/:id/relationships/host", patch(update_relationships_host))
        .route("/:id/host", get(get_host))
        // relationships: members
        .route("/:id/relationships/members", get(get_relationships_members))
        .route(
            "/:id/relationships/members",
            patch(update_relationships_members),
        )
        .route("/:id/members", get(get_members))
        // actions
        .route("/:id/actions/chat_message", post(actions_chat_message))
        .route("/:id/actions/join", post(actions_join))
        .route("/:id/actions/leave", post(actions_leave))
}

#[tracing::instrument(skip(state))]
async fn create_one(
    State(state): State<AppState>,
    user_id: user_id::UserId,
    Json(document): Json<ResourcesDocument<LobbyAttributes>>,
) -> Result<Response, ApiError> {
    let resource = document.try_get_individual()?;

    let name = resource
        .try_get_attribute(|a| a.name.as_ref(), "name", "Name")?
        .as_str();
    if name.is_empty() {
        return Err(ApiError::JsonApi(Box::new(jsonapi::Error {
            status: 422,
            source: Some(jsonapi::Source {
                header: None,
                parameter: None,
                pointer: Some("/data/attributes/name".to_string()),
            }),
            title: Some("Invalid Attribute".to_string()),
            detail: Some("Name must not be empty".to_string()),
        })));
    }

    let require_passcode = *resource.try_get_attribute(
        |a| a.require_passcode.as_ref(),
        "require_passcode",
        "Require Passcode",
    )?;

    let passcode = resource
        .try_get_attribute(|a| a.passcode.as_ref(), "passcode", "Passcode")
        .map(String::as_str);

    let lobby = match lobby::Lobby::create(name, user_id, passcode.clone().ok(), require_passcode) {
        Ok((lobby, events)) => {
            Database::save_lobby(&state.pool, lobby.id, &events).await?;
            lobby
        }
        Err(error) => match error {
            lobby::CreateError::MissingPasscode => {
                return Err(ApiError::JsonApi(passcode.unwrap_err()));
            }
        },
    };

    let document = ResourcesDocument {
        data: Resources::Individual(lobby.to_resource(Variation::Root)).into(),
        errors: None,
        links: Links([("self".to_string(), format!("{PATH}/{}", lobby.id.0))].into()).into(),
    };

    Ok((
        StatusCode::CREATED,
        [(LOCATION, format!("{PATH}/{}", lobby.id.0))],
        Json(document),
    )
        .into_response())
}

#[tracing::instrument(skip(state))]
async fn get_one(
    State(state): State<AppState>,
    local_id: local_id::LocalId,
    Path(id): Path<lobby_id::LobbyId>,
) -> Result<Response, ApiError> {
    let lobby = Database::load_lobby(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("lobby", "Lobby"))))?;

    let document = ResourcesDocument {
        data: Some(Resources::Individual(lobby.to_resource(Variation::Root))),
        errors: None,
        links: Some(Links(
            [("self".to_string(), format!("{PATH}/{}", lobby.id.0))].into(),
        )),
    };

    Ok((StatusCode::OK, Json(document)).into_response())
}

#[tracing::instrument(skip(state))]
async fn get_many(
    State(state): State<AppState>,
    local_id: local_id::LocalId,
    Query(pagination): Query<Pagination>,
) -> Result<Response, ApiError> {
    let keyset_pagination = pagination.try_into()?;

    let (lobbies, after) = Database::query_lobby(&state.pool, keyset_pagination).await?;

    let document = ResourcesDocument {
        data: Some(Resources::Collection(
            lobbies
                .iter()
                .map(|lobby| lobby.to_resource(Variation::Nested))
                .collect(),
        )),
        errors: None,
        links: Some(Links(
            [
                (
                    "self".to_string(),
                    format!(
                        "{PATH}?page[after]={}&page[size]={}",
                        keyset_pagination.id, keyset_pagination.limit
                    ),
                ),
                (
                    "next".to_string(),
                    format!(
                        "{PATH}?page[after]={}&page[size]={}",
                        after, keyset_pagination.limit
                    ),
                ),
            ]
            .into(),
        )),
    };

    Ok((StatusCode::OK, Json(document)).into_response())
}

#[tracing::instrument(skip(state))]
async fn update_one(
    State(state): State<AppState>,
    user_id: user_id::UserId,
    Path(id): Path<lobby_id::LobbyId>,
    Json(document): Json<ResourcesDocument<LobbyAttributes>>,
) -> Result<Response, ApiError> {
    let resource = document.try_get_individual()?;

    let mut lobby = Database::load_lobby(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("lobby", "Lobby"))))?;

    let name = resource
        .try_get_attribute(|a| a.name.as_ref(), "name", "Name")
        .map(String::as_str);
    let passcode = resource
        .try_get_attribute(|a| a.passcode.as_ref(), "passcode", "Passcode")
        .map(String::as_str);
    let require_passcode = resource
        .try_get_attribute(
            |a| a.require_passcode.as_ref(),
            "require_passcode",
            "Require Passcode",
        )
        .map(|x| *x);

    match lobby.update(
        user_id,
        name.ok(),
        passcode.clone().ok(),
        require_passcode.ok(),
    ) {
        Ok(events) => {
            Database::save_lobby(&state.pool, lobby.id, &events).await?;
        }
        Err(error) => match error {
            lobby::UpdateError::MissingPasscode => {
                passcode?;
            }
            lobby::UpdateError::NotHost => {
                return Err(ApiError::JsonApi(Box::new(jsonapi::Error::forbidden())));
            }
        },
    }

    let document = ResourcesDocument {
        data: Some(Resources::Individual(lobby.to_resource(Variation::Root))),
        errors: None,
        links: Some(Links(
            [("self".to_string(), format!("{PATH}/{}", lobby.id.0))].into(),
        )),
    };

    Ok((StatusCode::OK, Json(document)).into_response())
}

#[tracing::instrument(skip(state))]
async fn get_relationships_host(
    State(state): State<AppState>,
    local_id: local_id::LocalId,
    Path(id): Path<lobby_id::LobbyId>,
) -> Result<Response, ApiError> {
    let lobby = Database::load_lobby(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("lobby", "Lobby"))))?;

    let document = ResourceIdentifiersDocument {
        data: Some(ResourceIdentifiers::Individual(
            lobby.get_host().to_resource_identifier(),
        )),
        errors: None,
        links: Some(Links(
            [
                (
                    "self".to_string(),
                    format!("{PATH}/{}/relationships/host", lobby.id.0),
                ),
                ("related".to_string(), format!("{PATH}/{}/host", lobby.id.0)),
            ]
            .into(),
        )),
    };

    Ok((StatusCode::OK, Json(document)).into_response())
}

#[tracing::instrument(skip(_state))]
async fn update_relationships_host(
    State(_state): State<AppState>,
    user_id: user_id::UserId,
    Path(id): Path<lobby_id::LobbyId>,
) -> Result<Response, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED.into_response())
}

#[tracing::instrument(skip(state))]
async fn get_host(
    State(state): State<AppState>,
    local_id: local_id::LocalId,
    Path(id): Path<lobby_id::LobbyId>,
) -> Result<Response, ApiError> {
    let lobby = Database::load_lobby(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("lobby", "Lobby"))))?;

    let user = Database::load_user(&state.pool, lobby.get_host())
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("user", "User"))))?;

    let document = ResourcesDocument {
        data: Some(Resources::Individual(user.to_resource(Variation::Nested))),
        errors: None,
        links: Some(Links(
            [("self".to_string(), format!("{PATH}/{}/host", lobby.id.0))].into(),
        )),
    };

    Ok((StatusCode::OK, Json(document)).into_response())
}

#[tracing::instrument(skip(_state))]
async fn get_relationships_members(
    State(_state): State<AppState>,
    local_id: local_id::LocalId,
    Path(id): Path<lobby_id::LobbyId>,
) -> Result<Response, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED.into_response())
}

#[tracing::instrument(skip(_state))]
async fn update_relationships_members(
    State(_state): State<AppState>,
    user_id: user_id::UserId,
    Path(id): Path<lobby_id::LobbyId>,
) -> Result<Response, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED.into_response())
}

#[tracing::instrument(skip(state))]
async fn get_members(
    State(state): State<AppState>,
    local_id: local_id::LocalId,
    Path(id): Path<lobby_id::LobbyId>,
    Query(pagination): Query<Pagination>,
) -> Result<Response, ApiError> {
    let keyset_pagination = pagination.try_into()?;

    let (users, after) = Database::query_lobby_member(&state.pool, id, keyset_pagination).await?;

    let document = ResourcesDocument {
        data: Some(Resources::Collection(
            users
                .iter()
                .map(|user| user.to_resource(Variation::Nested))
                .collect(),
        )),
        errors: None,
        links: Some(Links(
            [
                (
                    "self".to_string(),
                    format!(
                        "{PATH}/{}/members?page[after]={}&page[size]={}",
                        id.0, keyset_pagination.id, keyset_pagination.limit
                    ),
                ),
                (
                    "next".to_string(),
                    format!(
                        "{PATH}/{}/members?page[after]={after}&page[size]={}",
                        id.0, keyset_pagination.limit
                    ),
                ),
            ]
            .into(),
        )),
    };

    Ok((StatusCode::OK, Json(document)).into_response())
}

#[tracing::instrument(skip(state))]
async fn actions_chat_message(
    State(state): State<AppState>,
    user_id: user_id::UserId,
    Path(id): Path<lobby_id::LobbyId>,
    Json(document): Json<ResourcesDocument<ChatMessageAttributes>>,
) -> Result<Response, ApiError> {
    let message = document.try_get_attribute(|a| a.message.as_ref(), "message", "Message")?;

    let mut lobby = Database::load_lobby(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("lobby", "Lobby"))))?;

    match lobby.send_chat_message(user_id, message) {
        Ok(events) => Database::save_lobby(&state.pool, lobby.id, &events).await?,
        Err(error) => match error {
            lobby::SendChatMessageError::NotMember => {
                return Err(ApiError::JsonApi(Box::new(jsonapi::Error::forbidden())))
            }
        },
    };

    Ok((
        StatusCode::ACCEPTED,
        Json(ResourcesDocument {
            data: Some(Resources::Individual(jsonapi::Resource {
                id: None,
                type_: Some("chat_message".to_string()),
                attributes: Some(ChatMessageAttributes {
                    message: Some(message.to_string()),
                }),
                links: None,
                relationships: None,
            })),
            errors: None,
            links: Some(Links(
                [(
                    "self".to_string(),
                    format!("{PATH}/{}/actions/chat_message", lobby.id.0),
                )]
                .into(),
            )),
        }),
    )
        .into_response())
}

#[tracing::instrument(skip(state))]
async fn actions_join(
    State(state): State<AppState>,
    user_id: user_id::UserId,
    Path(id): Path<lobby_id::LobbyId>,
    Json(document): Json<ResourcesDocument<LobbyAttributes>>,
) -> Result<Response, ApiError> {
    let mut lobby = Database::load_lobby(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("lobby", "Lobby"))))?;

    let passcode = if lobby.require_passcode {
        let passcode = document
            .try_get_attribute(|a| a.passcode.as_ref(), "passcode", "Passcode")?
            .as_str();
        Some(passcode)
    } else {
        None
    };

    match lobby.join(user_id, passcode) {
        Ok(events) => Database::save_lobby(&state.pool, lobby.id, &events).await?,
        Err(error) => match error {
            lobby::JoinError::AlreadyJoined => {
                // silently continue...
            }
            lobby::JoinError::IncorrectPasscode => {
                return Err(ApiError::JsonApi(Box::new(jsonapi::Error::forbidden())));
            }
        },
    };

    let document = ResourceIdentifiersDocument {
        data: Some(ResourceIdentifiers::Individual(
            user_id.to_resource_identifier(),
        )),
        errors: None,
        links: Some(Links(
            [
                (
                    "self".to_string(),
                    format!("{PATH}/{}/relationships/members", lobby.id.0),
                ),
                (
                    "related".to_string(),
                    format!("{PATH}/{}/members", lobby.id.0),
                ),
            ]
            .into(),
        )),
    };

    Ok((StatusCode::OK, Json(document)).into_response())
}

#[tracing::instrument(skip(state))]
async fn actions_leave(
    State(state): State<AppState>,
    user_id: user_id::UserId,
    Path(id): Path<lobby_id::LobbyId>,
) -> Result<Response, ApiError> {
    let mut lobby = Database::load_lobby(&state.pool, id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("lobby", "Lobby"))))?;

    match lobby.leave(user_id) {
        Ok(events) => Database::save_lobby(&state.pool, lobby.id, &events).await?,
        Err(error) => match error {
            lobby::LeaveError::NotMember => {
                return Err(ApiError::JsonApi(Box::new(jsonapi::Error::forbidden())));
            }
        },
    };

    let document = ResourceIdentifiersDocument {
        data: None,
        errors: None,
        links: None,
    };

    Ok((StatusCode::OK, Json(document)).into_response())
}

impl ToResource for lobby::Lobby {
    const PATH: &'static str = PATH;

    const TYPE: &'static str = TYPE;

    type Attributes = LobbyAttributes;

    fn __attributes(&self) -> Option<Self::Attributes> {
        Some(Self::Attributes {
            name: Some(self.name.to_string()),
            passcode: None,
            require_passcode: Some(self.require_passcode),
        })
    }

    fn __id(&self) -> String {
        self.id.0.to_string()
    }

    fn __relationships(&self) -> Option<Relationships> {
        Some(Relationships(
            [(
                "host".to_string(),
                Relationship {
                    data: Some(ResourceIdentifiers::Individual(
                        self.members
                            .iter()
                            .find(|member| member.host)
                            .unwrap()
                            .user_id
                            .to_resource_identifier(),
                    )),
                    links: Some(Links(
                        [
                            (
                                "self".to_string(),
                                format!("{}/{}/relationships/host", Self::PATH, self.id.0),
                            ),
                            (
                                "related".to_string(),
                                format!("{}/{}/host", Self::PATH, self.id.0),
                            ),
                        ]
                        .into(),
                    )),
                },
            )]
            .into(),
        ))
    }
}

impl ToResource for lobby::Query {
    const PATH: &'static str = PATH;

    const TYPE: &'static str = TYPE;

    type Attributes = LobbyAttributes;

    fn __attributes(&self) -> Option<Self::Attributes> {
        Some(Self::Attributes {
            name: Some(self.name.to_string()),
            passcode: None,
            require_passcode: Some(self.require_passcode),
        })
    }

    fn __id(&self) -> String {
        self.id.0.to_string()
    }

    fn __relationships(&self) -> Option<Relationships> {
        None
    }
}

impl ToResourceIdentifier for lobby_id::LobbyId {
    const TYPE: &'static str = TYPE;

    fn __id(&self) -> String {
        self.0.to_string()
    }
}
