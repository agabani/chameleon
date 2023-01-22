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
    domain::{lobby, LobbyId, LocalId, UserId},
    error::ApiError,
};

use super::{ToResource, ToResourceIdentifier, Variation};

pub const PATH: &str = "/api/v1/lobbies";
const TYPE: &str = "lobby";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_many))
        .route("/", post(create_one))
        .route("/:lobby_id", get(get_one))
        .route("/:lobby_id", patch(update_one))
        // relationships: host
        .route("/:lobby_id/relationships/host", get(get_relationships_host))
        .route(
            "/:lobby_id/relationships/host",
            patch(update_relationships_host),
        )
        .route("/:lobby_id/host", get(get_host))
        // relationships: members
        .route(
            "/:lobby_id/relationships/members",
            get(get_relationships_members),
        )
        .route(
            "/:lobby_id/relationships/members",
            patch(update_relationships_members),
        )
        .route("/:lobby_id/members", get(get_members))
        // actions
        .route(
            "/:lobby_id/actions/chat_message",
            post(actions_chat_message),
        )
        .route("/:lobby_id/actions/join", post(actions_join))
        .route("/:lobby_id/actions/leave", post(actions_leave))
}

#[tracing::instrument(skip(app_state))]
async fn create_one(
    State(app_state): State<AppState>,
    user_id: UserId,
    Json(document): Json<ResourcesDocument<LobbyAttributes>>,
) -> Result<Response, ApiError> {
    let resources = document.try_get_resources()?;

    match resources {
        Resources::Collection(_) => Ok(StatusCode::NOT_IMPLEMENTED.into_response()),
        Resources::Individual(resource) => {
            let name = resource.try_get_attribute(|a| a.name.as_ref(), "name", "Name")?;
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

            let require_passcode = resource.try_get_attribute(
                |attributes| attributes.require_passcode.as_ref(),
                "require_passcode",
                "Require Passcode",
            )?;

            let passcode = resource
                .try_get_attribute(
                    |attributes| attributes.passcode.as_ref(),
                    "passcode",
                    "Passcode",
                )
                .cloned();

            let lobby = match lobby::Lobby::create(
                name.clone(),
                user_id,
                passcode.clone().ok(),
                *require_passcode,
            ) {
                Ok((lobby, events)) => {
                    Database::save_lobby(&app_state.postgres_pool, lobby.id, &events).await?;
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
                links: Links([("self".to_string(), format!("{PATH}/{}", lobby.id.0))].into())
                    .into(),
            };

            Ok((
                StatusCode::CREATED,
                [(LOCATION, format!("{PATH}/{}", lobby.id.0))],
                Json(document),
            )
                .into_response())
        }
    }
}

#[tracing::instrument(skip(app_state))]
async fn get_one(
    State(app_state): State<AppState>,
    local_id: LocalId,
    Path(lobby_id): Path<LobbyId>,
) -> Result<Response, ApiError> {
    let lobby = Database::load_lobby(&app_state.postgres_pool, lobby_id)
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

#[tracing::instrument(skip(app_state))]
async fn get_many(
    State(app_state): State<AppState>,
    local_id: LocalId,
    Query(pagination): Query<Pagination>,
) -> Result<Response, ApiError> {
    let keyset_pagination = pagination.try_into()?;

    let (lobbies, after) =
        Database::query_lobby(&app_state.postgres_pool, keyset_pagination).await?;

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

#[tracing::instrument(skip(app_state))]
async fn update_one(
    State(app_state): State<AppState>,
    user_id: UserId,
    Path(lobby_id): Path<LobbyId>,
    Json(document): Json<ResourcesDocument<LobbyAttributes>>,
) -> Result<Response, ApiError> {
    let mut lobby = Database::load_lobby(&app_state.postgres_pool, lobby_id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("lobby", "Lobby"))))?;

    match document.try_get_resources()? {
        Resources::Collection(_) => Ok(StatusCode::NOT_IMPLEMENTED.into_response()),
        Resources::Individual(resource) => {
            let name =
                resource.try_get_attribute(|accessor| accessor.name.as_ref(), "name", "Name");
            let passcode = resource.try_get_attribute(
                |accessor| accessor.passcode.as_ref(),
                "passcode",
                "Passcode",
            );
            let require_passcode = resource.try_get_attribute(
                |accessor| accessor.require_passcode.as_ref(),
                "require_passcode",
                "Require Passcode",
            );

            match lobby.update(
                user_id,
                &name.cloned().ok(),
                &passcode.clone().cloned().ok(),
                require_passcode.cloned().ok(),
            ) {
                Ok(events) => {
                    Database::save_lobby(&app_state.postgres_pool, lobby_id, &events).await?;
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
    }
}

#[tracing::instrument(skip(app_state))]
async fn get_relationships_host(
    State(app_state): State<AppState>,
    local_id: LocalId,
    Path(lobby_id): Path<LobbyId>,
) -> Result<Response, ApiError> {
    let lobby = Database::load_lobby(&app_state.postgres_pool, lobby_id)
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

#[tracing::instrument(skip(_app_state))]
async fn update_relationships_host(
    State(_app_state): State<AppState>,
    user_id: UserId,
    Path(lobby_id): Path<LobbyId>,
) -> Result<Response, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED.into_response())
}

#[tracing::instrument(skip(app_state))]
async fn get_host(
    State(app_state): State<AppState>,
    local_id: LocalId,
    Path(lobby_id): Path<LobbyId>,
) -> Result<Response, ApiError> {
    let lobby = Database::load_lobby(&app_state.postgres_pool, lobby_id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("lobby", "Lobby"))))?;

    let user = Database::select_user(&app_state.postgres_pool, lobby.get_host())
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("user", "User"))))?;

    let document = ResourcesDocument {
        data: Some(Resources::Individual(user.to_resource(Variation::Nested))),
        errors: None,
        links: Some(Links(
            [("self".to_string(), format!("{PATH}/{}/host", lobby_id.0))].into(),
        )),
    };

    Ok((StatusCode::OK, Json(document)).into_response())
}

