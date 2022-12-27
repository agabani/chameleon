use axum::{
    extract::{Path, Query, State},
    http::header::LOCATION,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use chameleon_protocol::{
    attributes::GameAttributes,
    jsonapi::{
        self, Links, Pagination, Relationship, Relationships, ResourceIdentifiers,
        ResourceIdentifiersDocument, Resources, ResourcesDocument,
    },
};

use crate::{
    database::Database,
    domain::{Game, GameId, LocalId, UserId},
    error::ApiError,
    AppState,
};

use super::{ToResource, ToResourceIdentifier, Variation};

pub const PATH: &str = "/api/v1/games";
const TYPE: &str = "game";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_many))
        .route("/", post(create_one))
        .route("/:game_id", get(get_one))
        .route("/:game_id", patch(update_one))
        .route("/:game_id/relationships/host", get(get_relationships_host))
        .route(
            "/:game_id/relationships/host",
            patch(|| async { StatusCode::NOT_IMPLEMENTED.into_response() }),
        )
        .route("/:game_id/host", get(get_host))
}

#[tracing::instrument(skip(app_state))]
async fn create_one(
    State(app_state): State<AppState>,
    user_id: UserId,
    Json(document): Json<ResourcesDocument<GameAttributes>>,
) -> Result<Response, ApiError> {
    let resources = document.try_get_resources()?;

    match resources {
        Resources::Collection(_) => Ok(StatusCode::NOT_IMPLEMENTED.into_response()),
        Resources::Individual(resource) => {
            let _type = resource.try_get_field(|r| r.type_.as_ref(), "type", "Type")?;
            let name = resource.try_get_attribute(|a| a.name.as_ref(), "name", "Name")?;

            let game = Game {
                id: GameId::random(),
                name: name.clone(),
                host: user_id,
            };

            let mut conn = app_state.postgres_pool.begin().await?;
            Database::insert_game(&mut conn, &game).await?;
            Database::insert_game_player(&mut conn, &game).await?;
            conn.commit().await?;

            let document = ResourcesDocument {
                data: Resources::Individual(game.to_resource(Variation::Root)).into(),
                errors: None,
                links: Links([("self".to_string(), format!("{PATH}/{}", game.id.0))].into()).into(),
            };

            Ok((
                StatusCode::CREATED,
                [(LOCATION, format!("{PATH}/{}", game.id.0))],
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
    Path(game_id): Path<GameId>,
) -> Result<Response, ApiError> {
    let game = Database::select_game(&app_state.postgres_pool, game_id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("game", "Game"))))?;

    let document = ResourcesDocument {
        data: Some(Resources::Individual(game.to_resource(Variation::Root))),
        errors: None,
        links: Some(Links(
            [("self".to_string(), format!("{PATH}/{}", game.id.0))].into(),
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

    let (games, after) = Database::query_game(&app_state.postgres_pool, keyset_pagination).await?;

    let document = ResourcesDocument {
        data: Some(Resources::Collection(
            games
                .iter()
                .map(|game| game.to_resource(Variation::Nested))
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
    Path(game_id): Path<GameId>,
    Json(document): Json<ResourcesDocument<GameAttributes>>,
) -> Result<Response, ApiError> {
    let mut game = Database::select_game(&app_state.postgres_pool, game_id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("game", "Game"))))?;

    match document.try_get_resources()? {
        Resources::Collection(_) => Ok(StatusCode::NOT_IMPLEMENTED.into_response()),
        Resources::Individual(resource) => {
            if let Some(attributes) = &resource.attributes {
                game = game.update_attributes(attributes);
                Database::update_game(&app_state.postgres_pool, &game).await?;
            };

            let document = ResourcesDocument {
                data: Some(Resources::Individual(game.to_resource(Variation::Root))),
                errors: None,
                links: Some(Links(
                    [("self".to_string(), format!("{PATH}/{}", game.id.0))].into(),
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
    Path(game_id): Path<GameId>,
) -> Result<Response, ApiError> {
    let game = Database::select_game(&app_state.postgres_pool, game_id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("game", "Game"))))?;

    let document = ResourceIdentifiersDocument {
        data: Some(ResourceIdentifiers::Individual(
            game.host.to_resource_identifier(),
        )),
        errors: None,
        links: Some(Links(
            [
                (
                    "self".to_string(),
                    format!("{PATH}/{}/relationships/host", game.id.0),
                ),
                ("related".to_string(), format!("{PATH}/{}/host", game.id.0)),
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
    Path(game_id): Path<GameId>,
) -> Result<Response, ApiError> {
    let game = Database::select_game(&app_state.postgres_pool, game_id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("game", "Game"))))?;

    let user = Database::select_user(&app_state.postgres_pool, game.host)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("user", "User"))))?;

    let document = ResourcesDocument {
        data: Some(Resources::Individual(user.to_resource(Variation::Nested))),
        errors: None,
        links: Some(Links(
            [("self".to_string(), format!("{PATH}/{}/host", game_id.0))].into(),
        )),
    };

    Ok((StatusCode::OK, Json(document)).into_response())
}

impl Game {
    pub fn update_attributes(&self, attributes: &GameAttributes) -> Game {
        Game {
            id: self.id,
            name: attributes.name.as_ref().unwrap_or(&self.name).clone(),
            host: self.host,
        }
    }
}

impl ToResource for Game {
    const PATH: &'static str = PATH;

    const TYPE: &'static str = TYPE;

    type Attributes = GameAttributes;

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

impl ToResourceIdentifier for GameId {
    const TYPE: &'static str = TYPE;

    fn __id(&self) -> String {
        self.0.to_string()
    }
}
