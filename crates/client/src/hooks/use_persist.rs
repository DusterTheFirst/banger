use std::{
    fmt::{self, Debug, Display},
    ops::Deref,
    rc::Rc,
};

use dioxus::{
    core::{ScopeId, ScopeState},
    fermi::{use_atom_root, AtomId, AtomRoot, Readable},
};
use gloo_storage::{LocalStorage, Storage};
use serde::{de::DeserializeOwned, Serialize};
use tracing::warn;

use crate::atoms::persist::PersistAtom;

pub fn use_persist<T: 'static + Serialize + DeserializeOwned>(
    cx: &ScopeState,
    atom: PersistAtom<T>,
) -> &UsePersistAtom<T> {
    let root = use_atom_root(cx);

    let (_, persist) = cx.use_hook(|_| {
        root.initialize(atom);

        (
            PersistAtomSubscription {
                id: atom.unique_id(),
                root: root.clone(),
                scope_id: cx.scope_id(),
            },
            UsePersistAtom {
                id: atom.unique_id(),
                key: atom.key(),
                root: root.clone(),
                scope_id: cx.scope_id(),
                value: root.register(atom, cx.scope_id()),
            },
        )
    });

    // Update the value
    persist.value = root.register(atom, cx.scope_id());

    persist
}

pub struct PersistAtomSubscription {
    id: AtomId,
    root: Rc<AtomRoot>,
    scope_id: ScopeId,
}

impl Drop for PersistAtomSubscription {
    fn drop(&mut self) {
        self.root.unsubscribe(self.id, self.scope_id)
    }
}

pub struct UsePersistAtom<T: Serialize + DeserializeOwned + 'static> {
    id: AtomId,
    key: &'static str,
    value: Rc<T>,
    root: Rc<AtomRoot>,
    scope_id: ScopeId,
}

impl<T: Serialize + DeserializeOwned + 'static> Clone for UsePersistAtom<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            key: self.key,
            value: self.value.clone(),
            root: self.root.clone(),
            scope_id: self.scope_id,
        }
    }
}

impl<T: Serialize + DeserializeOwned + 'static> UsePersistAtom<T> {
    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn get_rc(&self) -> Rc<T> {
        self.value.clone()
    }
}

impl<T: Serialize + DeserializeOwned + 'static> Deref for UsePersistAtom<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Serialize + DeserializeOwned + Debug + 'static> Debug for UsePersistAtom<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: Serialize + DeserializeOwned + Display + 'static> Display for UsePersistAtom<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: Serialize + DeserializeOwned + Clone + 'static> UsePersistAtom<T> {
    pub fn set(&self, new: T) {
        self.root.force_update(self.id);
        self.root.set(self.id, new.clone());

        if let Err(error) = LocalStorage::set(self.key, new) {
            warn!(%error, key = self.key, "encountered an error when storing a PersistAtom");
        };
    }
}