#[tracing::instrument(skip(_app_state))]
async fn get_relationships_members(
    State(_app_state): State<AppState>,
    local_id: LocalId,
    Path(lobby_id): Path<LobbyId>,
) -> Result<Response, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED.into_response())
}

#[tracing::instrument(skip(_app_state))]
async fn update_relationships_members(
    State(_app_state): State<AppState>,
    user_id: UserId,
    Path(lobby_id): Path<LobbyId>,
) -> Result<Response, ApiError> {
    Ok(StatusCode::NOT_IMPLEMENTED.into_response())
}

#[tracing::instrument(skip(app_state))]
async fn get_members(
    State(app_state): State<AppState>,
    local_id: LocalId,
    Path(lobby_id): Path<LobbyId>,
    Query(pagination): Query<Pagination>,
) -> Result<Response, ApiError> {
    let keyset_pagination = pagination.try_into()?;

    let (users, after) =
        Database::query_lobby_member(&app_state.postgres_pool, lobby_id, keyset_pagination).await?;

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
                        lobby_id.0, keyset_pagination.id, keyset_pagination.limit
                    ),
                ),
                (
                    "next".to_string(),
                    format!(
                        "{PATH}/{}/members?page[after]={after}&page[size]={}",
                        lobby_id.0, keyset_pagination.limit
                    ),
                ),
            ]
            .into(),
        )),
    };

    Ok((StatusCode::OK, Json(document)).into_response())
}

#[tracing::instrument(skip(app_state))]
async fn actions_chat_message(
    State(app_state): State<AppState>,
    user_id: UserId,
    Path(lobby_id): Path<LobbyId>,
    Json(document): Json<ResourcesDocument<ChatMessageAttributes>>,
) -> Result<Response, ApiError> {
    let mut lobby = Database::load_lobby(&app_state.postgres_pool, lobby_id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("lobby", "Lobby"))))?;

    let message = document
        .try_get_resources()
        .and_then(Resources::try_get_individual)
        .and_then(|r| r.try_get_attribute(|a| a.message.as_ref(), "message", "Message"))?
        .clone();

    match lobby.send_chat_message(user_id, message.clone()) {
        Ok(events) => Database::save_lobby(&app_state.postgres_pool, lobby_id, &events).await?,
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
                    message: Some(message),
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

#[tracing::instrument(skip(app_state))]
async fn actions_join(
    State(app_state): State<AppState>,
    user_id: UserId,
    Path(lobby_id): Path<LobbyId>,
    Json(document): Json<ResourcesDocument<LobbyAttributes>>,
) -> Result<Response, ApiError> {
    let mut lobby = Database::load_lobby(&app_state.postgres_pool, lobby_id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("lobby", "Lobby"))))?;

    let passcode = if lobby.require_passcode {
        let passcode = document
            .try_get_resources()?
            .try_get_individual()?
            .try_get_attribute(|a| a.passcode.as_ref(), "passcode", "Passcode")?;
        Some(passcode.clone())
    } else {
        None
    };

    match lobby.join(user_id, &passcode) {
        Ok(events) => Database::save_lobby(&app_state.postgres_pool, lobby_id, &events).await?,
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

#[tracing::instrument(skip(app_state))]
async fn actions_leave(
    State(app_state): State<AppState>,
    user_id: UserId,
    Path(lobby_id): Path<LobbyId>,
) -> Result<Response, ApiError> {
    let mut lobby = Database::load_lobby(&app_state.postgres_pool, lobby_id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("lobby", "Lobby"))))?;

    match lobby.leave(user_id) {
        Ok(events) => Database::save_lobby(&app_state.postgres_pool, lobby.id, &events).await?,
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

impl ToResourceIdentifier for LobbyId {
    const TYPE: &'static str = TYPE;

    fn __id(&self) -> String {
        self.0.to_string()
    }
}
