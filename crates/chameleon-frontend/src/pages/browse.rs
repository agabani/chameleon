use yew::prelude::*;

use crate::components::{
    server_details::ServerDetails,
    server_list::ServerList,
    top_menu::{Item, TopMenu},
};

pub struct Browse {}

impl Component for Browse {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <TopMenu active={Item::Browse} />
                <div class="browse">
                    <div>
                        <ServerList />
                    </div>
                    <div>
                        <ServerDetails />
                    </div>
                </div>
            </>
        }
    }
}
