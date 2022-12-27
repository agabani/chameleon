use chameleon_protocol::attributes::UserAttributes;

use crate::domain::{User, UserId};

use super::{ToResource, ToResourceIdentifier, Variation};

const TYPE: &str = "user";

impl ToResource for User {
    const TYPE: &'static str = TYPE;

    type Attributes = UserAttributes;

    fn __attributes(&self) -> Option<Self::Attributes> {
        Some(Self::Attributes {
            name: Some(self.name.to_string()),
        })
    }

    fn __id(&self) -> String {
        self.id.0.to_string()
    }

    fn __relationships(
        &self,
        _variation: Variation,
    ) -> Option<chameleon_protocol::jsonapi::Relationships> {
        None
    }
}

impl ToResourceIdentifier for UserId {
    const TYPE: &'static str = TYPE;

    fn __id(&self) -> String {
        self.0.to_string()
    }
}
