use dioxus::fermi::{AtomId, AtomRoot, Readable};
use gloo_storage::{errors::StorageError, LocalStorage, Storage};
use serde::{de::DeserializeOwned, Serialize};
use tracing::warn;

pub struct PersistAtom<T: Serialize + DeserializeOwned> {
    key: &'static str,
    init: fn() -> T,
}

impl<T: Serialize + DeserializeOwned> Clone for PersistAtom<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            init: self.init,
        }
    }
}

impl<T: Serialize + DeserializeOwned> Copy for PersistAtom<T> {}

impl<T: Serialize + DeserializeOwned> PersistAtom<T> {
    pub const fn new(key: &'static str, init: fn() -> T) -> Self {
        Self { key, init }
    }

    pub fn init(&self) -> T {
        (self.init)()
    }

    pub fn key(&self) -> &'static str {
        self.key
    }
}

impl<T: Serialize + DeserializeOwned> Readable<T> for PersistAtom<T> {
    fn read(&self, _root: AtomRoot) -> Option<T> {
        unimplemented!()
    }

    fn init(&self) -> T {
        let local_storage = match LocalStorage::get(self.key) {
            Ok(value) => Some(value),
            Err(StorageError::KeyNotFound(_)) => None,
            Err(StorageError::JsError(error)) => {
                warn!(%error, key = self.key, "encountered a javascript error when loading PersistAtom");

                None
            }
            Err(StorageError::SerdeError(error)) => {
                warn!(%error, key = self.key, "encountered a deserialization error when loading PersistAtom");

                None
            }
        };

        local_storage.unwrap_or_else(self.init)
    }

    fn unique_id(&self) -> AtomId {
        self.init as *const ()
    }
}

#[test]
fn atom_compiles() {
    static TEST_ATOM: PersistAtom<Vec<String>> = PersistAtom::new("amogus", Vec::new);
    dbg!(TEST_ATOM.init());
}
