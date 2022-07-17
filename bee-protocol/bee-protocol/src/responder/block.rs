// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{any::TypeId, convert::Infallible};

use async_trait::async_trait;
use bee_gossip::PeerId;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{Tangle, TangleWorker};
use futures::stream::StreamExt;
use log::info;
use packable::PackableExt;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    types::metrics::NodeMetrics,
    {
        packets::{BlockPacket, BlockRequestPacket},
        peer::PeerManager,
        sender::Sender,
        storage::StorageBackend,
        MetricsWorker, PeerManagerResWorker,
    },
};

pub(crate) struct BlockResponderWorkerEvent {
    pub(crate) peer_id: PeerId,
    pub(crate) request: BlockRequestPacket,
}

pub(crate) struct BlockResponderWorker {
    pub(crate) tx: UnboundedSender<BlockResponderWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for BlockResponderWorker
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

            while let Some(BlockResponderWorkerEvent { peer_id, request }) = receiver.next().await {
                if let Some(block) = tangle.get(&request.block_id) {
                    Sender::<BlockPacket>::send(
                        &BlockPacket::new(block.pack_to_vec()),
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
