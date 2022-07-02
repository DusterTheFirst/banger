use gloo_storage::{LocalStorage, Storage};
use gloo_utils::window;
use rand::Rng;
use tracing::info;

use crate::{
    consts::{SPOTIFY_CLIENT_ID, SPOTIFY_STATE_STORAGE},
    oauth::ImplicitGrantRequest,
};

const SPOTIFY_AUTH_URL: &str = "https://accounts.spotify.com/authorize";

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
