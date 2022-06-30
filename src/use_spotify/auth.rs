use std::rc::Rc;

use const_format::concatcp;
use dioxus::fermi::UseAtomRef;
use gloo_storage::{LocalStorage, Storage};
use gloo_utils::window;
use rand::Rng;
use tracing::{info, trace};

use super::{SPOTIFY_CLIENT_ID, SPOTIFY_STATE_STORAGE, SPOTIFY_STORAGE};
use crate::oauth::{Authorization, ImplicitGrantRequest};

const SPOTIFY_ACCOUNTS: &str = "https://accounts.spotify.com";

const SPOTIFY_TOKEN_URL: &str = concatcp!(SPOTIFY_ACCOUNTS, "/api/token");
const SPOTIFY_AUTH_URL: &str = concatcp!(SPOTIFY_ACCOUNTS, "/authorize");

#[tracing::instrument(skip(authorization))]
pub fn unauthorize(authorization: &UseAtomRef<Option<Rc<Authorization>>>) {
    trace!("unauthorize");

    *authorization.write() = None;
    LocalStorage::delete(SPOTIFY_STORAGE);
}

#[tracing::instrument]
pub fn authorize() {
    // Save the random state to local storage for verification
    let state = {
        let mut state = [0_u8; 128];
        rand::thread_rng().fill(&mut state);

        let state = base64::encode(state);
        LocalStorage::set(SPOTIFY_STATE_STORAGE, &state)
            .expect("failed to save state to LocalStorage");

        state
    };

    let query = serde_urlencoded::to_string(ImplicitGrantRequest {
        response_type: Default::default(),
        client_id: SPOTIFY_CLIENT_ID,
        scope: "user-read-currently-playing",
        redirect_uri: &window().location().origin().unwrap(),
        state: &state,
    })
    .unwrap();

    let href = format!("{SPOTIFY_AUTH_URL}?{query}");

    info!(href, "redirecting to spotify authorization page");

    window().location().set_href(&href).unwrap();
}
