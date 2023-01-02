use chameleon_protocol::jsonapi::{Links, Relationships, Resource, ResourceIdentifier};

pub mod api_v1_lobbies;
pub mod api_v1_ping;
pub mod api_v1_userinfo;
pub mod api_v1_users;
pub mod ws_v1_lobbies;

trait ToResource {
    const PATH: &'static str;

    const TYPE: &'static str;

    type Attributes;

    fn to_resource(&self, variation: Variation) -> Resource<Self::Attributes> {
        Resource {
            id: Some(self.__id()),
            type_: Some(self.__type()),
            attributes: self.__attributes(),
            links: self.__links(variation),
            relationships: self.__relationships(),
        }
    }

    fn __attributes(&self) -> Option<Self::Attributes>;

    fn __id(&self) -> String;

    fn __links(&self, variation: Variation) -> Option<Links> {
        match variation {
            Variation::Nested => Some(Links(
                [(
                    "self".to_string(),
                    format!("{}/{}", Self::PATH, self.__id()),
                )]
                .into(),
            )),
            Variation::Root => None,
        }
    }

    fn __relationships(&self) -> Option<Relationships>;

    fn __type(&self) -> String {
        Self::TYPE.to_string()
    }
}

trait ToResourceIdentifier {
    const TYPE: &'static str;

    fn to_resource_identifier(&self) -> ResourceIdentifier {
        ResourceIdentifier {
            id: Some(self.__id()),
            type_: Some(self.__type()),
        }
    }

    fn __id(&self) -> String;

    fn __type(&self) -> String {
        Self::TYPE.to_string()
    }
}

#[derive(Debug, Clone, Copy)]
enum Variation {
    Nested,
    Root,
}
