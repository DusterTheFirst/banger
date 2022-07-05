use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display},
    marker::PhantomData,
    str::FromStr,
    sync::{Arc, Mutex},
};

use axum::{
    body::{boxed, Empty},
    extract::Query,
    http::{header, status::StatusCode, uri::Parts, Uri},
    response::Response,
    routing::get,
    Extension, Router,
};
use base64::display::Base64Display;
use monostate::MustBe;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::serde::from_to_str;

mod redirect_uri {
    #[cfg(debug_assertions)]
    const BASE: &str = "http://127.0.0.1:8080/";

    #[cfg(not(debug_assertions))]
    const BASE: &str = "http://banger.spotify.dusterthefirst.com/";

    pub const SPOTIFY: &str = const_format::concatcp!(BASE, "api/auth/spotify/redirect");
    pub const GITHUB: &str = const_format::concatcp!(BASE, "api/auth/github/redirect");
}

pub fn create_router() -> Router {
    Router::new()
        .route("/healthy", get(|| async { "OK" }))
        .route("/auth/spotify", get(spotify))
        .route("/auth/spotify/redirect", get(spotify_redirect))
        .route("/auth/github", get(|| async { "TODO" }))
        .route("/auth/github/redirect", get(|| async { "TODO" }))
        .layer(Extension(OAuthStateStorage::<SpotifyBucket>::default()))
        .layer(Extension(OAuthStateStorage::<GithubBucket>::default()))
}

#[derive(Debug, Serialize)]
struct CodeGrantRequest {
    response_type: MustBe!("code"),
    client_id: &'static str,
    scope: &'static str,
    redirect_uri: &'static str,
    #[serde(with = "from_to_str")]
    state: State,
}

#[derive(Debug, Deserialize)]
struct CodeGrantResponse {
    code: String,
    #[serde(with = "from_to_str")]
    state: State,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct State {
    state: [u8; 128],
}

impl State {
    pub fn random() -> Self {
        let mut state = [0_u8; 128];
        rand::thread_rng().fill(&mut state);

        Self { state }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Base64Display::with_config(&self.state, base64::URL_SAFE).fmt(f)
    }
}

impl FromStr for State {
    type Err = base64::DecodeError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let mut state = [0_u8; 128];

        let len = base64::decode_config_slice(str, base64::URL_SAFE, &mut state)?;

        if len != state.len() {
            return Err(base64::DecodeError::InvalidLength);
        }

        Ok(Self { state })
    }
}

#[derive(Default, Clone)]
struct OAuthStateStorage<B: Bucket> {
    storage: Arc<Mutex<HashSet<State>>>,
    _bucket: PhantomData<B>,
}

trait Bucket: Default {}

#[derive(Default, Debug, Clone, Copy)]
struct SpotifyBucket;
impl Bucket for SpotifyBucket {}

#[derive(Default, Debug, Clone, Copy)]
struct GithubBucket;
impl Bucket for GithubBucket {}

impl<B: Bucket> OAuthStateStorage<B> {
    pub fn create_state(&self) -> State {
        let mut storage = self.storage.lock().unwrap();

        // If state collides, skip it
        let state = loop {
            let state = State::random();

            if storage.contains(&state) {
                warn!(%state, "state collision occurred");

                continue;
            }

            break state;
        };

        storage.insert(state);

        state
    }

    pub fn validate_state(&self, state: State) -> bool {
        self.storage.lock().unwrap().remove(&state)
    }
}

const SPOTIFY_AUTH_URL: &str = "https://accounts.spotify.com/authorize";
const SPOTIFY_CLIENT_ID: &str = "be6201c1e3154c51b50ffb302e770db5";
const SPOTIFY_SCOPE: &str = "user-read-currently-playing";

async fn spotify(
    Extension(state_storage): Extension<OAuthStateStorage<SpotifyBucket>>,
) -> Response {
    let query = serde_urlencoded::to_string(CodeGrantRequest {
        response_type: Default::default(),
        client_id: SPOTIFY_CLIENT_ID,
        scope: SPOTIFY_SCOPE,
        redirect_uri: redirect_uri::SPOTIFY,
        state: state_storage.create_state(),
    })
    .unwrap();

    let redirect = format!("{SPOTIFY_AUTH_URL}?{query}");

    Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header(header::LOCATION, redirect)
        .body(boxed(Empty::new()))
        .unwrap()
}

async fn spotify_redirect(
    Query(grant): Query<CodeGrantResponse>,
    Extension(state_storage): Extension<OAuthStateStorage<SpotifyBucket>>,
) -> Response {
    todo!()
}
