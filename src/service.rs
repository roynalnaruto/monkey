use std::path::Path;
use std::sync::Arc;

use bincode::serialize;
use libp2p::{PeerId, Swarm};

use crate::behaviour::Behaviour;
use crate::block::{Block, SignedBlock};
use crate::errors::Error;
use crate::store::DiscStore;

pub struct Service {
    store: Arc<DiscStore>,

    #[allow(dead_code)]
    swarm: Arc<Swarm<Behaviour>>,
}

impl Service {
    pub fn new(store_path: &Path) -> Result<Self, Error> {
        let store = DiscStore::open(&store_path)?;

        // TODO: refactor
        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        let transport = libp2p::build_development_transport(keypair)?;
        let behaviour = Behaviour::new(&peer_id).ok().unwrap();
        let swarm = Swarm::new(transport, behaviour, peer_id);
        // TODO: refactor

        Ok(Service {
            store: Arc::new(store),
            swarm: Arc::new(swarm),
        })
    }

    pub fn import_genesis(&self) -> Result<(), Error> {
        let (genesis_block_hash, genesis_block) = Block::genesis_block();

        self.store.put(&genesis_block_hash, &genesis_block)?;

        Ok(())
    }

    pub fn import_block(&self, signed_block: &SignedBlock) -> Result<(), Error> {
        signed_block.message.clone().validate()?;

        match signed_block.verify_signature() {
            true => {
                let block_hash = signed_block.message.hash.to_be_bytes();
                let parent_hash = signed_block.message.parent_hash.to_be_bytes();

                if let None = self.store.get(&parent_hash) {
                    return Err(Error::UnknownParentBlock);
                }

                if let Some(_) = self.store.get(&block_hash) {
                    return Err(Error::DuplicateBlock);
                }

                let signed_block_bytes = serialize(&signed_block).unwrap();
                self.store.put(&block_hash, &signed_block_bytes)?;

                Ok(())
            }
            false => Err(Error::InvalidSignature),
        }
    }
}
