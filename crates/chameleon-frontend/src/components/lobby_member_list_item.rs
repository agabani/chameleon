use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,
    pub name: AttrValue,
}

#[function_component]
pub fn LobbyMemberListItem(props: &Props) -> Html {
    html! {
        <div class="lobby-member-list-item">
            <div>{ &props.name }</div>
        </div>
    }
}
