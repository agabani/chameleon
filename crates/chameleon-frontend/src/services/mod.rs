use std::rc::Rc;

pub mod api;
pub mod storage;
pub mod websocket;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Service {
    pub api: api::ApiService,
    pub websocket: websocket::WebSocketService,
}

impl Service {
    pub fn from_context<COMP: yew::BaseComponent>(ctx: &yew::Context<COMP>) -> Rc<Service> {
        let (service, _) = ctx
            .link()
            .context::<Rc<Service>>(yew::Callback::noop())
            .expect("Failed to find service in context");

        service
    }
}
