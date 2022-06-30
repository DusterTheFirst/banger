use std::borrow::Cow;

use monostate::MustBe;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ImplicitGrantRequest<'a> {
    pub response_type: MustBe!("token"),
    pub client_id: &'a str,
    pub scope: &'a str,
    pub redirect_uri: &'a str,
    pub state: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct ImplicitGrantResponse<'t> {
    access_token: Cow<'t, str>,
    #[allow(dead_code)]
    token_type: MustBe!("Bearer"),
    state: Cow<'t, str>,
    expires_in: u64,
}

impl ImplicitGrantResponse<'_> {
    /// This should be called as soon as an [`ImplicitGrant`] is procured
    pub fn into_authorization(self, known_state: &str) -> Option<Authorization> {
        if self.state != known_state {
            return None;
        }

        Some(Authorization {
            access_token: self.access_token.into_owned(),
            expires_at: instant::now() as u64 + self.expires_in * 1000,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Authorization {
    access_token: String,
    expires_at: u64,
}

impl Authorization {
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn expires_at(&self) -> u64 {
        self.expires_at
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at < instant::now() as u64
    }
}
