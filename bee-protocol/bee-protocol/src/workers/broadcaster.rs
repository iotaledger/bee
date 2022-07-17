// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{any::TypeId, convert::Infallible};

use async_trait::async_trait;
use bee_gossip::PeerId;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use futures::stream::StreamExt;
use log::info;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    types::metrics::NodeMetrics,
    workers::{packets::BlockPacket, peer::PeerManager, sender::Sender, MetricsWorker, PeerManagerResWorker},
};

pub(crate) struct BroadcasterWorkerEvent {
    pub(crate) source: Option<PeerId>,
    pub(crate) block: BlockPacket,
}

pub(crate) struct BroadcasterWorker {
    pub(crate) tx: mpsc::UnboundedSender<BroadcasterWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for BroadcasterWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<PeerManagerResWorker>(), TypeId::of::<MetricsWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let peer_manager = node.resource::<PeerManager>();
        let metrics = node.resource::<NodeMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(BroadcasterWorkerEvent { source, block }) = receiver.next().await {
                peer_manager.for_each(|peer_id, _| {
                    if source.map_or(true, |ref source| peer_id != source) {
                        Sender::<BlockPacket>::send(&block, peer_id, &peer_manager, &metrics);
                    }
                });
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
