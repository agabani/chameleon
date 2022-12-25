use yew::prelude::*;

pub struct Browse {}

impl Component for Browse {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div>{ "Browse" }</div>
        }
    }
}
