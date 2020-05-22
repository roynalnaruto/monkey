use bincode::{deserialize, serialize};
use libp2p::{
    gossipsub::{MessageId, TopicHash},
    PeerId,
};
use serde::{Deserialize, Serialize};

use crate::block::SignedBlock;
use crate::errors::Error;

#[derive(Debug)]
pub enum BehaviourEvent {
    GossipsubMessage {
        id: MessageId,
        source: PeerId,
        message: GossipsubMessage,
    },
    PeerSubscribed(PeerId, TopicHash),
    PeerUnsubscribed(PeerId, TopicHash),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GossipsubMessage {
    Block(Box<SignedBlock>),
}

impl GossipsubMessage {
    #[allow(dead_code)]
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        let bytes: Vec<u8> = serialize(&self)?;

        Ok(bytes)
    }

    pub fn decode(data: &[u8]) -> Result<Self, Error> {
        let instance: Self = deserialize(&data[..])?;

        Ok(instance)
    }
}
