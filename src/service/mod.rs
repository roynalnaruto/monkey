use std::path::Path;
use std::sync::Arc;
use std::task::{Context, Poll};

use async_std::io;
use bincode::serialize;
use futures::{future, prelude::*};
use libp2p::{
    gossipsub::Topic, identity::Keypair, swarm::NetworkBehaviourAction::GenerateEvent, Multiaddr,
    PeerId, Swarm,
};
use tokio::{runtime::Handle, sync::mpsc};
use void::Void;

use crate::behaviour::{
    types::{BehaviourEvent, GossipsubMessage},
    Behaviour,
};
use crate::block::{Block, SignedBlock};
use crate::errors::Error;
use crate::store::DiscStore;

mod handler;
use handler::{Handler, HandlerMessage};

pub struct Service {
    local_keypair: Keypair,
    store: Arc<DiscStore>,
    swarm: Swarm<Behaviour>,
}

#[derive(Debug)]
pub enum ServiceMessage {
    NewBlock(Block),
}

impl Service {
    pub fn new(store_path: &Path) -> Result<Self, Error> {
        let store = DiscStore::open(&store_path)?;

        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        let transport = libp2p::build_development_transport(keypair.clone())?;
        let behaviour = Behaviour::new(&peer_id);
        let swarm = Swarm::new(transport, behaviour, peer_id);

        Ok(Service {
            local_keypair: keypair,
            store: Arc::new(store),
            swarm: swarm,
        })
    }

    pub fn start(
        &mut self,
        rt_handle: &Handle,
        to_dial: Option<Multiaddr>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let topic = Topic::new("monkey-chain".into());
        self.swarm.subscribe(&topic);

        if let Some(addr) = to_dial {
            Swarm::dial_addr(&mut self.swarm, addr.clone())?;
            info!("Dialed {:?}", addr);
        }

        Swarm::listen_on(&mut self.swarm, "/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

        let mut stdin = io::BufReader::new(io::stdin()).lines();

        let (service_send, mut service_recv) = mpsc::unbounded_channel::<ServiceMessage>();
        let handler_send = Handler::new(&rt_handle, service_send.clone());

        let mut listening = false;
        rt_handle.block_on(future::poll_fn(move |cx: &mut Context| {
            loop {
                match stdin.try_poll_next_unpin(cx)? {
                    Poll::Ready(Some(line)) => {
                        handler_send
                            .send(HandlerMessage::Stdin(line, self.local_keypair.public()))?;
                    }
                    Poll::Ready(None) => panic!("Stdin closed"),
                    Poll::Pending => break,
                }
            }

            loop {
                match self.swarm.poll_next_unpin(cx) {
                    Poll::Ready(Some(event)) => debug!("{:?}", event),
                    Poll::Ready(None) => return Poll::Ready(Ok(())),
                    Poll::Pending => {
                        if !listening {
                            for addr in Swarm::listeners(&self.swarm) {
                                info!("Listening on {:?}", addr);
                                listening = true;
                            }
                        }
                        break;
                    }
                }
            }

            loop {
                match self.swarm.poll::<Void>() {
                    Poll::Pending => break,
                    Poll::Ready(GenerateEvent(event)) => match event {
                        BehaviourEvent::PeerSubscribed(peer_id, topic_hash) => {
                            info!("Peer {} subscribed to {}", peer_id, topic_hash);
                        }
                        BehaviourEvent::PeerUnsubscribed(peer_id, topic_hash) => {
                            info!("Peer {} unsubscribed to {}", peer_id, topic_hash);
                        }
                        BehaviourEvent::GossipsubMessage {
                            id,
                            source,
                            message,
                        } => {
                            info!("Gossipsub message {} from {}: {:?}", id, source, message);
                            handler_send.send(HandlerMessage::Publish(id, source, message))?;
                        }
                    },
                    Poll::Ready(unhandled_event) => {
                        debug!("Found unhandled event: {:?}", unhandled_event);
                    }
                }
            }

            loop {
                match service_recv.try_recv() {
                    Ok(service_msg) => self.handle_message(service_msg),
                    Err(..) => break,
                }
            }

            Poll::Pending
        }))
    }

    #[allow(unused_variables)]
    fn handle_message(&mut self, service_msg: ServiceMessage) {
        match service_msg {
            ServiceMessage::NewBlock(block) => {
                let keypair = match &self.local_keypair {
                    Keypair::Ed25519(kp) => kp,
                    _ => panic!("Only Ed25519 scheme is supported"),
                };

                let signed_block = block.sign(&keypair);
                let msg = GossipsubMessage::Block(signed_block);

                match msg.encode() {
                    Ok(encoded_msg) => {
                        let topic = Topic::new("monkey-chain".into());
                        self.swarm.publish(&topic, &encoded_msg);

                        // TODO: import signed block in own store
                        todo!();
                    }
                    Err(e) => {
                        error!("Failed to encode Gossipsub message: {:?}", e);
                    }
                }
            }
        };
    }

    pub fn import_genesis(&self) -> Result<(), Error> {
        let (_, genesis_block_hash, genesis_block) = Block::genesis_block();

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
