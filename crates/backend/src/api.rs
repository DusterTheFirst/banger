use std::{
    collections::HashSet,
    env,
    fmt::{self, Display},
    str::FromStr,
    sync::{Arc, Mutex},
};

use axum::{
    extract::Query,
    http::status::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Extension, Router,
};
use base64::display::Base64Display;
use monostate::MustBe;
use rand::Rng;
use reqwest::header;
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
        .layer(Extension(OAuthStateStorage::default()))
        .layer(Extension(OAuthConfig::from_env()))
        .layer(Extension(
            reqwest::ClientBuilder::new()
                .https_only(true)
                .use_native_tls()
                .user_agent(concat!(
                    env!("CARGO_PKG_NAME"),
                    "/",
                    env!("CARGO_PKG_VERSION"),
                ))
                .build()
                .unwrap(),
        ))
}

#[derive(Debug, Clone)]
struct OAuthConfig {
    spotify_client_secret: Arc<str>,
    spotify_client_id: Arc<str>,
    // github_client_secret: Arc<str>
}

impl OAuthConfig {
    pub fn from_env() -> Self {
        Self {
            spotify_client_secret: Arc::from(
                env::var("SPOTIFY_CLIENT_SECRET").expect("SPOTIFY_CLIENT_SECRET env var not set"),
            ),

            spotify_client_id: Arc::from(
                env::var("SPOTIFY_CLIENT_ID").expect("SPOTIFY_CLIENT_ID env var not set"),
            ),
        }
    }
}

#[derive(Debug, Serialize)]
struct CodeGrantRequest<'s> {
    response_type: MustBe!("code"),
    client_id: &'s str,
    scope: &'s str,
    redirect_uri: &'s str,
    #[serde(with = "from_to_str")]
    state: State,
    show_dialog: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CodeGrantResponseInner {
    Success { code: String },
    Failure { error: String },
}

#[derive(Debug, Deserialize)]
struct CodeGrantResponse {
    #[serde(flatten)]
    inner: CodeGrantResponseInner,

    #[serde(with = "from_to_str")]
    state: State,
}

#[derive(Debug, Serialize)]
struct AccessTokenRequest {
    grant_type: MustBe!("authorization_code"),
    code: String,
    redirect_uri: &'static str,
}

#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    token_type: MustBe!("Bearer"),
    scope: String,
    expires_in: i32,
    refresh_token: String,
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
struct OAuthStateStorage {
    storage: Arc<Mutex<HashSet<State>>>,
}

impl OAuthStateStorage {
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
const SPOTIFY_TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
const SPOTIFY_SCOPE: &str = "user-read-currently-playing";

async fn spotify(
    Extension(state_storage): Extension<OAuthStateStorage>,
    Extension(config): Extension<OAuthConfig>,
) -> Redirect {
    let query = serde_urlencoded::to_string(CodeGrantRequest {
        response_type: Default::default(),
        client_id: &config.spotify_client_id,
        scope: SPOTIFY_SCOPE,
        redirect_uri: redirect_uri::SPOTIFY,
        state: state_storage.create_state(),
        show_dialog: true,
    })
    .unwrap();

    Redirect::temporary(&format!("{SPOTIFY_AUTH_URL}?{query}"))
}

async fn spotify_redirect(
    Query(grant): Query<CodeGrantResponse>,
    Extension(reqwest): Extension<reqwest::Client>,
    Extension(state_storage): Extension<OAuthStateStorage>,
    Extension(config): Extension<OAuthConfig>,
) -> Response {
    // TODO: html error pages
    if !state_storage.validate_state(grant.state) {
        return (
            StatusCode::BAD_REQUEST,
            "invalid state, suspected request forgery. did you navigate back to this page?",
        )
            .into_response();
    }

    match grant.inner {
        CodeGrantResponseInner::Failure { error } => (
            StatusCode::UNAUTHORIZED,
            format!("spotify rejected authorization request: {error}"),
        )
            .into_response(),
        CodeGrantResponseInner::Success { code } => {
            let auth = base64::encode(format!(
                "{}:{}",
                config.spotify_client_id, config.spotify_client_secret
            ));

            let response = reqwest
                .post(SPOTIFY_TOKEN_URL)
                .header(header::AUTHORIZATION, format!("Basic {auth}"))
                .form(&AccessTokenRequest {
                    code,
                    grant_type: Default::default(),
                    redirect_uri: redirect_uri::SPOTIFY,
                })
                .send()
                .await;

            // TODO: store and so shizzle with response

            dbg!(response
                .unwrap()
                .json::<AccessTokenResponse>()
                .await
                .unwrap());

            todo!()
        }
    }
}
