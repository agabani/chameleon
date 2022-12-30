use std::str::FromStr;

use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::contexts::theme::{ThemeContext, ThemeState, ThemeVariant};

#[function_component]
pub fn ThemePicker() -> Html {
    let context = use_context::<ThemeContext>().unwrap();
    let node_ref = use_node_ref();
    let onchange = use_callback(
        |_, (context, node_ref)| {
            let value = node_ref.cast::<HtmlInputElement>().unwrap().value();
            let variant = ThemeVariant::from_str(&value).unwrap_or_default();
            context.dispatch(ThemeState { variant });
        },
        (context.clone(), node_ref.clone()),
    );

    html! {
        <div class="theme-picker">
            <select onchange={onchange} ref={node_ref}>
                {
                    ThemeVariant::into_iter().map(|variant| html! {
                        <option
                            value={variant.to_string()}
                            selected={variant == context.variant}
                        >{ variant }</option>
                    }).collect::<Html>()
                }
            </select>
        </div>
    }
}
