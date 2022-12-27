use yew::prelude::*;

pub struct ServerListItem {}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub selected: bool,
    pub name: AttrValue,
    pub details: AttrValue,
    pub players: AttrValue,
    pub password: bool,
    pub onclick: Callback<MouseEvent>,
}

impl Component for ServerListItem {
    type Message = ();

    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div
                class={classes!("server-list-item", if ctx.props().selected {"selected"} else {""})}
                onclick={&ctx.props().onclick}>
                <div class="server-list-item--meta">
                    <div class="server-list-item--name">{ &ctx.props().name }</div>
                    <div class="server-list-item--detail">{ &ctx.props().details }</div>
                </div>
                <div class="server-list-item--players">{ &ctx.props().players }</div>
                <div class="server-list-item--locked">{ if ctx.props().password { "🔒" } else { " " } }</div>
            </div>
        }
    }
}
