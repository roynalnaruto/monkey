use std::path::Path;
use std::sync::Arc;
use std::task::{Context, Poll};

use async_std::io;
use bincode::serialize;
use futures::{future, prelude::*};
use libp2p::{gossipsub::Topic, Multiaddr, PeerId, Swarm};
use tokio::runtime::Handle;
use void::Void;

use crate::behaviour::Behaviour;
use crate::block::{Block, SignedBlock};
use crate::errors::Error;
use crate::handler::Handler;
use crate::store::DiscStore;

pub struct Service {
    store: Arc<DiscStore>,

    #[allow(dead_code)]
    swarm: Swarm<Behaviour>,
}

impl Service {
    pub fn new(store_path: &Path) -> Result<Self, Error> {
        let store = DiscStore::open(&store_path)?;

        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        let transport = libp2p::build_development_transport(keypair)?;
        let behaviour = Behaviour::new(&peer_id);
        let swarm = Swarm::new(transport, behaviour, peer_id);

        Ok(Service {
            store: Arc::new(store),
            swarm: swarm,
        })
    }

    #[allow(unused_variables)]
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

        // TODO: add MPSC between service and handler
        // for handling stdin and gossipsub events
        let handler = Handler::new(&rt_handle);

        let mut listening = false;
        rt_handle.block_on(future::poll_fn(move |cx: &mut Context| {
            loop {
                match stdin.try_poll_next_unpin(cx)? {
                    Poll::Ready(Some(line)) => {
                        self.swarm.publish(&topic, line.as_bytes());
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
                    Poll::Ready(msg) => info!("polled event = {:?}", msg),
                }
            }

            Poll::Pending
        }))
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
