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

use components::spotify::Spotify;

mod atoms;
mod components;
mod consts;
mod hooks;
mod oauth;

fn main() {
    LogTracer::init_with_filter(LevelFilter::Info).unwrap();
    tracing_wasm::set_as_global_default();

    dioxus::web::launch(app);
}

static AUTO_REAUTHORIZE: PersistAtom<bool> = PersistAtom::new(SETTING_AUTO_REFRESH, || false);

fn app(cx: Scope) -> Element {
    let auto_reauthorize = use_persist(&cx, AUTO_REAUTHORIZE);
    let spotify = use_spotify(&cx);

    if let SpotifyState::Authorized(SpotifySession::Invalid(session)) = &spotify {
        if *auto_reauthorize.get() && session.authorization().is_expired() {
            info!("Attempting to re-authorize spotify");
            session.reauthorize();
        }
    }

    cx.render(rsx! {
        main {
            class: "auth_section",
            Spotify { state: spotify }
            label {
                class: "auto_reauthorize",
                "Automatically Reauthorize"
                input {
                    r#type: "checkbox",
                    checked: "{auto_reauthorize}",
                    onclick: |_| auto_reauthorize.set(!auto_reauthorize.get())
                }
            }
        }
    })
}
