use chameleon_protocol::http::{self, UserResponse};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::services::Service;

pub struct TestUser {
    input: NodeRef,
    communicating: bool,
    user: Option<UserResponse>,
}

pub enum Msg {
    UserUpdated(UserResponse),
    SubmitClicked,
}

impl Component for TestUser {
    type Message = Msg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let service = Service::from_context(ctx);

        ctx.link().send_future_batch(async move {
            let user = service
                .api
                .get_user()
                .await
                .expect("Expected successful API response");

            if let Some(user) = user {
                vec![Msg::UserUpdated(user)]
            } else {
                vec![]
            }
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
                    <li>{ "user id: "} { id }</li>
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
            Msg::UserUpdated(user) => {
                self.communicating = false;
                self.input
                    .cast::<HtmlInputElement>()
                    .expect("Failed to find input")
                    .set_value("");
                self.user = Some(user);
                true
            }
            Msg::SubmitClicked => {
                self.communicating = true;
                let registered = self.user.is_some();
                let value = self
                    .input
                    .cast::<HtmlInputElement>()
                    .expect("Failed to find input")
                    .value();

                let service = Service::from_context(ctx);

                ctx.link().send_future(async move {
                    if registered {
                        service
                            .api
                            .put_user(&http::UserRequest { name: value })
                            .await
                            .expect("Failed to update user");
                    } else {
                        service
                            .api
                            .signup(&http::UserRequest { name: value })
                            .await
                            .expect("Failed to update user");
                    }

                    let user = service
                        .api
                        .get_user()
                        .await
                        .expect("Failed to get user")
                        .expect("Failed to get user");

                    Msg::UserUpdated(user)
                });

                true
            }
        }
    }
}
