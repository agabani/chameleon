use chameleon_protocol::jsonapi::Resource;

mod game;
mod user;

pub trait ToJsonApi {
    type Attributes;

    fn to_resource(&self, variation: Variation) -> Resource<Self::Attributes>;
}

#[derive(Debug, Clone, Copy)]
pub enum Variation<'a> {
    Collection(&'a str),
    Individual(&'a str),
}
