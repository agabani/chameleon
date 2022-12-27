use chameleon_protocol::jsonapi::{Links, Relationships, Resource, ResourceIdentifier};

mod game;
mod user;

pub trait ToResource {
    const TYPE: &'static str;

    type Attributes;

    fn to_resource(&self, variation: Variation) -> Resource<Self::Attributes> {
        Resource {
            id: self.__id().into(),
            type_: self.__type().into(),
            attributes: self.__attributes(),
            links: self.__links(variation),
            relationships: self.__relationships(variation),
        }
    }

    fn __attributes(&self) -> Option<Self::Attributes>;

    fn __id(&self) -> String;

    fn __links(&self, variation: Variation) -> Option<Links> {
        match variation {
            Variation::Nested(path) => {
                Links([("self".to_string(), format!("{path}/{}", self.__id()))].into()).into()
            }
            Variation::Root(_) => None,
        }
    }

    fn __relationships(&self, variation: Variation) -> Option<Relationships>;

    fn __type(&self) -> String {
        Self::TYPE.to_string()
    }
}

pub trait ToResourceIdentifier {
    const TYPE: &'static str;

    fn to_resource_identifier(&self) -> ResourceIdentifier {
        ResourceIdentifier {
            id: self.__id().into(),
            type_: self.__type().into(),
        }
    }

    fn __id(&self) -> String;

    fn __type(&self) -> String {
        Self::TYPE.to_string()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Variation<'a> {
    Nested(&'a str),
    Root(&'a str),
}
