use yew::prelude::*;

#[function_component]
pub fn LobbyListItem(props: &Props) -> Html {
    let onclick = use_callback(
        |_, (callback, id)| handle_onclick(callback, id),
        (props.onclick.clone(), props.id.clone()),
    );

    html! {
        <>
            <div class="lobby-list-item--name" onclick={onclick.clone()}>{ &props.name }</div>
            <div class="lobby-list-item--players" onclick={onclick.clone()}></div>
            <div class="lobby-list-item--locked" {onclick}>
                if props.require_passcode {
                    { "🔒" }
                }
            </div>
        </>
    }
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub id: AttrValue,

    pub name: AttrValue,

    pub require_passcode: bool,

    #[prop_or_default]
    pub onclick: Callback<AttrValue>,
}

fn handle_onclick(callback: &Callback<AttrValue>, id: &AttrValue) {
    callback.emit(id.clone());
}
