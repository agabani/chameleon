use yew::prelude::*;

use crate::{
    components::theme_container::ThemeContainer,
    contexts::{current_user::CurrentUserProvider, theme::ThemeProvider},
};

#[function_component]
pub fn App() -> Html {
    html! {
        <ThemeProvider>
        <CurrentUserProvider>
            <ThemeContainer>
                { "app" }
            </ThemeContainer>
        </CurrentUserProvider>
        </ThemeProvider>
    }
}
