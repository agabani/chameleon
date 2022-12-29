use yew::prelude::*;

#[derive(Default, PartialEq)]
pub struct State {
    variant: Variant,
}

#[derive(Default, PartialEq)]
pub enum Variant {
    #[default]
    Dark,
}

impl Reducible for State {
    type Action = Self;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        action.into()
    }
}

pub type Context = UseReducerHandle<State>;

#[derive(PartialEq, Properties)]
pub struct Props {
    pub children: Children,
}

#[function_component]
pub fn Provider(props: &Props) -> Html {
    let context = use_reducer(State::default);

    html! {
        <ContextProvider<Context> context={context}>
            { props.children.clone() }
        </ContextProvider<Context>>
    }
}
