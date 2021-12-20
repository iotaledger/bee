// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::metrics::NodeMetrics,
    workers::{packets::MessagePacket, peer::PeerManager, sender::Sender, MetricsWorker, PeerManagerResWorker},
};

use bee_network::PeerId;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, convert::Infallible};

pub(crate) struct BroadcasterWorkerEvent {
    pub(crate) source: Option<PeerId>,
    pub(crate) message: MessagePacket,
}

pub(crate) struct BroadcasterWorker {
    pub(crate) tx: mpsc::UnboundedSender<BroadcasterWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for BroadcasterWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<MetricsWorker>(), TypeId::of::<PeerManagerResWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let metrics = node.resource::<NodeMetrics>();
        let peer_manager = node.resource::<PeerManager>();

        let (tx, rx) = mpsc::unbounded_channel();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(BroadcasterWorkerEvent { source, message }) = receiver.next().await {
                peer_manager.for_each(|peer_id, _| {
                    if source.map_or(true, |ref source| peer_id != source) {
                        Sender::<MessagePacket>::send(&message, peer_id, &peer_manager, &metrics);
                    }
                });
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
