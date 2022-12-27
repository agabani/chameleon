use chameleon_protocol::{
    attributes::GameAttributes,
    jsonapi::{Links, Relationship, Relationships, ResourceIdentifier, ResourceIdentifiers},
};

use crate::domain::Game;

use super::{ToResource, Variation};

impl ToResource for Game {
    type Attributes = GameAttributes;

    fn to_resource(
        &self,
        variation: Variation,
    ) -> chameleon_protocol::jsonapi::Resource<Self::Attributes> {
        chameleon_protocol::jsonapi::Resource {
            id: self.id.0.to_string().into(),
            type_: "game".to_string().into(),
            attributes: Self::Attributes {
                name: self.name.clone().into(),
            }
            .into(),
            links: match variation {
                Variation::Nested(base) => {
                    Links([("self".to_string(), format!("{base}/{}", self.id.0))].into()).into()
                }
                Variation::Root(_) => None,
            },
            relationships: Relationships(
                [(
                    "host".to_string(),
                    Relationship {
                        data: ResourceIdentifiers::Individual(ResourceIdentifier {
                            id: self.host.0.to_string().into(),
                            type_: "user".to_string().into(),
                        })
                        .into(),
                        links: Links(
                            [
                                (
                                    "self".to_string(),
                                    match variation {
                                        Variation::Nested(base) | Variation::Root(base) => {
                                            format!("{base}/{}/relationships/host", self.id.0)
                                        }
                                    },
                                ),
                                (
                                    "related".to_string(),
                                    match variation {
                                        Variation::Nested(base) | Variation::Root(base) => {
                                            format!("{base}/{}/host", self.id.0)
                                        }
                                    },
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
        }
    }
}
