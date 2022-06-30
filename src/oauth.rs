use std::borrow::Cow;

use monostate::MustBe;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ImplicitGrant<'t> {
    access_token: Cow<'t, str>,
    #[allow(dead_code)]
    token_type: MustBe!("Bearer"),
    state: Cow<'t, str>,
    expires_in: u64,
}

impl ImplicitGrant<'_> {
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

#[derive(Deserialize, Serialize, Debug)]
pub struct Authorization {
    access_token: String,
    expires_at: u64,
}
