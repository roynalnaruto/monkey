use std::path::Path;

use bincode::serialize;

use crate::block::SignedBlock;
use crate::errors::{BlockError, StoreError};
use crate::store::DiscStore;

pub struct Service {
    store: DiscStore,
}

impl Service {
    pub fn new(store_path: &Path) -> Result<Self, StoreError> {
        let store = DiscStore::open(&store_path)?;

        Ok(Service { store: store })
    }

    pub fn import_block(&self, signed_block: &SignedBlock) -> Result<(), BlockError> {
        signed_block.message.clone().validate()?;

        match signed_block.verify_signature() {
            true => {
                let block_hash = signed_block.message.hash.to_be_bytes();
                let parent_hash = signed_block.message.parent_hash.to_be_bytes();

                if let None = self.store.get(&parent_hash) {
                    return Err(BlockError::UnknownParentBlock);
                }

                if let Some(_) = self.store.get(&block_hash) {
                    return Err(BlockError::DuplicateBlock);
                }

                let signed_block_bytes = serialize(&signed_block).unwrap();
                self.store.put(&block_hash, &signed_block_bytes)?;

                Ok(())
            }
            false => Err(BlockError::InvalidSignature),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;

    use libp2p::identity::ed25519::Keypair;

    use crate::block::Block;

    lazy_static! {
        static ref GENESIS_DATA: u16 = 1337;

        static ref SERVICE: Service = {
            let path = Path::new(".data").join(".test").join("servicedata");
            let service = Service::new(&path).ok().unwrap();

            // insert genesis block
            // TODO: decide on a more sane way of achieving this
            {
                let mut hasher = DefaultHasher::new();
                hasher.write_u16(*GENESIS_DATA);
                let genesis_hash = hasher.finish();
                service.store.put(&genesis_hash.to_be_bytes(), &[0, 1, 0, 1, 0]).unwrap();
            }

            service
        };
    }

    #[test]
    fn test_import_block() {
        let wordlist = vec![
            "and".to_string(),
            "for".to_string(),
            "that".to_string(),
            "this".to_string(),
        ];
        let proposer = Keypair::generate();
        let mut hasher = DefaultHasher::new();
        hasher.write_u16(*GENESIS_DATA);
        let parent_hash = hasher.finish();

        let result = Block::new(wordlist, proposer.public(), parent_hash);
        let block = result.ok().unwrap();
        let signed_block = block.clone().sign(&proposer);

        assert!(SERVICE.import_block(&signed_block).is_ok());
    }

    #[test]
    fn test_import_block_invalid_parent() {
        let wordlist = vec![
            "and".to_string(),
            "for".to_string(),
            "that".to_string(),
            "this".to_string(),
        ];
        let proposer = Keypair::generate();
        let mut hasher = DefaultHasher::new();
        hasher.write_u16(7331);
        let parent_hash = hasher.finish();

        let result = Block::new(wordlist, proposer.public(), parent_hash);
        let block = result.ok().unwrap();
        let signed_block = block.clone().sign(&proposer);

        let result = SERVICE.import_block(&signed_block);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), BlockError::UnknownParentBlock);
    }

    #[test]
    fn test_import_duplicate_block() {
        let wordlist = vec![
            "never".to_string(),
            "ever".to_string(),
            "where".to_string(),
            "when".to_string(),
        ];
        let proposer = Keypair::generate();
        let mut hasher = DefaultHasher::new();
        hasher.write_u16(*GENESIS_DATA);
        let parent_hash = hasher.finish();

        let result = Block::new(wordlist, proposer.public(), parent_hash);
        let block = result.ok().unwrap();
        let signed_block = block.clone().sign(&proposer);

        SERVICE.import_block(&signed_block).unwrap();

        assert!(SERVICE.import_block(&signed_block).is_err());
    }
}
