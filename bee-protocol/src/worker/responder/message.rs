// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    packet::{Message as MessagePacket, MessageRequest},
    peer::PeerManager,
    tangle::MsTangle,
    worker::{MetricsWorker, PeerManagerWorker, TangleWorker},
    ProtocolMetrics, Sender,
};

use bee_common::{node::Node, packable::Packable, shutdown_stream::ShutdownStream, worker::Worker};
use bee_message::MessageId;
use bee_network::{NetworkController, PeerId};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;
use tokio::sync::mpsc;

use std::{any::TypeId, convert::Infallible};

pub(crate) struct MessageResponderWorkerEvent {
    pub(crate) peer_id: PeerId,
    pub(crate) request: MessageRequest,
}

pub(crate) struct MessageResponderWorker {
    pub(crate) tx: mpsc::UnboundedSender<MessageResponderWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for MessageResponderWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TangleWorker>(),
            TypeId::of::<MetricsWorker>(),
            TypeId::of::<PeerManagerWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let network = node.resource::<NetworkController>();
        let metrics = node.resource::<ProtocolMetrics>();
        let peer_manager = node.resource::<PeerManager>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(MessageResponderWorkerEvent { peer_id, request }) = receiver.next().await {
                if let Some(message) = tangle.get(&MessageId::from(request.message_id)).await {
                    let mut bytes = Vec::new();

                    if message.pack(&mut bytes).is_ok() {
                        Sender::<MessagePacket>::send(
                            &network,
                            &peer_manager,
                            &metrics,
                            &peer_id,
                            MessagePacket::new(&bytes),
                        );
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
