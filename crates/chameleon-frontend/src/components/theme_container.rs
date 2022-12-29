use yew::prelude::*;

use crate::contexts::theme::ThemeContext;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub children: Children,
}

#[function_component]
pub fn ThemeContainer(props: &Props) -> Html {
    let context = use_context::<ThemeContext>().unwrap();

    html! {
        <div class={classes!("theme-container", format!("theme--{}", context.variant))}>
            { props.children.clone() }
        </div>
    }
}
