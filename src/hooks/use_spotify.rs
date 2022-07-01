use std::rc::Rc;

use const_format::concatcp;
use dioxus::prelude::*;
use futures_util::StreamExt;
use gloo_net::http::Request;
use gloo_storage::{errors::StorageError, LocalStorage, Storage};
use gloo_utils::window;
use tracing::{error, trace};

use self::{model::Me, state::SpotifyState};
use crate::{
    atoms::persist::PersistAtom,
    hooks::use_spotify::state::{
        InvalidSession, Session, SpotifySession, Unauthorized, ValidSession,
    },
    oauth::{Authorization, ImplicitGrantResponse},
};

use super::use_persist::use_persist;

mod auth;
pub mod model;
pub mod state;

const SPOTIFY_STORAGE: &str = concat!(env!("CARGO_PKG_NAME"), "_spotify");
const SPOTIFY_STATE_STORAGE: &str = concatcp!(SPOTIFY_STORAGE, "_state");

const SPOTIFY_CLIENT_ID: &str = "be6201c1e3154c51b50ffb302e770db5";

// TODO: abstract away gets and sets to this to ensure sync with local storage
static SPOTIFY_CREDENTIALS: PersistAtom<Option<Authorization>> =
    PersistAtom::new(SPOTIFY_STORAGE, || None);

static ME: AtomRef<Option<Result<Rc<Me>, ()>>> = |_| None;

async fn get_me(auth: &Authorization) -> Result<Result<Rc<Me>, ()>, gloo_net::Error> {
    let response = Request::new("https://api.spotify.com/v1/me")
        .header("Authorization", &format!("Bearer {}", auth.access_token()))
        .header("Accept", "application/json")
        .send()
        .await?;

    Ok(if !response.ok() {
        let error = response.json::<serde_json::Value>().await;

        error!(?error, "Spotify api returned error");

        Err(())
    } else {
        Ok(Rc::new(response.json::<Me>().await?))
    })
}

pub fn use_spotify(cx: &ScopeState) -> SpotifyState {
    let spotify_credentials = use_persist(cx, SPOTIFY_CREDENTIALS);
    let me = use_atom_ref(cx, ME);

    let routine = use_coroutine::<Authorization, _, _>(cx, |mut rx| {
        let me = me.clone();

        async move {
            while let Some(auth) = rx.next().await {
                if me.read().is_some() {
                    continue;
                }

                match get_me(&auth).await {
                    Ok(new_me) => *me.write() = Some(new_me),
                    Err(error) => {
                        error!(?error, "failed to fetch /me")
                    }
                };
            }
        }
    });

    let hash = window().location().hash().unwrap();
    if let Some(query) = hash.strip_prefix('#') {
        let query = serde_urlencoded::from_str::<ImplicitGrantResponse>(query).unwrap();
        trace!(?query);

        // TODO: less jank?
        match LocalStorage::get(SPOTIFY_STATE_STORAGE) as Result<String, _> {
            Ok(known_state) => match query.into_authorization(&known_state) {
                Some(authorization) => {
                    LocalStorage::set(SPOTIFY_STORAGE, &authorization).unwrap();
                    spotify_credentials.set(Some(authorization));
                }
                None => error!("States do not match, rejecting token"),
            },
            Err(StorageError::KeyNotFound(_)) => {
                error!("No state saved, rejecting token");
            }
            Err(StorageError::SerdeError(serde_error)) => {
                error!(%serde_error, "Encountered an error parsing spotify state local storage");
            }
            Err(StorageError::JsError(js_error)) => {
                error!(%js_error, "Encountered a javascript error loading spotify state local storage",);
            }
        }

        *me.write() = None;
        LocalStorage::delete(SPOTIFY_STATE_STORAGE);
        window().location().set_hash("").unwrap();
    }

    if let Some(authorization) = spotify_credentials.read() {
        let session = Session {
            atom_ref: spotify_credentials,
            authorization,
        };

        // Skip the whole /me shenanigans if expired
        if session.authorization.is_expired() {
            return SpotifyState::Authorized(SpotifySession::Invalid(InvalidSession { session }));
        }

        SpotifyState::Authorized(match me.read().as_ref() {
            Some(Ok(me)) => SpotifySession::Valid(ValidSession {
                session,
                me: me.clone(),
            }),
            Some(Err(())) => SpotifySession::Invalid(InvalidSession { session }),
            None => {
                routine.send(session.authorization.clone());

                SpotifySession::Unknown
            }
        })
    } else {
        SpotifyState::Unauthorized(Unauthorized {})
    }
}
