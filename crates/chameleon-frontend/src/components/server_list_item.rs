use yew::prelude::*;

pub struct ServerListItem {}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub selected: bool,
    pub name: AttrValue,
    pub details: AttrValue,
    pub players: AttrValue,
    pub password: bool,
}

impl Component for ServerListItem {
    type Message = ();

    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={classes!("server-list-item", if ctx.props().selected {"selected"} else {""})}>
                <div class="meta">
                    <div class="name">{ &ctx.props().name }</div>
                    <div class="detail">{ &ctx.props().details }</div>
                </div>
                <div class="players">{ &ctx.props().players }</div>
                <div class="locked">{ if ctx.props().password { "🔒" } else { " " } }</div>
            </div>
        }
    }
}
