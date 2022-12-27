use chameleon_protocol::{
    attributes::UserAttributes,
    jsonapi::{Links, Resource, ResourceIdentifier},
};

use crate::domain::{User, UserId};

use super::{ToResource, ToResourceIdentifier, Variation};

impl ToResource for User {
    type Attributes = UserAttributes;

    fn to_resource(&self, variation: Variation) -> Resource<Self::Attributes> {
        Resource {
            id: self.id.0.to_string().into(),
            type_: "user".to_string().into(),
            attributes: Self::Attributes {
                name: self.name.to_string().into(),
            }
            .into(),
            links: match variation {
                Variation::Nested(path) => {
                    Links([("self".to_string(), format!("{path}/{}", self.id.0))].into()).into()
                }
                Variation::Root(_) => None,
            },
            relationships: None,
        }
    }
}

impl ToResourceIdentifier for UserId {
    fn to_resource_identifier(&self) -> ResourceIdentifier {
        ResourceIdentifier {
            id: self.0.to_string().into(),
            type_: "user".to_string().into(),
        }
    }
}
