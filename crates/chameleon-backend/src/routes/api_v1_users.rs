use axum::{
    extract::{Path, State},
    http::{header::LOCATION, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use chameleon_protocol::{
    attributes::UserAttributes,
    jsonapi::{self, Errors, Links, Resource, Resources, ResourcesDocument, Source},
};

use crate::{
    database::Database,
    domain::{LocalId, User, UserId},
    error::ApiError,
    jsonapi::{ToResource, Variation},
    AppState,
};

pub const PATH: &str = "/api/v1/users";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_one))
        .route("/:id", get(get_one))
        .route("/:id", patch(update_one))
}

#[tracing::instrument(skip(state))]
async fn create_one(
    State(state): State<AppState>,
    local_id: LocalId,
    Json(document): Json<ResourcesDocument<UserAttributes>>,
) -> Result<Response, ApiError> {
    if Database::select_user_id_by_local_id(&state.postgres_pool, local_id)
        .await?
        .is_some()
    {
        return Err(ApiError::JsonApi(
            jsonapi::Error {
                status: 403,
                source: Source {
                    header: "x-chameleon-local-id".to_string().into(),
                    parameter: None,
                    pointer: None,
                }
                .into(),
                title: "Forbidden".to_string().into(),
                detail: "`x-chameleon-local-id` is already associated with a user"
                    .to_string()
                    .into(),
            }
            .into(),
        ));
    }

    let user = User {
        id: UserId::random(),
        name: document
            .try_get_resources()?
            .try_get_individual()?
            .try_get_attribute(|a| a.name.as_ref(), "name", "Name")?
            .clone(),
    };

    let mut conn = state.postgres_pool.begin().await?;
    Database::insert_user(&mut conn, &user).await?;
    Database::insert_local(&mut conn, local_id, user.id).await?;
    conn.commit().await?;

    let document = ResourcesDocument {
        data: Resources::Individual(Resource {
            id: user.id.0.to_string().into(),
            type_: "user".to_string().into(),
            attributes: UserAttributes {
                name: user.name.into(),
            }
            .into(),
            links: None,
            relationships: None,
        })
        .into(),
        errors: None,
        links: Links([("self".to_string(), format!("{PATH}/{}", user.id.0))].into()).into(),
    };

    Ok((
        StatusCode::CREATED,
        [(LOCATION, format!("{PATH}/{}", user.id.0))],
        Json(document),
    )
        .into_response())
}

#[tracing::instrument(skip(state))]
async fn get_one(
    State(state): State<AppState>,
    local_id: LocalId,
    Path(id): Path<UserId>,
) -> Result<Response, ApiError> {
    let user = Database::select_user(&state.postgres_pool, id).await?;

    if let Some(user) = user {
        let document = ResourcesDocument {
            data: Resources::Individual(user.to_resource(Variation::Root)).into(),
            errors: None,
            links: Links([("self".to_string(), format!("{PATH}/{}", user.id.0))].into()).into(),
        };

        Ok((StatusCode::OK, Json(document)).into_response())
    } else {
        let document = ResourcesDocument::<()> {
            data: None,
            errors: Errors(vec![jsonapi::Error {
                status: 404,
                source: None,
                title: "Not Found".to_string().into(),
                detail: format!("User {} does not exist", id.0).into(),
            }])
            .into(),
            links: None,
        };

        Ok((StatusCode::NOT_FOUND, Json(document)).into_response())
    }
}

#[tracing::instrument(skip(state))]
async fn update_one(
    State(state): State<AppState>,
    user_id: UserId,
    Path(id): Path<UserId>,
    Json(document): Json<ResourcesDocument<UserAttributes>>,
) -> Result<Response, ApiError> {
    if user_id != id {
        return Err(ApiError::JsonApi(
            jsonapi::Error {
                status: 403,
                source: None,
                title: "Forbidden".to_string().into(),
                detail: "You do not have sufficient permissions to update this user"
                    .to_string()
                    .into(),
            }
            .into(),
        ));
    }

    let Some(mut user) = Database::select_user(&state.postgres_pool, id).await? else {
        return Err(ApiError::JsonApi(jsonapi::Error {
            status: 404,
            source: None,
            title: "Not Found".to_string().into(),
            detail: "User does not exist".to_string().into()
        }.into()));
    };

    let resource = document.try_get_resources()?.try_get_individual()?;

    if let Some(attributes) = &resource.attributes {
        user = user.update_attributes(attributes);
        Database::update_user(&state.postgres_pool, &user).await?;
    }

    let document = ResourcesDocument {
        data: Resources::Individual(user.to_resource(Variation::Root)).into(),
        errors: None,
        links: Links([("self".to_string(), format!("{PATH}/{}", user.id.0))].into()).into(),
    };

    Ok((StatusCode::OK, Json(document)).into_response())
}

impl User {
    pub fn update_attributes(&self, attributes: &UserAttributes) -> Self {
        Self {
            id: self.id,
            name: attributes.name.as_ref().unwrap_or(&self.name).clone(),
        }
    }
}
