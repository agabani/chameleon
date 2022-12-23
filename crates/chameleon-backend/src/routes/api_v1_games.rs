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
        self, Document, Errors, Links, Pagination, Relationship, Relationships, Resource,
        ResourceIdentifier, ResourceIdentifiers, Resources,
    },
};

use crate::{
    database::Database,
    domain::{Game, GameId, LocalId, UserId},
    error::ApiError,
    AppState,
};

pub const PATH: &str = "/api/v1/games";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_many))
        .route("/", post(create_one))
        .route("/:game_id", get(get_one))
        .route("/:game_id", patch(update_one))
        .route(
            "/:game_id/relationships/host",
            get(|| async { StatusCode::NOT_IMPLEMENTED.into_response() }),
        )
        .route(
            "/:game_id/relationships/host",
            patch(|| async { StatusCode::NOT_IMPLEMENTED.into_response() }),
        )
        .route(
            "/:game_id/host",
            get(|| async { StatusCode::NOT_IMPLEMENTED.into_response() }),
        )
}

#[tracing::instrument(skip(app_state))]
async fn create_one(
    State(app_state): State<AppState>,
    user_id: UserId,
    Json(document): Json<Document<GameAttributes>>,
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

            let document = Document {
                data: Resources::Individual(Resource {
                    id: game.id.0.to_string().into(),
                    type_: "game".to_string().into(),
                    attributes: GameAttributes {
                        name: game.name.into(),
                    }
                    .into(),
                    links: None,
                    relationships: Relationships(
                        [(
                            "host".to_string(),
                            Relationship {
                                data: ResourceIdentifiers::Individual(ResourceIdentifier {
                                    id: "user".to_string().into(),
                                    type_: game.host.0.to_string().into(),
                                })
                                .into(),
                                links: Links(
                                    [
                                        (
                                            "self".to_string(),
                                            format!("{PATH}/{}/relationships/host", game.id.0),
                                        ),
                                        (
                                            "related".to_string(),
                                            format!("{PATH}/{}/host", game.host.0),
                                        ),
                                    ]
                                    .into(),
                                )
                                .into(),
                            },
                        )]
                        .into(),
                    )
                    .into(),
                })
                .into(),
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
    let game = Database::select_game(&app_state.postgres_pool, game_id).await?;

    if let Some(game) = game {
        let document = Document {
            data: Resources::Individual(Resource {
                id: game.id.0.to_string().into(),
                type_: "game".to_string().into(),
                attributes: GameAttributes {
                    name: game.name.into(),
                }
                .into(),
                links: None,
                relationships: Relationships(
                    [(
                        "host".to_string(),
                        Relationship {
                            data: ResourceIdentifiers::Individual(ResourceIdentifier {
                                id: "user".to_string().into(),
                                type_: game.host.0.to_string().into(),
                            })
                            .into(),
                            links: Links(
                                [
                                    (
                                        "self".to_string(),
                                        format!("{PATH}/{}/relationships/host", game.id.0),
                                    ),
                                    (
                                        "related".to_string(),
                                        format!("{PATH}/{}/host", game.host.0),
                                    ),
                                ]
                                .into(),
                            )
                            .into(),
                        },
                    )]
                    .into(),
                )
                .into(),
            })
            .into(),
            errors: None,
            links: Links([("self".to_string(), format!("{PATH}/{}", game.id.0))].into()).into(),
        };

        Ok((StatusCode::OK, Json(document)).into_response())
    } else {
        let document = Document::<()> {
            data: None,
            errors: Errors(vec![jsonapi::Error {
                status: 404,
                source: None,
                title: "Not Found".to_string().into(),
                detail: format!("Game {} does not exist", game_id.0).into(),
            }])
            .into(),
            links: None,
        };

        Ok((StatusCode::NOT_FOUND, Json(document)).into_response())
    }
}

#[tracing::instrument(skip(app_state))]
async fn get_many(
    State(app_state): State<AppState>,
    local_id: LocalId,
    Query(pagination): Query<Pagination>,
) -> Result<Response, ApiError> {
    let keyset_pagination = pagination.try_into()?;

    let (games, after) = Database::query_game(&app_state.postgres_pool, keyset_pagination).await?;

    let document = Document {
        data: Resources::Collection(
            games
                .into_iter()
                .map(|game| Resource {
                    id: game.id.0.to_string().into(),
                    type_: "game".to_string().into(),
                    attributes: GameAttributes {
                        name: game.name.into(),
                    }
                    .into(),
                    links: Links([("self".to_string(), format!("{PATH}/{}", game.id.0))].into())
                        .into(),
                    relationships: Relationships(
                        [(
                            "host".to_string(),
                            Relationship {
                                data: ResourceIdentifiers::Individual(ResourceIdentifier {
                                    id: "user".to_string().into(),
                                    type_: game.host.0.to_string().into(),
                                })
                                .into(),
                                links: Links(
                                    [
                                        (
                                            "self".to_string(),
                                            format!("{PATH}/{}/relationships/host", game.id.0),
                                        ),
                                        (
                                            "related".to_string(),
                                            format!("{PATH}/{}/host", game.host.0),
                                        ),
                                    ]
                                    .into(),
                                )
                                .into(),
                            },
                        )]
                        .into(),
                    )
                    .into(),
                })
                .collect(),
        )
        .into(),
        errors: None,
        links: Links(
            [
                (
                    "self".to_string(),
                    format!(
                        "/api/v1/games?page[after]={}&page[size]={}",
                        keyset_pagination.id, keyset_pagination.limit
                    ),
                ),
                (
                    "next".to_string(),
                    format!(
                        "/api/v1/games?page[after]={}&page[size]={}",
                        after, keyset_pagination.limit
                    ),
                ),
            ]
            .into(),
        )
        .into(),
    };

    Ok((StatusCode::OK, Json(document)).into_response())
}

#[tracing::instrument(skip(app_state))]
async fn update_one(
    State(app_state): State<AppState>,
    user_id: UserId,
    Path(game_id): Path<GameId>,
    Json(document): Json<Document<GameAttributes>>,
) -> Result<Response, ApiError> {
    let Some(mut game) = Database::select_game(&app_state.postgres_pool, game_id).await? else {
        let document = Document::<()> {
            data: None,
            errors: Errors(vec![jsonapi::Error {
                status: 404,
                source: None,
                title: "Not Found".to_string().into(),
                detail: format!("Game {} does not exist", game_id.0).into(),
            }])
            .into(),
            links: None,
        };

        return Ok((StatusCode::NOT_FOUND, Json(document)).into_response());
    };

    match document.try_get_resources()? {
        Resources::Collection(_) => Ok(StatusCode::NOT_IMPLEMENTED.into_response()),
        Resources::Individual(resource) => {
            if let Some(attributes) = &resource.attributes {
                game = game.update_attributes(attributes);
                Database::update_game(&app_state.postgres_pool, &game).await?;
            };

            let document = Document {
                data: Resources::Individual(Resource {
                    id: game.id.0.to_string().into(),
                    type_: "game".to_string().into(),
                    attributes: GameAttributes {
                        name: game.name.into(),
                    }
                    .into(),
                    links: None,
                    relationships: Relationships(
                        [(
                            "host".to_string(),
                            Relationship {
                                data: ResourceIdentifiers::Individual(ResourceIdentifier {
                                    id: "user".to_string().into(),
                                    type_: game.host.0.to_string().into(),
                                })
                                .into(),
                                links: Links(
                                    [
                                        (
                                            "self".to_string(),
                                            format!("{PATH}/{}/relationships/host", game.id.0),
                                        ),
                                        (
                                            "related".to_string(),
                                            format!("{PATH}/{}/host", game.host.0),
                                        ),
                                    ]
                                    .into(),
                                )
                                .into(),
                            },
                        )]
                        .into(),
                    )
                    .into(),
                })
                .into(),
                errors: None,
                links: Links([("self".to_string(), format!("{PATH}/{}", game.id.0))].into()).into(),
            };

            Ok((StatusCode::OK, Json(document)).into_response())
        }
    }
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
