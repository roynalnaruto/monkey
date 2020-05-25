use std::path::Path;
use std::sync::Arc;
use std::task::{Context, Poll};

use async_std::io;
use futures::{future, prelude::*};
use libp2p::{
    gossipsub::{MessageId, Topic},
    identity::Keypair,
    swarm::NetworkBehaviourAction::GenerateEvent,
    Multiaddr, PeerId, Swarm,
};
use tokio::{
    runtime::Handle,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use void::Void;

use crate::behaviour::{
    types::{BehaviourEvent, GossipsubMessage},
    Behaviour,
};
use crate::block::Block;
use crate::display::Display;
use crate::errors::Error;
use crate::store::DiscStore;

mod handler;
use handler::{Handler, HandlerMessage};

pub struct Service {
    local_keypair: Keypair,

    #[allow(dead_code)]
    store: Arc<DiscStore>,

    swarm: Swarm<Behaviour>,
    handler_send: UnboundedSender<HandlerMessage>,
    service_recv: UnboundedReceiver<ServiceMessage>,
}

#[derive(Debug)]
pub enum ServiceMessage {
    NewBlock(Block),
    PropagateGossip(MessageId, PeerId),
}

impl Service {
    pub fn new(rt_handle: &Handle, store_path: &Path) -> Result<Self, Error> {
        let disc_store = DiscStore::open(&store_path)?;
        let store = Arc::new(disc_store);

        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        let transport = libp2p::build_development_transport(keypair.clone())?;
        let behaviour = Behaviour::new(&peer_id);
        let swarm = Swarm::new(transport, behaviour, peer_id);

        let (service_send, service_recv) = mpsc::unbounded_channel::<ServiceMessage>();
        let handler_send = Handler::new(&rt_handle, &store, service_send.clone());

        Ok(Service {
            local_keypair: keypair,
            store: store,
            swarm: swarm,
            handler_send: handler_send,
            service_recv: service_recv,
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
            debug!("Dialed {:?}", addr);
        }

        Swarm::listen_on(&mut self.swarm, "/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();

        let mut stdin = io::BufReader::new(io::stdin()).lines();

        let mut listening = false;
        rt_handle.block_on(future::poll_fn(move |cx: &mut Context| {
            loop {
                match stdin.try_poll_next_unpin(cx)? {
                    Poll::Ready(Some(line)) => {
                        self.handler_send
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
                            Display::init().unwrap();
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
                            debug!("Peer {} subscribed to {}", peer_id, topic_hash);
                        }
                        BehaviourEvent::PeerUnsubscribed(peer_id, topic_hash) => {
                            debug!("Peer {} unsubscribed to {}", peer_id, topic_hash);
                        }
                        BehaviourEvent::GossipsubMessage {
                            id,
                            source,
                            message,
                        } => {
                            debug!("Gossipsub message {} from {}: {:?}", id, source, message);
                            self.handler_send
                                .send(HandlerMessage::Publish(id, source, message))?;
                        }
                    },
                    Poll::Ready(unhandled_event) => {
                        debug!("Found unhandled event: {:?}", unhandled_event);
                    }
                }
            }

            loop {
                match self.service_recv.try_recv() {
                    Ok(service_msg) => {
                        self.handle_message(service_msg)?;
                    }
                    Err(..) => break,
                }
            }

            Poll::Pending
        }))
    }

    fn handle_message(
        &mut self,
        service_msg: ServiceMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match service_msg {
            ServiceMessage::NewBlock(block) => {
                let keypair = match &self.local_keypair {
                    Keypair::Ed25519(kp) => kp,
                    _ => panic!("Only Ed25519 scheme is supported"),
                };

                let signed_block = block.sign(&keypair);
                let msg = GossipsubMessage::Block(signed_block.clone());

                match msg.encode() {
                    Ok(encoded_msg) => {
                        let topic = Topic::new("monkey-chain".into());
                        self.swarm.publish(&topic, &encoded_msg);

                        self.handler_send
                            .send(HandlerMessage::OwnBlock(signed_block))?;
                    }
                    Err(e) => {
                        error!("Failed to encode Gossipsub message: {:?}", e);
                    }
                }
            }
            ServiceMessage::PropagateGossip(id, source) => {
                self.swarm.progagate_message(&id, &source);
            }
        };

        Ok(())
    }
}
