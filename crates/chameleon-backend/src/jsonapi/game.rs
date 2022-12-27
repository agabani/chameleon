use chameleon_protocol::{
    attributes::GameAttributes,
    jsonapi::{Links, Relationship, Relationships, ResourceIdentifiers},
};

use crate::domain::{Game, GameId};

use super::{ToResource, ToResourceIdentifier, Variation};

const TYPE: &str = "game";

impl ToResource for Game {
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

    fn __relationships(&self, variation: Variation) -> Option<Relationships> {
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
