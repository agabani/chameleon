use chameleon_protocol::{
    attributes::UserAttributes,
    jsonapi::{Links, Resource},
};

use crate::domain::User;

use super::{ToJsonApi, Variation};

impl ToJsonApi for User {
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
                Variation::Collection(path) => {
                    Links([("self".to_string(), format!("{path}/{}", self.id.0))].into()).into()
                }
                Variation::Individual(_) => None,
            },
            relationships: None,
        }
    }
}
