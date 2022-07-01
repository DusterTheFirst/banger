use dioxus::{core::Scope, prelude::*};

use crate::hooks::use_spotify::state::{SpotifySession, SpotifyState};

#[inline_props]
#[allow(non_snake_case)]
pub fn Spotify<'s>(cx: Scope<'s>, state: SpotifyState<'s>) -> Element {
    let spotify = match state {
        SpotifyState::Unauthorized(state) => rsx! {
            div { "Unauthorized" }
            button {
                class: "unauthorize",
                onclick: move |_| state.authorize(),
                "Authorize"
            }
        },
        SpotifyState::Authorized(state) => match state {
            SpotifySession::Unknown => rsx! {
                div { "Loading Authorization" }
            },
            SpotifySession::Valid(session) => {
                let me = session.me();

                let username = me.display_name.as_ref().unwrap_or(&me.id);
                let url = &me.external_urls.spotify;

                rsx! {
                    div {
                        "Authorized as "
                        a {
                            href: "{url}",
                            target: "_blank",
                            "{username}"
                        }
                    }
                    button {
                        class: "unauthorize",
                        onclick: move |_| session.unauthorize(),
                        "Unauthorize"
                    }
                }
            }
            SpotifySession::Invalid(session) => rsx! {
                div { "Authorization Expired" }
                button {
                    class: "authorize",
                    onclick: { let session = session.clone(); move |_| session.reauthorize() },
                    "Refresh Authorization"
                }
                button {
                    class: "unauthorize",
                    onclick: move |_| session.unauthorize(), "Unauthorize"
                }
            },
        },
    };

    cx.render(rsx! {
        div {
            class: "spotify",
            div {
                img {
                    class: "logo",
                    src: "/img/Spotify_Logo_RGB_Green.png"
                }
            }
            spotify
        }
    })
}
