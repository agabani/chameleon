use std::rc::Rc;

use yew::prelude::*;

#[derive(Default, PartialEq)]
pub struct ThemeState {
    pub variant: ThemeVariant,
}

#[derive(Default, PartialEq)]
pub enum ThemeVariant {
    #[default]
    Dark,
}

impl Reducible for ThemeState {
    type Action = Self;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        action.into()
    }
}

pub type ThemeContext = UseReducerHandle<ThemeState>;

#[derive(PartialEq, Properties)]
pub struct ThemeProps {
    pub children: Children,
}

#[function_component]
pub fn ThemeProvider(props: &ThemeProps) -> Html {
    let context = use_reducer(ThemeState::default);

    html! {
        <ContextProvider<ThemeContext> context={context}>
            { props.children.clone() }
        </ContextProvider<ThemeContext>>
    }
}
