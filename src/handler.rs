use libp2p::{gossipsub::MessageId, PeerId};
use tokio::{
    runtime::Handle,
    sync::mpsc::{self, UnboundedSender},
};

use crate::behaviour::types::GossipsubMessage;
use crate::service::ServiceMessage;

#[allow(dead_code)]
pub struct Handler {
    service_send: UnboundedSender<ServiceMessage>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum HandlerMessage {
    Publish(MessageId, PeerId, GossipsubMessage),
    Stdin(String),
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
            HandlerMessage::Stdin(msg) => {
                // TODO: create, validate block and send back
                // to service to be progagated to the swarm
                todo!();
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
