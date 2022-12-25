use yew::prelude::*;

pub struct Host {}

impl Component for Host {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div>{ "Host" }</div>
        }
    }
}
