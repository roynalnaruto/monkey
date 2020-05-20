use std::path::Path;
use std::sync::{Arc, Mutex};

use rusty_leveldb::{Options, DB};

use crate::errors::StoreError;

pub struct DiscStore {
    db: Arc<Mutex<DB>>,
}

impl DiscStore {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let db = DB::open(&path, Options::default())?;

        Ok(DiscStore {
            db: Arc::new(Mutex::new(db)),
        })
    }

    pub fn put(&self, k: &[u8], v: &[u8]) -> Result<(), StoreError> {
        let store = Arc::clone(&self.db);
        let mut db = store.lock().unwrap();

        db.put(k, v)?;

        Ok(())
    }

    pub fn get(&self, k: &[u8]) -> Option<Vec<u8>> {
        let store = Arc::clone(&self.db);
        let mut db = store.lock().unwrap();

        db.get(k)
    }

    pub fn flush(&self) -> Result<(), StoreError> {
        let store = Arc::clone(&self.db);
        let mut db = store.lock().unwrap();

        db.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    lazy_static! {
        static ref DISC_STORE: DiscStore = {
            let path = Path::new(".data").join(".test").join("testdata");
            let store = DiscStore::open(&path).ok().unwrap();

            store
        };
    }

    #[test]
    fn test_put() {
        assert!(DISC_STORE.put(&[1, 2], &[2, 3]).is_ok());
    }

    #[test]
    fn test_get() {
        assert!(DISC_STORE.put(&[100, 101], &[11, 13]).is_ok());
        assert_eq!(DISC_STORE.get(&[100, 101]), Some(vec![11, 13]));
        assert!(DISC_STORE.get(&[10, 11]).is_none());
    }
}
