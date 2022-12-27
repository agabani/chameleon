use chameleon_protocol::jsonapi::Resource;

mod game;

pub trait ToJsonApi {
    type Attributes;

    fn to_resource(&self, variation: Variation) -> Resource<Self::Attributes>;
}

#[derive(Debug, Clone, Copy)]
pub enum Variation<'a> {
    Individual(&'a str),
    Collection(&'a str),
}
