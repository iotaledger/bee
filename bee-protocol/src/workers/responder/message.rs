// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{any::TypeId, convert::Infallible};

use async_trait::async_trait;
use bee_common::packable::Packable;
use bee_gossip::PeerId;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{Tangle, TangleWorker};
use futures::stream::StreamExt;
use log::info;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    types::metrics::NodeMetrics,
    workers::{
        packets::{MessagePacket, MessageRequestPacket},
        peer::PeerManager,
        sender::Sender,
        storage::StorageBackend,
        MetricsWorker, PeerManagerResWorker,
    },
};

pub(crate) struct MessageResponderWorkerEvent {
    pub(crate) peer_id: PeerId,
    pub(crate) request: MessageRequestPacket,
}

pub(crate) struct MessageResponderWorker {
    pub(crate) tx: UnboundedSender<MessageResponderWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for MessageResponderWorker
where
    N::Backend: StorageBackend,
{
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<TangleWorker>(),
            TypeId::of::<MetricsWorker>(),
            TypeId::of::<PeerManagerResWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let tangle = node.resource::<Tangle<N::Backend>>();
        let metrics = node.resource::<NodeMetrics>();
        let peer_manager = node.resource::<PeerManager>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(MessageResponderWorkerEvent { peer_id, request }) = receiver.next().await {
                if let Some(message) = tangle.get(&request.message_id) {
                    Sender::<MessagePacket>::send(
                        &MessagePacket::new(message.pack_new()),
                        &peer_id,
                        &peer_manager,
                        &metrics,
                    );
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
