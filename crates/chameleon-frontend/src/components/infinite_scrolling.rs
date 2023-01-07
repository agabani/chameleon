use web_sys::HtmlElement;
use yew::prelude::*;

#[function_component]
pub fn InfiniteScrolling(props: &Props) -> Html {
    let node_ref = use_node_ref();

    let onclick = use_callback(
        |_, callback| handle_onclick(callback),
        props.onclick.clone(),
    );

    let onscroll = use_callback(
        |_, (callback, node_ref)| handle_onscroll(callback, node_ref),
        (props.onscroll.clone(), node_ref.clone()),
    );

    html! {
        <div class="infinite-scrolling" {onscroll} ref={node_ref}>
            { props.children.clone() }
            <button {onclick}>{ "load more" }</button>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,

    #[prop_or_default]
    pub onclick: Callback<()>,

    #[prop_or_default]
    pub onscroll: Callback<OnscrollEvent>,
}

fn handle_onclick(callback: &Callback<()>) {
    callback.emit(());
}

fn handle_onscroll(callback: &Callback<OnscrollEvent>, node_ref: &NodeRef) {
    let Some(html_element) = node_ref.cast::<HtmlElement>() else {
        return;
    };

    callback.emit(OnscrollEvent {
        client_height: html_element.client_height(),
        scroll_height: html_element.scroll_height(),
        scroll_top: html_element.scroll_top(),
    });
}

pub struct OnscrollEvent {
    pub client_height: i32,
    pub scroll_height: i32,
    pub scroll_top: i32,
}
