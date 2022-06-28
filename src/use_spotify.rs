use std::{
    cell::Ref,
    fmt::{self, Debug},
};

use dioxus::prelude::*;
use gloo_net::http::Request;
use gloo_storage::{errors::StorageError, LocalStorage, Storage};
use tracing::{error, trace, warn};

use crate::oauth::Authorization;

const SPOTIFY_STORAGE: &str = concat!(env!("CARGO_PKG_NAME"), "_spotify");

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

    // let future = use_coroutine(cx, |rx| async {});

    trace!(spotify_credentials = ?spotify_credentials.read());

    if let Some(spotify_credentials) = spotify_credentials.read().as_ref() {
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

fn logout() {}

pub mod state {
    use std::fmt::{self, Debug};

    use dioxus::fermi::UseAtomRef;
    use gloo_storage::{LocalStorage, Storage};
    use tracing::trace;

    use crate::oauth::Authorization;

    use super::SPOTIFY_STORAGE;

    #[derive(Debug)]
    pub struct LoggedOut {}

    impl LoggedOut {
        #[tracing::instrument]
        pub fn login(&self) {
            trace!("login");
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
        #[tracing::instrument]
        pub fn logout(&self) {
            trace!("logout");

            *self.authorization.write() = None;
            LocalStorage::delete(SPOTIFY_STORAGE);
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
        #[tracing::instrument]
        pub fn logout(&self) {
            trace!("logout");

            *self.authorization.write() = None;
            LocalStorage::delete(SPOTIFY_STORAGE);
        }

        #[tracing::instrument]
        pub fn login(&self) {
            trace!("login");
        }
    }
}
