use std::{
    fmt::{self, Debug},
    rc::Rc,
};

use super::{auth::authorize, model::Me};
use crate::{hooks::use_persist::UsePersistAtom, oauth::Authorization};

#[derive(Debug)]
pub enum SpotifyState<'auth> {
    Unauthorized(Unauthorized),
    Authorized(SpotifySession<'auth>),
}

#[derive(Debug)]
pub enum SpotifySession<'auth> {
    Unknown,
    Valid(ValidSession<'auth>),
    Invalid(InvalidSession<'auth>),
}

#[derive(Debug)]
pub struct Unauthorized {}

impl Unauthorized {
    pub fn authorize(&self) {
        authorize()
    }
}

#[derive(Clone)]
pub(super) struct Session<'auth> {
    pub(super) atom_ref: &'auth UsePersistAtom<Option<Authorization>>,
    pub(super) authorization: &'auth Authorization,
}

impl<'auth> Debug for Session<'auth> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoggedIn")
            .field("authorization", &self.authorization)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct ValidSession<'auth> {
    pub(super) session: Session<'auth>,
    pub(super) me: Rc<Me>,
}

impl<'auth> ValidSession<'auth> {
    pub fn reauthorize(&self) {
        authorize()
    }

    pub fn unauthorize(&self) {
        self.session.atom_ref.set(None);
    }

    pub fn authorization(&self) -> &Authorization {
        self.session.authorization
    }

    pub fn me(&self) -> &Me {
        &self.me
    }
}

#[derive(Debug, Clone)]
pub struct InvalidSession<'auth> {
    pub(super) session: Session<'auth>,
}

impl<'auth> InvalidSession<'auth> {
    pub fn reauthorize(&self) {
        authorize()
    }

    pub fn unauthorize(&self) {
        self.session.atom_ref.set(None)
    }

    pub fn authorization(&self) -> &Authorization {
        self.session.authorization
    }
}
