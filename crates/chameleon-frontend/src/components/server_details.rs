use yew::prelude::*;

pub struct ServerDetails {}

impl Component for ServerDetails {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="server-details"></div>
        }
    }
}
