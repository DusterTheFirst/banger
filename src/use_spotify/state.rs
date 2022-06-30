use std::{
    fmt::{self, Debug},
    rc::Rc,
};

use dioxus::fermi::UseAtomRef;

use super::{
    auth::{authorize, unauthorize},
    model::Me,
};
use crate::oauth::Authorization;

#[derive(Debug)]
pub enum SpotifyState {
    Unauthorized(Unauthorized),
    Authorized(SpotifySession),
}

#[derive(Debug)]
pub enum SpotifySession {
    Unknown,
    Valid(ValidSession),
    Invalid(InvalidSession),
}

#[derive(Debug)]
pub struct Unauthorized {}

impl Unauthorized {
    pub fn authorize(&self) {
        authorize()
    }
}

#[derive(Clone)]
pub(super) struct Session {
    pub(super) atom_ref: UseAtomRef<Option<Rc<Authorization>>>,
    pub(super) authorization: Rc<Authorization>,
}

impl Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoggedIn")
            .field("authorization", &self.authorization)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct ValidSession {
    pub(super) session: Session,
    pub(super) me: Rc<Me>,
}

impl ValidSession {
    pub fn reauthorize(&self) {
        authorize()
    }

    pub fn unauthorize(&self) {
        unauthorize(&self.session.atom_ref)
    }

    pub fn authorization(&self) -> &Authorization {
        &self.session.authorization
    }

    pub fn me(&self) -> &Me {
        &self.me
    }
}

#[derive(Debug, Clone)]
pub struct InvalidSession {
    pub(super) session: Session,
}

impl InvalidSession {
    pub fn reauthorize(&self) {
        authorize()
    }

    pub fn unauthorize(&self) {
        unauthorize(&self.session.atom_ref)
    }

    pub fn authorization(&self) -> &Authorization {
        &self.session.authorization
    }
}
