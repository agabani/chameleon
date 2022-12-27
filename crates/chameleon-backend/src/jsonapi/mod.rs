use chameleon_protocol::jsonapi::{Resource, ResourceIdentifier};

mod game;
mod user;

pub trait ToResource {
    type Attributes;

    fn to_resource(&self, variation: Variation) -> Resource<Self::Attributes>;
}

pub trait ToResourceIdentifier {
    fn to_resource_identifier(&self) -> ResourceIdentifier;
}

#[derive(Debug, Clone, Copy)]
pub enum Variation<'a> {
    Nested(&'a str),
    Root(&'a str),
}
