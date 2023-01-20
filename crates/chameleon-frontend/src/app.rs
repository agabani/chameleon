use yew::prelude::*;
use yew_router::prelude::*;

use crate::{
    components::{authentication_switch::AuthenticationSwitch, theme_container::ThemeContainer},
    contexts::{current_user::CurrentUserProvider, network::NetworkProvider, theme::ThemeProvider},
    pages::{
        browse::Browse, host::Host, lobby::Lobby, main_menu::MainMenu, name_invite::NameInvite,
        user::User,
    },
};

#[function_component]
pub fn App() -> Html {
    html! {
        <NetworkProvider>
        <ThemeProvider>
        <CurrentUserProvider>
        <BrowserRouter>
            <ThemeContainer>
                <Switch<Route> render={|route| match route {
                    Route::MainMenu => html! {
                        <AuthenticationSwitch
                            challenge={|_| html!{ <User /> }}
                            render={|_| html!{ <MainMenu /> }}
                        />
                    },
                    Route::Browse =>  html !{
                        <AuthenticationSwitch
                            challenge={|_| html!{ <User /> }}
                            render={|_| html!{ <Browse /> }}
                        />
                    },
                    Route::Lobby { id } => {
                        let id1 = AttrValue::from(id);
                        let id2 = id1.clone();
                        html! {
                           <AuthenticationSwitch
                                challenge={move |_| {
                                    html!{ <NameInvite id={id1.clone()} /> }
                                }}
                                render={move |_| {
                                    html!{ <Lobby id={id2.clone()} /> }
                                }}
                            />
                        }
                    },
                    Route::Host =>  html !{
                        <AuthenticationSwitch
                            challenge={|_| html!{ <User /> }}
                            render={|_| html!{ <Host /> }}
                        />
                    },
                    Route::NotFound => html !{
                        <div>{ "Not Found" }</div>
                    },
                }} />
            </ThemeContainer>
        </BrowserRouter>
        </CurrentUserProvider>
        </ThemeProvider>
        </NetworkProvider>
    }
}

#[derive(Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    MainMenu,
    #[at("/browse")]
    Browse,
    #[at("/host")]
    Host,
    #[at("/lobby/:id")]
    Lobby { id: String },
    #[not_found]
    #[at("/not-found")]
    NotFound,
}
