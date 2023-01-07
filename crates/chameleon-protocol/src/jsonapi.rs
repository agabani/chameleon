use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Errors(pub Vec<Error>);

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Links(pub HashMap<String, String>);

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Pagination {
    #[serde(rename = "page[after]")]
    pub after: Option<String>,

    #[serde(rename = "page[before]")]
    pub before: Option<String>,

    #[serde(rename = "page[size]")]
    pub size: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ResourceIdentifier {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ResourceIdentifiers {
    Collection(Vec<ResourceIdentifier>),
    Individual(ResourceIdentifier),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ResourceIdentifiersDocument {
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<ResourceIdentifiers>,

    #[serde(rename = "errors", skip_serializing_if = "Option::is_none")]
    pub errors: Option<Errors>,

    #[serde(rename = "links", skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Resources<T> {
    Collection(Vec<Resource<T>>),
    Individual(Resource<T>),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ResourcesDocument<T> {
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<Resources<T>>,

    #[serde(rename = "errors", skip_serializing_if = "Option::is_none")]
    pub errors: Option<Errors>,

    #[serde(rename = "links", skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Relationship {
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<ResourceIdentifiers>,

    #[serde(rename = "links", skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Relationships(pub HashMap<String, Relationship>);

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Source {
    #[serde(rename = "header", skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,

    #[serde(rename = "parameter", skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,

    #[serde(rename = "pointer", skip_serializing_if = "Option::is_none")]
    pub pointer: Option<String>,
}

impl Error {
    pub fn forbidden() -> Error {
        Error {
            status: 403,
            source: None,
            title: Some("Forbidden".to_string()),
            detail: Some("You are not authorized to perform this action".to_string()),
        }
    }

    pub fn not_found(_name: &str, display: &str) -> Error {
        Error {
            status: 404,
            source: None,
            title: Some("Not Found".to_string()),
            detail: Some(format!("{display} does not exist")),
        }
    }
}

impl Relationship {
    pub fn try_get_resource_identifiers(
        &self,
        name: &str,
    ) -> Result<&ResourceIdentifiers, Box<Error>> {
        self.data.as_ref().ok_or_else(|| {
            Box::new(Error {
                status: 422,
                source: Some(Source {
                    header: None,
                    parameter: None,
                    pointer: Some(format!("/data/relationship/{name}/data")),
                }),
                title: Some("Invalid Member".to_string()),
                detail: Some("Data must be present".to_string()),
            })
        })
    }
}

impl<T> Resource<T> {
    pub fn try_get_attribute<A>(
        &self,
        accessor: impl Fn(&T) -> Option<&A>,
        name: &str,
        display: &str,
    ) -> Result<&A, Box<Error>> {
        self.attributes.as_ref().and_then(accessor).ok_or_else(|| {
            Box::new(Error {
                status: 422,
                source: Some(Source {
                    header: None,
                    parameter: None,
                    pointer: Some(format!("/data/attributes/{name}")),
                }),
                title: Some("Invalid Attribute".to_string()),
                detail: Some(format!("{display} must be present")),
            })
        })
    }

    pub fn try_get_field<A>(
        &self,
        accessor: impl Fn(&Self) -> Option<&A>,
        name: &str,
        display: &str,
    ) -> Result<&A, Box<Error>> {
        accessor(self).ok_or_else(|| {
            Box::new(Error {
                status: 422,
                source: Some(Source {
                    header: None,
                    parameter: None,
                    pointer: Some(format!("/data/{name}")),
                }),
                title: Some("Invalid Field".to_string()),
                detail: Some(format!("{display} must be present")),
            })
        })
    }

    pub fn try_get_relationship(
        &self,
        name: &str,
        display: &str,
    ) -> Result<&Relationship, Box<Error>> {
        self.relationships
            .as_ref()
            .and_then(|r| r.0.get(name))
            .ok_or_else(|| {
                Box::new(Error {
                    status: 422,
                    source: Some(Source {
                        header: None,
                        parameter: None,
                        pointer: Some(format!("/data/relationships/{name}")),
                    }),
                    title: Some("Invalid Attribute".to_string()),
                    detail: Some(format!("{display} must be present")),
                })
            })
    }
}

impl ResourceIdentifier {
    pub fn try_get_field<A>(
        &self,
        accessor: impl Fn(&Self) -> Option<&A>,
        name: &str,
        display: &str,
    ) -> Result<&A, Box<Error>> {
        accessor(self).ok_or_else(|| {
            Box::new(Error {
                status: 422,
                source: Some(Source {
                    header: None,
                    parameter: None,
                    pointer: Some(format!("/data/relationships/*/data/{name}")),
                }),
                title: Some("Invalid Field".to_string()),
                detail: Some(format!("{display} must be present")),
            })
        })
    }
}

impl ResourceIdentifiers {
    pub fn try_get_collection(
        &self,
        name: &str,
        display: &str,
    ) -> Result<&Vec<ResourceIdentifier>, Box<Error>> {
        match self {
            ResourceIdentifiers::Collection(resources) => Ok(resources),
            ResourceIdentifiers::Individual(_) => Err(Box::new(Error {
                status: 422,
                source: Some(Source {
                    header: None,
                    parameter: None,
                    pointer: Some(format!("/data/relationships/{name}")),
                }),
                title: Some("Invalid Relationship".to_string()),
                detail: Some(format!("{display} must be a resource identifier array")),
            })),
        }
    }

    pub fn try_get_individual(
        &self,
        name: &str,
        display: &str,
    ) -> Result<&ResourceIdentifier, Box<Error>> {
        match self {
            ResourceIdentifiers::Collection(_) => Err(Box::new(Error {
                status: 422,
                source: Some(Source {
                    header: None,
                    parameter: None,
                    pointer: Some(format!("/data/relationships/{name}")),
                }),
                title: Some("Invalid Relationship".to_string()),
                detail: Some(format!("{display} must be a resource identifier object")),
            })),
            ResourceIdentifiers::Individual(resource) => Ok(resource),
        }
    }
}

impl ResourceIdentifiersDocument {
    pub fn internal_server_error() -> ResourceIdentifiersDocument {
        ResourceIdentifiersDocument {
            data: None,
            errors: Some(Errors(vec![Error {
                status: 500,
                source: None,
                title: Some("Internal Server Error".to_string()),
                detail: Some("An unexpected error has occurred".to_string()),
            }])),
            links: None,
        }
    }
}

impl<T> Resources<T> {
    pub fn try_get_collection(&self) -> Result<&Vec<Resource<T>>, Box<Error>> {
        match self {
            Resources::Collection(resources) => Ok(resources),
            Resources::Individual(_) => Err(Box::new(Error {
                status: 422,
                source: Some(Source {
                    header: None,
                    parameter: None,
                    pointer: Some("/data".to_string()),
                }),
                title: Some("Invalid Member".to_string()),
                detail: Some("Data must be a resource array".to_string()),
            })),
        }
    }

    pub fn try_get_individual(&self) -> Result<&Resource<T>, Box<Error>> {
        match self {
            Resources::Collection(_) => Err(Box::new(Error {
                status: 422,
                source: Some(Source {
                    header: None,
                    parameter: None,
                    pointer: Some("/data".to_string()),
                }),
                title: Some("Invalid Member".to_string()),
                detail: Some("Data must be a resource object".to_string()),
            })),
            Resources::Individual(resource) => Ok(resource),
        }
    }
}

impl<T> ResourcesDocument<T> {
    pub fn forbidden() -> ResourcesDocument<T> {
        ResourcesDocument {
            data: None,
            errors: Some(Errors(vec![Error {
                status: 403,
                source: None,
                title: Some("Forbidden".to_string()),
                detail: Some("You are not authorized to perform this action".to_string()),
            }])),
            links: None,
        }
    }

    pub fn internal_server_error() -> ResourcesDocument<T> {
        ResourcesDocument {
            data: None,
            errors: Some(Errors(vec![Error {
                status: 500,
                source: None,
                title: Some("Internal Server Error".to_string()),
                detail: Some("An unexpected error has occurred".to_string()),
            }])),
            links: None,
        }
    }

    pub fn unauthorized() -> ResourcesDocument<T> {
        ResourcesDocument {
            data: None,
            errors: Some(Errors(vec![Error {
                status: 401,
                source: Some(Source {
                    header: Some("x-chameleon-local-id".to_string()),
                    parameter: None,
                    pointer: None,
                }),
                title: "Invalid Header".to_string().into(),
                detail: Some("`x-chameleon-local-id` does not have a user".to_string()),
            }])),
            links: None,
        }
    }

    pub fn try_get_link(&self, name: &str, display: &str) -> Result<&String, Box<Error>> {
        self.links
            .as_ref()
            .and_then(|links| links.0.get(name))
            .ok_or_else(|| {
                Box::new(Error {
                    status: 422,
                    source: Some(Source {
                        header: None,
                        parameter: None,
                        pointer: Some(format!("/links/{name}")),
                    }),
                    title: Some("Invalid Link".to_string()),
                    detail: Some(format!("{display} must be present")),
                })
            })
    }

    pub fn try_get_resources(&self) -> Result<&Resources<T>, Box<Error>> {
        self.data.as_ref().ok_or_else(|| {
            Box::new(Error {
                status: 422,
                source: Some(Source {
                    header: None,
                    parameter: None,
                    pointer: Some("/data".to_string()),
                }),
                title: Some("Invalid Member".to_string()),
                detail: Some("Data must be present".to_string()),
            })
        })
    }
}

/// Convenience method.
impl<T> ResourcesDocument<T> {
    /// Try get attribute of individual resource.
    ///
    /// Convenience method.
    pub fn try_get_attribute<A>(
        &self,
        accessor: impl Fn(&T) -> Option<&A>,
        name: &str,
        display: &str,
    ) -> Result<&A, Box<Error>> {
        self.try_get_resources()?
            .try_get_individual()?
            .try_get_attribute(accessor, name, display)
    }

    pub fn try_get_collection_resources(&self) -> Result<&Vec<Resource<T>>, Box<Error>> {
        self.try_get_resources()?.try_get_collection()
    }

    /// Try get field of individual resource.
    ///
    /// Convenience method.
    pub fn try_get_field<A>(
        &self,
        accessor: impl Fn(&Resource<T>) -> Option<&A>,
        name: &str,
        display: &str,
    ) -> Result<&A, Box<Error>> {
        self.try_get_resources()?
            .try_get_individual()?
            .try_get_field(accessor, name, display)
    }

    /// Try get relationship of individual resource.
    ///
    /// Convenience method.
    pub fn try_get_relationship<A>(
        &self,
        name: &str,
        display: &str,
    ) -> Result<&Relationship, Box<Error>> {
        self.try_get_resources()?
            .try_get_individual()?
            .try_get_relationship(name, display)
    }
}
