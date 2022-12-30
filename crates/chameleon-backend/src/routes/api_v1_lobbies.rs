use axum::{
    extract::{Path, Query, State},
    http::header::LOCATION,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use chameleon_protocol::{
    attributes::LobbyAttributes,
    jsonapi::{
        self, Links, Pagination, Relationship, Relationships, ResourceIdentifiers,
        ResourceIdentifiersDocument, Resources, ResourcesDocument,
    },
};

use crate::{
    database::Database,
    domain::{Lobby, LobbyId, LocalId, UserId},
    error::ApiError,
    AppState,
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
        .route("/:lobby_id/relationships/host", get(get_relationships_host))
        .route(
            "/:lobby_id/relationships/host",
            patch(|| async { StatusCode::NOT_IMPLEMENTED.into_response() }),
        )
        .route("/:lobby_id/host", get(get_host))
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
            let _type = resource.try_get_field(|r| r.type_.as_ref(), "type", "Type")?;
            let name = resource.try_get_attribute(|a| a.name.as_ref(), "name", "Name")?;

            let lobby = Lobby {
                id: LobbyId::random(),
                name: name.clone(),
                host: user_id,
            };

            let mut conn = app_state.postgres_pool.begin().await?;
            Database::insert_lobby(&mut conn, &lobby).await?;
            Database::insert_lobby_member(&mut conn, &lobby).await?;
            conn.commit().await?;

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
    let lobby = Database::select_lobby(&app_state.postgres_pool, lobby_id)
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
                .collect::<Vec<_>>(),
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
    let mut lobby = Database::select_lobby(&app_state.postgres_pool, lobby_id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("lobby", "Lobby"))))?;

    match document.try_get_resources()? {
        Resources::Collection(_) => Ok(StatusCode::NOT_IMPLEMENTED.into_response()),
        Resources::Individual(resource) => {
            if let Some(attributes) = &resource.attributes {
                lobby = lobby.update_attributes(attributes);
                Database::update_lobby(&app_state.postgres_pool, &lobby).await?;
            };

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
    let lobby = Database::select_lobby(&app_state.postgres_pool, lobby_id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("lobby", "Lobby"))))?;

    let document = ResourceIdentifiersDocument {
        data: Some(ResourceIdentifiers::Individual(
            lobby.host.to_resource_identifier(),
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

#[tracing::instrument(skip(app_state))]
async fn get_host(
    State(app_state): State<AppState>,
    local_id: LocalId,
    Path(lobby_id): Path<LobbyId>,
) -> Result<Response, ApiError> {
    let lobby = Database::select_lobby(&app_state.postgres_pool, lobby_id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("lobby", "Lobby"))))?;

    let user = Database::select_user(&app_state.postgres_pool, lobby.host)
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

impl Lobby {
    pub fn update_attributes(&self, attributes: &LobbyAttributes) -> Lobby {
        Lobby {
            id: self.id,
            name: attributes.name.as_ref().unwrap_or(&self.name).clone(),
            host: self.host,
        }
    }
}

impl ToResource for Lobby {
    const PATH: &'static str = PATH;

    const TYPE: &'static str = TYPE;

    type Attributes = LobbyAttributes;

    fn __attributes(&self) -> Option<Self::Attributes> {
        Some(Self::Attributes {
            name: Some(self.name.to_string()),
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
                        self.host.to_resource_identifier(),
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

impl ToResourceIdentifier for LobbyId {
    const TYPE: &'static str = TYPE;

    fn __id(&self) -> String {
        self.0.to_string()
    }
}
