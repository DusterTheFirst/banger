use dioxus::prelude::*;
use use_spotify::{use_spotify, Spotify};

mod oauth;
mod use_spotify;

fn main() {
    tracing_wasm::set_as_global_default();

    dioxus::web::launch(app);
}

fn app(cx: Scope) -> Element {
    let spotify = use_spotify(&cx);

    cx.render(rsx! {
        div { "hello, wasm!" }
        div { "{spotify:?}" }
        div {
            {{
                match spotify {
                    Spotify::Verifying => None,
                    Spotify::LoggedOut(state) => Some(rsx!{
                        button { onclick: move |evt| state.login(), "Log In" }
                    }),
                    Spotify::LoggedIn(state) => Some(rsx!{
                        button { onclick: move |evt| state.logout(), "Log Out" }
                    }),
                    Spotify::InvalidSession(state) => Some(rsx!{
                        button { onclick: move |evt| state.login(), "Refresh Login" }
                        // button { onclick: move |evt| state.logout(), "Log Out" }
                    }),
                }
            }}
        }
    })
}
