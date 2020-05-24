use libp2p::{gossipsub::MessageId, identity::PublicKey, PeerId};
use tokio::{
    runtime::Handle,
    sync::mpsc::{self, UnboundedSender},
};

use crate::behaviour::types::GossipsubMessage;
use crate::block::Block;
use crate::service::ServiceMessage;

pub struct Handler {
    service_send: UnboundedSender<ServiceMessage>,
}

#[derive(Debug)]
pub enum HandlerMessage {
    Publish(MessageId, PeerId, GossipsubMessage),
    Stdin(String, PublicKey),
}

impl Handler {
    pub fn new(
        rt_handle: &Handle,
        service_send: UnboundedSender<ServiceMessage>,
    ) -> UnboundedSender<HandlerMessage> {
        let (handler_send, mut handler_recv) = mpsc::unbounded_channel::<HandlerMessage>();

        let handler = Handler {
            service_send: service_send,
        };

        rt_handle.spawn_blocking(move || loop {
            match handler_recv.try_recv() {
                Ok(handler_msg) => handler.handle_message(handler_msg),
                Err(..) => {}
            }
        });

        handler_send
    }

    #[allow(unused_variables)]
    fn handle_message(&self, handler_msg: HandlerMessage) {
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
                    Err(..) => {
                        warn!("Invalid block");
                    }
                }
            }
            HandlerMessage::Publish(id, source, msg) => {
                // TODO: verify signature of signed block
                // import to blockchain if its valid
                // reject otherwise
                todo!();
            }
        };
    }
}
