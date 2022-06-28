use std::time::Instant;

use monostate::MustBe;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

#[derive(Deserialize)]
pub struct ImplicitGrant {
    access_token: String,
    token_type: MustBe!("Bearer"),
    state: String,
    expires_in: i64,
}

impl ImplicitGrant {
    /// This should be called as soon as an [`ImplicitGrant`] is procured
    pub fn into_authorization(self) -> Authorization {
        Authorization {
            access_token: self.access_token,
            expires_at: OffsetDateTime::now_utc() + Duration::seconds(self.expires_in),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Authorization {
    access_token: String,
    expires_at: OffsetDateTime,
}
