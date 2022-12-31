use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub name: AttrValue,
    pub message: AttrValue,
}

#[function_component]
pub fn LobbyChatListItem(props: &Props) -> Html {
    html! {
        <div class="lobby-chat-list-item">
            <div>{ &props.name } { ": " } { &props.message }</div>
        </div>
    }
}
