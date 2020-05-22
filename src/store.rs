use std::path::Path;
use std::sync::Mutex;

use rusty_leveldb::{Options, DB};

use crate::errors::Error;

pub struct DiscStore {
    db: Mutex<DB>,
}

impl DiscStore {
    pub fn open(path: &Path) -> Result<Self, Error> {
        let db = DB::open(&path, Options::default())?;

        Ok(DiscStore { db: Mutex::new(db) })
    }

    pub fn put(&self, k: &[u8], v: &[u8]) -> Result<(), Error> {
        let mut db = self.db.lock().unwrap();

        db.put(k, v)?;

        Ok(())
    }

    pub fn get(&self, k: &[u8]) -> Option<Vec<u8>> {
        let mut db = self.db.lock().unwrap();

        db.get(k)
    }

    #[allow(dead_code)]
    pub fn flush(&self) -> Result<(), Error> {
        let mut db = self.db.lock().unwrap();

        db.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    lazy_static! {
        static ref DISC_STORE: Arc<DiscStore> = {
            let path = Path::new(".data").join(".test").join("storedata");
            let store = DiscStore::open(&path).ok().unwrap();

            Arc::new(store)
        };
    }

    #[test]
    fn test_put() {
        let disc_store = Arc::clone(&DISC_STORE);

        assert!(disc_store.put(&[1, 2], &[2, 3]).is_ok());
    }

    #[test]
    fn test_get() {
        let disc_store = Arc::clone(&DISC_STORE);

        assert!(disc_store.put(&[100, 101], &[11, 13]).is_ok());
        assert_eq!(disc_store.get(&[100, 101]), Some(vec![11, 13]));
        assert!(disc_store.get(&[10, 11]).is_none());
    }
}
