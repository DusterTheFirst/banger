use std::{
    fmt::{self, Debug},
    rc::Rc,
};

use super::{auth::authorize, model::Me};
use crate::{hooks::use_persist::UsePersistAtom, oauth::Authorization};

#[derive(Debug)]
pub enum SpotifyState<'state> {
    Unauthorized(Unauthorized),
    Authorized(SpotifySession<'state>),
}

#[derive(Debug)]
pub enum SpotifySession<'state> {
    Unknown,
    Valid(ValidSession<'state>),
    Invalid(InvalidSession<'state>),
}

#[derive(Debug)]
pub struct Unauthorized {}

impl Unauthorized {
    pub fn authorize(&self) {
        authorize()
    }
}

#[derive(Clone)]
pub(super) struct Session<'state> {
    pub(super) atom_ref: &'state UsePersistAtom<Option<Authorization>>,
    pub(super) authorization: &'state Authorization,
}

impl<'state> Debug for Session<'state> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoggedIn")
            .field("authorization", &self.authorization)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct ValidSession<'state> {
    pub(super) session: Session<'state>,
    pub(super) me: &'state Me,
}

impl<'state> ValidSession<'state> {
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
        self.me
    }
}

#[derive(Debug, Clone)]
pub struct InvalidSession<'state> {
    pub(super) session: Session<'state>,
}

impl<'state> InvalidSession<'state> {
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
