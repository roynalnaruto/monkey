use libp2p::{
    gossipsub::{Gossipsub, GossipsubConfig, GossipsubEvent},
    swarm::NetworkBehaviourEventProcess,
    NetworkBehaviour, PeerId,
};

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    gossipsub: Gossipsub,
}

impl Behaviour {
    #[allow(dead_code)]
    pub fn new(peer_id: &PeerId) -> Result<Self, String> {
        Ok(Behaviour {
            gossipsub: Gossipsub::new(peer_id.clone(), GossipsubConfig::default()),
        })
    }
}

impl NetworkBehaviourEventProcess<GossipsubEvent> for Behaviour {
    fn inject_event(&mut self, _event: GossipsubEvent) {
        unimplemented!();
    }
}
