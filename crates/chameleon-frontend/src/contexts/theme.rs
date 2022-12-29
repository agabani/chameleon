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
    Light,
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

impl Reducible for ThemeState {
    type Action = Self;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        action.into()
    }
}

impl ThemeVariant {
    pub fn into_iter() -> impl Iterator<Item = Self> {
        [ThemeVariant::Dark, ThemeVariant::Light].into_iter()
    }
}

impl std::fmt::Display for ThemeVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ThemeVariant::Dark => "dark",
                ThemeVariant::Light => "light",
            }
        )
    }
}

impl std::str::FromStr for ThemeVariant {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dark" => Ok(ThemeVariant::Dark),
            "light" => Ok(ThemeVariant::Light),
            _ => Err(()),
        }
    }
}
