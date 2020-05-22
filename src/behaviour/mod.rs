use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::task::Poll;

use libp2p::{
    gossipsub::{Gossipsub, GossipsubConfigBuilder, GossipsubEvent, GossipsubMessage, MessageId},
    swarm::{NetworkBehaviourAction, NetworkBehaviourEventProcess},
    NetworkBehaviour, PeerId,
};

mod types;

use crate::behaviour::types::{BehaviourEvent, GossipsubMessage as DecodedMessage};

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    gossipsub: Gossipsub,

    #[behaviour(ignore)]
    events: Vec<BehaviourEvent>,
}

impl Behaviour {
    pub fn new(peer_id: &PeerId) -> Self {
        let message_id_fn = |message: &GossipsubMessage| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            MessageId(s.finish().to_string())
        };

        let gossipsub_config = GossipsubConfigBuilder::new()
            .message_id_fn(message_id_fn)
            .build();

        Behaviour {
            gossipsub: Gossipsub::new(peer_id.clone(), gossipsub_config),
            events: Vec::<BehaviourEvent>::new(),
        }
    }

    #[allow(dead_code)]
    pub fn poll<TBehaviourIn>(
        &mut self,
    ) -> Poll<NetworkBehaviourAction<TBehaviourIn, BehaviourEvent>> {
        if !self.events.is_empty() {
            return Poll::Ready(NetworkBehaviourAction::GenerateEvent(self.events.remove(0)));
        }

        Poll::Pending
    }
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for Behaviour {
    fn inject_event(&mut self, event: GossipsubEvent) {
        match event {
            GossipsubEvent::Message(source, id, gossipsub_msg) => {
                match DecodedMessage::decode(&gossipsub_msg.data) {
                    Ok(msg) => {
                        self.events.push(BehaviourEvent::GossipsubMessage {
                            id,
                            source: source,
                            message: msg,
                        });
                    }
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                    }
                }
            }
            GossipsubEvent::Subscribed { peer_id, topic } => {
                self.events
                    .push(BehaviourEvent::PeerSubscribed(peer_id, topic));
            }
            GossipsubEvent::Unsubscribed { peer_id, topic } => {
                self.events
                    .push(BehaviourEvent::PeerUnsubscribed(peer_id, topic));
            }
        }
    }
}
