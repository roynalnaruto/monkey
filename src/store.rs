use std::path::Path;

use rusty_leveldb::{Options, DB};

use crate::errors::StoreError;

pub struct DiscStore {
    db: DB,
}

impl DiscStore {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let db = DB::open(&path, Options::default())?;

        Ok(DiscStore { db: db })
    }

    pub fn put(&mut self, k: &[u8], v: &[u8]) -> Result<(), StoreError> {
        self.db.put(k, v)?;

        Ok(())
    }

    pub fn get(&mut self, k: &[u8]) -> Option<Vec<u8>> {
        self.db.get(k)
    }

    pub fn flush(&mut self) -> Result<(), StoreError> {
        self.db.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    lazy_static! {
        static ref DISC_STORE: Arc<Mutex<DiscStore>> = {
            let path = Path::new(".data").join(".test").join("testdata");
            let store = DiscStore::open(&path).ok().unwrap();

            Arc::new(Mutex::new(store))
        };
    }

    #[test]
    fn test_put() {
        let store = Arc::clone(&DISC_STORE);
        let mut store = store.lock().unwrap();

        assert!(store.put(&[1, 2], &[2, 3]).is_ok());
    }

    #[test]
    fn test_get() {
        let store = Arc::clone(&DISC_STORE);
        let mut store = store.lock().unwrap();

        assert!(store.put(&[100, 101], &[11, 13]).is_ok());

        let v = store.get(&[100, 101]);
        assert_eq!(v, Some(vec![11, 13]));

        assert!(store.get(&[10, 11]).is_none());
    }
}
