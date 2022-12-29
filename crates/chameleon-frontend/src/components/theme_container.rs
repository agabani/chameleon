use yew::prelude::*;

use crate::contexts::theme::{ThemeContext, ThemeVariant};

#[derive(PartialEq, Properties)]
pub struct Props {
    pub children: Children,
}

#[function_component]
pub fn ThemeContainer(props: &Props) -> Html {
    let context = use_context::<ThemeContext>().unwrap();

    let theme_variant = match context.variant {
        ThemeVariant::Dark => "theme--dark",
    };

    html! {
        <div class={classes!("theme", theme_variant)}>
            { props.children.clone() }
        </div>
    }
}
