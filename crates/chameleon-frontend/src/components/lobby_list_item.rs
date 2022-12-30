use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,
    pub name: AttrValue,
    pub onclick: Callback<()>,
}

#[function_component]
pub fn LobbyListItem(props: &Props) -> Html {
    let onclick = use_callback(|_, callback| callback.emit(()), props.onclick.clone());

    html! {
        <div class="lobby-list-item" key={props.id.as_ref()} onclick={onclick}>
            <div>{ &props.name }</div>
        </div>
    }
}
