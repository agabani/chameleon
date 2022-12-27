use chameleon_protocol::{
    attributes::GameAttributes,
    jsonapi::{Resource, Resources, ResourcesDocument},
};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::scope_ext::RouterScopeExt;

use crate::{
    components::top_menu::{Item, TopMenu},
    services::Service,
    Route,
};

pub struct Host {
    name: NodeRef,
}

pub enum Msg {
    HostClicked,
    Success(ResourcesDocument<GameAttributes>),
    Failed(gloo::net::Error),
}

impl Component for Host {
    type Message = Msg;

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            name: NodeRef::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onsubmit = ctx.link().callback(|event: SubmitEvent| {
            event.prevent_default();
            Msg::HostClicked
        });

        html! {
            <div class="host">
                <TopMenu active={Item::Host} />
                <div>
                    <div class="host--title">{ "Host Server" }</div>
                    <div>
                        <form {onsubmit}>
                            <div class="host--input-group">
                                <label class="host--label">{ "Name:" }</label>
                                <input class="host--input" ref={ self.name.clone() } />
                            </div>
                            <div>
                                <button class="host--submit">{ "Host" }</button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HostClicked => self.handle_submit(ctx),
            Msg::Failed(error) => Self::handle_failed(ctx, &error),
            Msg::Success(document) => Self::handle_success(ctx, document),
        }
    }
}

impl Host {
    fn handle_submit(&self, ctx: &Context<Self>) -> bool {
        let name = self.name.cast::<HtmlInputElement>().expect("Input").value();
        let service = Service::from_context(ctx);

        ctx.link().send_future(async move {
            let result = service
                .api
                .create_game(&ResourcesDocument {
                    data: Some(Resources::Individual(Resource {
                        id: None,
                        type_: Some("game".to_string()),
                        attributes: Some(GameAttributes { name: Some(name) }),
                        links: None,
                        relationships: None,
                    })),
                    errors: None,
                    links: None,
                })
                .await;

            match result {
                Ok(document) => Msg::Success(document),
                Err(error) => Msg::Failed(error),
            }
        });

        true
    }

    fn handle_failed(_ctx: &Context<Self>, error: &gloo::net::Error) -> bool {
        gloo::console::error!(format!("{error:?}"));
        true
    }

    fn handle_success(ctx: &Context<Self>, document: ResourcesDocument<GameAttributes>) -> bool {
        if let Some(errors) = document.errors {
            gloo::console::error!(format!("{errors:?}"));
            return true;
        }

        let id = document
            .try_get_resources()
            .and_then(Resources::try_get_individual)
            .and_then(|resource| {
                resource.try_get_field(|resource| resource.id.as_ref(), "id", "Id")
            })
            .expect("Id");

        gloo::console::log!(format!("Server {id} created"));

        ctx.link().navigator().unwrap().push(&Route::Browse);
        true
    }
}
