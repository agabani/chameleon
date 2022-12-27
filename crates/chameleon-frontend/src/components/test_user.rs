use chameleon_protocol::{
    attributes::UserAttributes,
    jsonapi::{Resource, Resources, ResourcesDocument},
};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::services::Service;

pub struct TestUser {
    input: NodeRef,
    communicating: bool,
    user: Option<User>,
}

pub enum Msg {
    SubmitClicked,
    DocumentReceived(ResourcesDocument<UserAttributes>),
    UserUpdated(Option<User>),
}

pub struct User {
    id: String,
    name: String,
}

impl Component for TestUser {
    type Message = Msg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let service = Service::from_context(ctx);

        ctx.link().send_future_batch(async move {
            let Some(userinfo) = service
                .api
                .get_userinfo()
                .await
                .expect("Failed to get userinfo") else {
                    return vec![];
                };

            let document = service
                .api
                .get_user(&userinfo.sub)
                .await
                .expect("Failed to get user");

            vec![Msg::DocumentReceived(document)]
        });

        Self {
            input: NodeRef::default(),
            communicating: false,
            user: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onsubmit = ctx.link().callback(|event: SubmitEvent| {
            event.prevent_default();
            Msg::SubmitClicked
        });

        let id = self.user.as_ref().map_or("", |u| u.id.as_str());
        let name = self.user.as_ref().map_or("", |u| u.name.as_str());

        html! {
            <div>
                <h2>{ "Test User" }</h2>
                    <ul>
                    <li>{ "user id: "}{ id }</li>
                    <li>{ "user name: "}{ name }</li>
                </ul>
                <form onsubmit={ onsubmit }>
                    <input
                        disabled={self.communicating}
                        ref={ &self.input }
                        type="text" />
                    <button
                        disabled={self.communicating}
                        type="submit">
                        { match (&self.user, self.communicating) {
                            (Some(_), true) => "Updating",
                            (Some(_), false) => "Update",
                            (None, true) => "Registering",
                            (None, false) => "Register",
                        } }
                    </button>
                </form>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SubmitClicked => {
                self.communicating = true;

                let id = self.user.as_ref().map(|x| x.id.clone());
                let name = self
                    .input
                    .cast::<HtmlInputElement>()
                    .expect("Failed to find input")
                    .value();

                let document: ResourcesDocument<UserAttributes> = ResourcesDocument {
                    data: Resources::Individual(Resource {
                        id: id.clone(),
                        type_: "user".to_string().into(),
                        attributes: UserAttributes { name: name.into() }.into(),
                        links: None,
                        relationships: None,
                    })
                    .into(),
                    errors: None,
                    links: None,
                };

                let service = Service::from_context(ctx);

                ctx.link().send_future(async move {
                    let document = if let Some(id) = id {
                        service
                            .api
                            .update_user(&id, &document)
                            .await
                            .expect("Failed to update user")
                    } else {
                        service
                            .api
                            .create_user(&document)
                            .await
                            .expect("Failed to create user")
                    };

                    Msg::DocumentReceived(document)
                });

                true
            }
            Msg::DocumentReceived(document) => {
                self.communicating = false;

                let Some(resource) = document
                    .try_get_resources()
                    .and_then(Resources::try_get_individual)
                    .ok() else {
                        ctx.link().send_message(Msg::UserUpdated(None));
                        return false;
                    };

                let id = resource
                    .try_get_field(|a| a.id.as_ref(), "id", "Id")
                    .unwrap()
                    .clone();
                let name = resource
                    .try_get_attribute(|a| a.name.as_ref(), "name", "Name")
                    .unwrap()
                    .clone();

                ctx.link()
                    .send_message(Msg::UserUpdated(User { id, name }.into()));

                false
            }
            Msg::UserUpdated(user) => {
                self.user = user;
                true
            }
        }
    }
}
