use yew::prelude::*;

pub struct App {}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub children: Children,
}

impl Component for App {
    type Message = ();

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="app dark">
            </div>
        }
    }
}
