use axum::{
    extract::{Path, State},
    http::{header::LOCATION, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use chameleon_protocol::{
    attributes::UserAttributes,
    jsonapi::{self, Links, Resources, ResourcesDocument, Source},
};

use crate::{
    app::AppState,
    database::Database,
    domain::{local_id, user, user_id},
    error::ApiError,
};

use super::{ToResource, ToResourceIdentifier, Variation};

pub const PATH: &str = "/api/v1/users";

const TYPE: &str = "user";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_one))
        .route("/:id", get(get_one))
        .route("/:id", patch(update_one))
}

#[tracing::instrument(skip(state))]
async fn create_one(
    State(state): State<AppState>,
    local_id: local_id::LocalId,
    Json(document): Json<ResourcesDocument<UserAttributes>>,
) -> Result<Response, ApiError> {
    if Database::select_user_id_by_local_id(&state.postgres_pool, local_id)
        .await?
        .is_some()
    {
        return Err(ApiError::JsonApi(Box::new(jsonapi::Error {
            status: 403,
            source: Some(Source {
                header: Some("x-chameleon-local-id".to_string()),
                parameter: None,
                pointer: None,
            }),
            title: Some("Forbidden".to_string()),
            detail: Some("`x-chameleon-local-id` is already associated with a user".to_string()),
        })));
    }

    let name = document
        .try_get_resources()?
        .try_get_individual()?
        .try_get_attribute(|a| a.name.as_ref(), "name", "Name")?;

    let user = match user::User::signup(local_id, name) {
        Ok((user, events)) => {
            Database::save_user(&state.postgres_pool, user.id, &events).await?;
            user
        }
        Err(error) => match error {},
    };

    let document = ResourcesDocument {
        data: Some(Resources::Individual(user.to_resource(Variation::Root))),
        errors: None,
        links: Some(Links(
            [("self".to_string(), format!("{PATH}/{}", user.id.0))].into(),
        )),
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
    local_id: local_id::LocalId,
    Path(id): Path<user_id::UserId>,
) -> Result<Response, ApiError> {
    let user = Database::load_user(&state.postgres_pool, id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("user", "User"))))?;

    let document = ResourcesDocument {
        data: Some(Resources::Individual(user.to_resource(Variation::Root))),
        errors: None,
        links: Some(Links(
            [("self".to_string(), format!("{PATH}/{}", user.id.0))].into(),
        )),
    };

    Ok((StatusCode::OK, Json(document)).into_response())
}

#[tracing::instrument(skip(state))]
async fn update_one(
    State(state): State<AppState>,
    user_id: user_id::UserId,
    Path(id): Path<user_id::UserId>,
    Json(document): Json<ResourcesDocument<UserAttributes>>,
) -> Result<Response, ApiError> {
    let mut user = Database::load_user(&state.postgres_pool, id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("user", "User"))))?;

    let name = document
        .try_get_attribute(|accessor| accessor.name.as_ref(), "name", "Name")
        .ok()
        .map(String::as_str);

    match user.update(user_id, name) {
        Ok(events) => {
            Database::save_user(&state.postgres_pool, user.id, &events).await?;
        }
        Err(error) => match error {
            user::UpdateError::NotOwner => {
                return Err(ApiError::JsonApi(Box::new(jsonapi::Error::forbidden())));
            }
        },
    }

    let document = ResourcesDocument {
        data: Some(Resources::Individual(user.to_resource(Variation::Root))),
        errors: None,
        links: Some(Links(
            [("self".to_string(), format!("{PATH}/{}", user.id.0))].into(),
        )),
    };

    Ok((StatusCode::OK, Json(document)).into_response())
}

impl ToResource for user::User {
    const PATH: &'static str = PATH;

    const TYPE: &'static str = TYPE;

    type Attributes = UserAttributes;

    fn __attributes(&self) -> Option<Self::Attributes> {
        Some(Self::Attributes {
            name: Some(self.name.to_string()),
        })
    }

    fn __id(&self) -> String {
        self.id.0.to_string()
    }

    fn __relationships(&self) -> Option<chameleon_protocol::jsonapi::Relationships> {
        None
    }
}

impl ToResourceIdentifier for user_id::UserId {
    const TYPE: &'static str = TYPE;

    fn __id(&self) -> String {
        self.0.to_string()
    }
}
