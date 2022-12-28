use chameleon_protocol::{
    attributes::UserAttributes,
    jsonapi::{Resource, Resources, ResourcesDocument},
};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::services::Service;

pub struct UserSignup {
    input_field: NodeRef,
}

pub enum Msg {
    NetworkFailure(gloo::net::Error),
    NetworkSuccess(ResourcesDocument<UserAttributes>),
    SubmitClicked,
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub signup_success: Callback<()>,
}

impl Component for UserSignup {
    type Message = Msg;

    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            input_field: NodeRef::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onsubmit = ctx.link().callback(|event: SubmitEvent| {
            event.prevent_default();
            Msg::SubmitClicked
        });

        html! {
            <div class="user-signup">
                <form class="user-signup--form" {onsubmit}>
                    <div class="user-signup--input-group">
                        <label>{ "Username" }</label>
                        <input ref={ self.input_field.clone() } />
                    </div>
                    <div class="user-signup--input-group">
                        <button type="submit">{ "Signup" }</button>
                    </div>
                </form>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SubmitClicked => self.handle_submit_clicked(ctx),
            Msg::NetworkFailure(error) => Self::handle_network_failure(&error),
            Msg::NetworkSuccess(document) => Self::handle_network_success(ctx, &document),
        }
    }
}

impl UserSignup {
    fn handle_submit_clicked(&self, ctx: &Context<Self>) -> bool {
        let name = self.input_field.cast::<HtmlInputElement>().unwrap().value();
        let service = Service::from_context(ctx);

        ctx.link().send_future(async move {
            let result = service
                .api
                .create_user(&ResourcesDocument {
                    data: Some(Resources::Individual(Resource {
                        id: None,
                        type_: Some("user".to_string()),
                        attributes: Some(UserAttributes { name: Some(name) }),
                        links: None,
                        relationships: None,
                    })),
                    errors: None,
                    links: None,
                })
                .await;

            match result {
                Ok(document) => Msg::NetworkSuccess(document),
                Err(error) => Msg::NetworkFailure(error),
            }
        });

        true
    }

    fn handle_network_failure(error: &gloo::net::Error) -> bool {
        gloo::console::error!(format!("{error:?}"));
        false
    }

    fn handle_network_success(
        ctx: &Context<Self>,
        document: &ResourcesDocument<UserAttributes>,
    ) -> bool {
        if let Some(errors) = &document.errors {
            gloo::console::error!(format!("{errors:?}"));
            return true;
        }

        if document.data.is_some() {
            ctx.props().signup_success.emit(());
        }

        true
    }
}
