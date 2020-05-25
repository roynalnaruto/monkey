use std::sync::Arc;

use bincode::serialize;
use libp2p::{gossipsub::MessageId, identity::PublicKey, PeerId};
use tokio::{
    runtime::Handle,
    sync::mpsc::{self, UnboundedSender},
};

use crate::behaviour::types::GossipsubMessage;
use crate::block::{Block, SignedBlock};
use crate::display::Display;
use crate::errors::Error;
use crate::service::ServiceMessage;
use crate::store::DiscStore;

pub struct Handler {
    store: Arc<DiscStore>,
    service_send: UnboundedSender<ServiceMessage>,
    display: Display,
}

#[derive(Debug)]
pub enum HandlerMessage {
    Publish(MessageId, PeerId, GossipsubMessage),
    OwnBlock(SignedBlock),
    Stdin(String, PublicKey),
}

impl Handler {
    pub fn new(
        rt_handle: &Handle,
        store: &Arc<DiscStore>,
        service_send: UnboundedSender<ServiceMessage>,
    ) -> UnboundedSender<HandlerMessage> {
        let (handler_send, mut handler_recv) = mpsc::unbounded_channel::<HandlerMessage>();

        let display = Display::new();

        let mut handler = Handler {
            service_send: service_send,
            store: Arc::clone(store),
            display: display,
        };

        if let Err(e) = handler.import_genesis() {
            panic!("Error inserting genesis block: {:?}", e)
        }

        rt_handle.spawn_blocking(move || loop {
            match handler_recv.try_recv() {
                Ok(handler_msg) => handler.handle_message(handler_msg),
                Err(..) => {}
            }
        });

        handler_send
    }

    fn handle_message(&mut self, handler_msg: HandlerMessage) {
        match handler_msg {
            HandlerMessage::Stdin(msg, public_key) => {
                let proposer = match public_key {
                    PublicKey::Ed25519(pk) => pk,
                    _ => panic!("Only Ed25519 scheme is supported"),
                };

                let wordlist: Vec<String> = msg
                    .split_ascii_whitespace()
                    .map(|w| w.to_lowercase())
                    .collect();

                // TODO: this will be removed once we have a state
                // that maintains the longest chain and parent_hash
                // is the block hash of the last inserted block
                let (dummy_parent_hash, _, _) = Block::genesis_block();
                match Block::new(wordlist, proposer, dummy_parent_hash) {
                    Ok(block) => {
                        if let Err(e) = self.service_send.send(ServiceMessage::NewBlock(block)) {
                            error!("Error sending message between Handler and Service: {:?}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Invalid block: {:?}", e);

                        Display::notice_invalid_block().unwrap();
                    }
                }
            }
            HandlerMessage::OwnBlock(signed_block) => match self.import_block(&signed_block) {
                Ok(()) => {
                    info!("Inserted own block {:?}", signed_block.message.hash);

                    self.display.new_block(signed_block.display()).unwrap();
                    Display::notice_valid_block().unwrap();
                }
                Err(e) => warn!("Ignoring invalid own block: {:?}", e),
            },
            HandlerMessage::Publish(id, source, msg) => match msg {
                GossipsubMessage::Block(signed_block) => match self.import_block(&signed_block) {
                    Ok(()) => {
                        info!("Inserted published block {:?}", signed_block.message.hash);

                        if let Err(e) = self
                            .service_send
                            .send(ServiceMessage::PropagateGossip(id, source))
                        {
                            error!("Error sending message between Handler and Service: {:?}", e);
                        }
                    }
                    Err(e) => warn!("Ignoring invalid published block: {:?}", e),
                },
            },
        };
    }

    fn import_genesis(&self) -> Result<(), Error> {
        let (_, genesis_block_hash, genesis_block) = Block::genesis_block();

        self.store.put(&genesis_block_hash, &genesis_block)?;

        Ok(())
    }

    fn import_block(&self, signed_block: &SignedBlock) -> Result<(), Error> {
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
