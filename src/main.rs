use dioxus::prelude::*;
use hooks::use_spotify::{
    state::{SpotifySession, SpotifyState},
    use_spotify,
};
use tracing_log::{log::LevelFilter, LogTracer};

mod atoms;
mod hooks;
mod oauth;

fn main() {
    LogTracer::init_with_filter(LevelFilter::Info).unwrap();
    tracing_wasm::set_as_global_default();

    dioxus::web::launch(app);
}

fn app(cx: Scope) -> Element {
    let spotify = use_spotify(&cx);

    let spotify_string = format!("{spotify:#?}");
    let spotify = match spotify {
        SpotifyState::Unauthorized(state) => rsx! {
            h1 { "Unauthorized" }
            button { onclick: move |_| state.authorize(), "Authorize" }
        },
        SpotifyState::Authorized(state) => match state {
            SpotifySession::Unknown => rsx! {
                h1 { "Authorized" }
                h2 { "Loading ..." }
            },
            SpotifySession::Valid(session) => {
                let me = format!("{:#?}", session.me());

                rsx! {
                    h1 { "Authorized" }
                    h2 { "Valid" }
                    pre { "{me}" }
                    button { onclick: { let session = session.clone(); move |_| session.reauthorize() }, "Refresh Login" }
                    button { onclick: move |_| session.unauthorize(), "Log Out" }
                }
            }
            SpotifySession::Invalid(session) => rsx! {
                h1 { "Authorized" }
                h2 { "Invalid" }
                button { onclick: { let session = session.clone(); move |_| session.reauthorize() }, "Refresh Login" }
                button { onclick: move |_| session.unauthorize(), "Log Out" }
            },
        },
    };

    cx.render(rsx! {
        pre { "{spotify_string}" }
        div {
            spotify
        }
    })
}
