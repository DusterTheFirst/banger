use atoms::persist::PersistAtom;
use consts::SETTING_AUTO_REFRESH;
use dioxus::prelude::*;
use hooks::{
    use_persist::use_persist,
    use_spotify::{
        state::{SpotifySession, SpotifyState},
        use_spotify,
    },
};
use tracing::info;
use tracing_log::{log::LevelFilter, LogTracer};

mod atoms;
mod consts;
mod hooks;
mod oauth;

fn main() {
    LogTracer::init_with_filter(LevelFilter::Info).unwrap();
    tracing_wasm::set_as_global_default();

    dioxus::web::launch(app);
}

static AUTO_REFRESH: PersistAtom<bool> = PersistAtom::new(SETTING_AUTO_REFRESH, || false);

fn app(cx: Scope) -> Element {
    let auto_refresh = use_persist(&cx, AUTO_REFRESH);
    let spotify = use_spotify(&cx);

    if let SpotifyState::Authorized(SpotifySession::Invalid(session)) = &spotify {
        if *auto_refresh.get() && session.authorization().is_expired() {
            info!("Attempting to re-authorize");
            session.reauthorize();
        }
    }

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
        label {
            "auto-refresh"
            input {
                r#type: "checkbox",
                checked: "{auto_refresh}",
                onclick: |_| auto_refresh.set(!auto_refresh.get())
            }
        }
    })
}
