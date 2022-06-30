use std::fmt::{self, Debug};

use const_format::concatcp;
use dioxus::{fermi::UseAtomRef, prelude::*};
use gloo_net::http::Request;
use gloo_storage::{errors::StorageError, LocalStorage, Storage};
use gloo_utils::window;
use monostate::MustBe;
use rand::Rng;
use serde::Serialize;
use tracing::{error, info, trace, warn};

use crate::oauth::{Authorization, ImplicitGrant};

const SPOTIFY_STORAGE: &str = concat!(env!("CARGO_PKG_NAME"), "_spotify");
const SPOTIFY_STATE_STORAGE: &str = concatcp!(SPOTIFY_STORAGE, "_state");

const SPOTIFY_ACCOUNTS: &str = "https://accounts.spotify.com";

const SPOTIFY_TOKEN_URL: &str = concatcp!(SPOTIFY_ACCOUNTS, "/api/token");
const SPOTIFY_AUTH_URL: &str = concatcp!(SPOTIFY_ACCOUNTS, "/authorize");

const SPOTIFY_CLIENT_ID: &str = "be6201c1e3154c51b50ffb302e770db5";

// TODO: abstract away gets and sets to this to ensure sync with local storage
static SPOTIFY_CREDENTIALS: AtomRef<Option<Authorization>> = |builder| {
    warn!("Evaluating atom");

    match LocalStorage::get(SPOTIFY_STORAGE) {
        Ok(data) => Some(data),
        Err(StorageError::KeyNotFound(_)) => None,
        Err(StorageError::SerdeError(serde_error)) => {
            error!(%serde_error, "Encountered an error parsing spotify local storage");

            None
        }
        Err(StorageError::JsError(js_error)) => {
            error!(%js_error, "Encountered a javascript error loading spotify local storage",);

            None
        }
    }
};

pub fn use_spotify(cx: &ScopeState) -> Spotify {
    let spotify_credentials = use_atom_ref(cx, SPOTIFY_CREDENTIALS);

    trace!(spotify_credentials = ?spotify_credentials.read());

    // let future = use_coroutine(cx, |rx| async {});

    let hash = window().location().hash().unwrap();
    if let Some(query) = hash.strip_prefix('#') {
        let query = serde_urlencoded::from_str::<ImplicitGrant>(query).unwrap();
        trace!(?query);

        match LocalStorage::get(SPOTIFY_STATE_STORAGE) as Result<String, _> {
            Ok(known_state) => match query.into_authorization(&known_state) {
                Some(authorization) => {
                    LocalStorage::delete(SPOTIFY_STATE_STORAGE);
                    LocalStorage::set(SPOTIFY_STORAGE, &authorization).unwrap();
                    *spotify_credentials.write() = Some(authorization);
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

        window().location().set_hash("").unwrap();
    }

    if let Some(authorization) = spotify_credentials.read().as_ref() {
        Spotify::Verifying
    } else {
        Spotify::LoggedOut(state::LoggedOut {})
    }
}

pub enum Spotify<'auth> {
    Verifying,
    LoggedOut(state::LoggedOut),
    LoggedIn(state::LoggedIn<'auth>),
    InvalidSession(state::InvalidSession<'auth>),
}

impl Debug for Spotify<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Spotify::Verifying => f.debug_tuple("Verifying").finish(),
            Spotify::LoggedOut(state) => state.fmt(f),
            Spotify::LoggedIn(state) => state.fmt(f),
            Spotify::InvalidSession(state) => state.fmt(f),
        }
    }
}

#[tracing::instrument(skip(authorization))]
fn logout(authorization: &UseAtomRef<Option<Authorization>>) {
    trace!("logout");

    *authorization.write() = None;
    LocalStorage::delete(SPOTIFY_STORAGE);
}

#[derive(Debug, Serialize)]
struct AuthQuery<'a> {
    response_type: MustBe!("token"),
    client_id: &'a str,
    scope: &'a str,
    redirect_uri: &'a str,
    state: &'a str,
}

#[tracing::instrument]
fn login() {
    // Save the random state to local storage for verification
    let state = {
        let mut state = [0_u8; 128];
        rand::thread_rng().fill(&mut state);

        let state = base64::encode(state);
        LocalStorage::set(SPOTIFY_STATE_STORAGE, &state)
            .expect("failed to save state to LocalStorage");

        state
    };

    let query = serde_urlencoded::to_string(AuthQuery {
        response_type: Default::default(),
        client_id: SPOTIFY_CLIENT_ID,
        scope: "user-read-currently-playing",
        redirect_uri: &window().location().origin().unwrap(),
        state: &state,
    })
    .unwrap();

    let href = format!("{SPOTIFY_AUTH_URL}?{query}");

    info!(href, "redirecting to spotify login page");

    window().location().set_href(&href).unwrap();
}

pub mod state {
    use std::fmt::{self, Debug};

    use dioxus::fermi::UseAtomRef;

    use crate::oauth::Authorization;

    use super::{login, logout};

    #[derive(Debug)]
    pub struct LoggedOut {}

    impl LoggedOut {
        pub fn login(&self) {
            login()
        }
    }

    pub struct LoggedIn<'auth> {
        pub(super) authorization: &'auth UseAtomRef<Option<Authorization>>,
        pub(super) username: String,
    }

    impl Debug for LoggedIn<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("LoggedIn")
                .field("username", &self.username)
                .finish()
        }
    }

    impl LoggedIn<'_> {
        pub fn logout(&self) {
            logout(self.authorization)
        }
    }

    pub struct InvalidSession<'auth> {
        pub(super) authorization: &'auth UseAtomRef<Option<Authorization>>,
        pub(super) username: String,
    }

    impl Debug for InvalidSession<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("InvalidSession")
                .field("username", &self.username)
                .finish()
        }
    }

    impl InvalidSession<'_> {
        pub fn logout(&self) {
            logout(self.authorization)
        }

        pub fn login(&self) {
            login()
        }
    }
}
