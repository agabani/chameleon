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
    database::Database,
    domain::{LocalId, User, UserId},
    error::ApiError,
    AppState,
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
    local_id: LocalId,
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
    local_id: LocalId,
    Path(id): Path<UserId>,
) -> Result<Response, ApiError> {
    let user = Database::select_user(&state.postgres_pool, id)
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
    user_id: UserId,
    Path(id): Path<UserId>,
    Json(document): Json<ResourcesDocument<UserAttributes>>,
) -> Result<Response, ApiError> {
    if user_id != id {
        return Err(ApiError::JsonApi(Box::new(jsonapi::Error {
            status: 403,
            source: None,
            title: Some("Forbidden".to_string()),
            detail: Some("You do not have sufficient permissions to update this user".to_string()),
        })));
    }

    let mut user = Database::select_user(&state.postgres_pool, id)
        .await?
        .ok_or_else(|| ApiError::JsonApi(Box::new(jsonapi::Error::not_found("user", "User"))))?;

    let resource = document.try_get_resources()?.try_get_individual()?;

    if let Some(attributes) = &resource.attributes {
        user = user.update_attributes(attributes);
        Database::update_user(&state.postgres_pool, &user).await?;
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

impl User {
    pub fn update_attributes(&self, attributes: &UserAttributes) -> Self {
        Self {
            id: self.id,
            name: attributes.name.as_ref().unwrap_or(&self.name).clone(),
        }
    }
}

impl ToResource for User {
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

impl ToResourceIdentifier for UserId {
    const TYPE: &'static str = TYPE;

    fn __id(&self) -> String {
        self.0.to_string()
    }
}
