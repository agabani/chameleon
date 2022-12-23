use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Document<T> {
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<Resources<T>>,

    #[serde(rename = "errors", skip_serializing_if = "Option::is_none")]
    pub errors: Option<Errors>,

    #[serde(rename = "links", skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Error {
    #[serde(rename = "status")]
    pub status: u16,

    #[serde(rename = "source", skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,

    #[serde(rename = "title", skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(rename = "detail", skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Errors(pub Vec<Error>);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Links(pub HashMap<String, String>);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pagination {
    #[serde(rename = "page[after]")]
    pub after: Option<String>,

    #[serde(rename = "page[before]")]
    pub before: Option<String>,

    #[serde(rename = "page[size]")]
    pub size: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Resource<T> {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,

    #[serde(rename = "attributes", skip_serializing_if = "Option::is_none")]
    pub attributes: Option<T>,

    #[serde(rename = "links", skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,

    #[serde(rename = "relationships", skip_serializing_if = "Option::is_none")]
    pub relationships: Option<Relationships>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceIdentifier {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ResourceIdentifiers {
    Collection(Vec<ResourceIdentifier>),
    Individual(ResourceIdentifier),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Resources<T> {
    Collection(Vec<Resource<T>>),
    Individual(Resource<T>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Relationship {
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<ResourceIdentifiers>,

    #[serde(rename = "links", skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Relationships(pub HashMap<String, Relationship>);

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Source {
    #[serde(rename = "header", skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,

    #[serde(rename = "parameter", skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,

    #[serde(rename = "pointer", skip_serializing_if = "Option::is_none")]
    pub pointer: Option<String>,
}

impl<T> Document<T> {
    #[allow(clippy::result_large_err)]
    pub fn try_get_resources(&self) -> Result<&Resources<T>, Error> {
        self.data.as_ref().ok_or_else(|| Error {
            status: 422,
            source: Source {
                header: None,
                parameter: None,
                pointer: "/data".to_string().into(),
            }
            .into(),
            title: "Invalid Member".to_string().into(),
            detail: "Data must be present".to_string().into(),
        })
    }
}

impl<T> Resource<T> {
    #[allow(clippy::result_large_err)]
    pub fn try_get_attribute<A>(
        &self,
        accessor: impl Fn(&T) -> Option<&A>,
        name: &str,
        display: &str,
    ) -> Result<&A, Error> {
        self.attributes
            .as_ref()
            .and_then(accessor)
            .ok_or_else(|| Error {
                status: 422,
                source: Source {
                    header: None,
                    parameter: None,
                    pointer: format!("/data/attributes/{name}").into(),
                }
                .into(),
                title: "Invalid Attribute".to_string().into(),
                detail: format!("{display} must be present").into(),
            })
    }

    #[allow(clippy::result_large_err)]
    pub fn try_get_field<A>(
        &self,
        accessor: impl Fn(&Self) -> Option<&A>,
        name: &str,
        display: &str,
    ) -> Result<&A, Error> {
        accessor(self).ok_or_else(|| Error {
            status: 422,
            source: Source {
                header: None,
                parameter: None,
                pointer: format!("/data/{name}").into(),
            }
            .into(),
            title: "Invalid Field".to_string().into(),
            detail: format!("{display} must be present").into(),
        })
    }
}

impl<T> Resources<T> {
    #[allow(clippy::result_large_err)]
    pub fn try_get_individual(&self) -> Result<&Resource<T>, Error> {
        match self {
            Resources::Collection(_) => Err(Error {
                status: 422,
                source: Source {
                    header: None,
                    parameter: None,
                    pointer: "/data".to_string().into(),
                }
                .into(),
                title: "Invalid Member".to_string().into(),
                detail: "Data must be a resource object".to_string().into(),
            }),
            Resources::Individual(resource) => Ok(resource),
        }
    }
}
