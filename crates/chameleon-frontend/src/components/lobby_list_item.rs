use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,
    pub name: AttrValue,
}

#[function_component]
pub fn LobbyListItem(props: &Props) -> Html {
    html! {
        <div class="lobby-list-item" key={props.id.as_ref()}>
            <div>{ &props.name }</div>
        </div>
    }
}
