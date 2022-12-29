use yew::prelude::*;

use crate::{
    components::{
        authentication_switch::AuthenticationSwitch, theme_container::ThemeContainer,
        theme_picker::ThemePicker,
    },
    contexts::{current_user::CurrentUserProvider, theme::ThemeProvider},
};

#[function_component]
pub fn App() -> Html {
    html! {
        <ThemeProvider>
        <CurrentUserProvider>
            <ThemeContainer>
                <ThemePicker />
                <AuthenticationSwitch
                    challenge={|_| html!{ { "login" } }}
                    render={|_| html!{ { "app" } }} />
            </ThemeContainer>
        </CurrentUserProvider>
        </ThemeProvider>
    }
}
