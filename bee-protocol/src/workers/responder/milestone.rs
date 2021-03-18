// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    peer::PeerManager,
    storage::StorageBackend,
    types::{
        metrics::NodeMetrics,
        packets::{Message as MessagePacket, MilestoneRequest},
    },
    workers::{MetricsWorker, PeerManagerResWorker},
    Sender,
};

use bee_common::packable::Packable;
use bee_network::PeerId;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::{MsTangle, TangleWorker};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use std::{any::TypeId, convert::Infallible};

pub(crate) struct MilestoneResponderWorkerEvent {
    pub(crate) peer_id: PeerId,
    pub(crate) request: MilestoneRequest,
}

pub(crate) struct MilestoneResponderWorker {
    pub(crate) tx: mpsc::UnboundedSender<MilestoneResponderWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for MilestoneResponderWorker
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

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let metrics = node.resource::<NodeMetrics>();
        let peer_manager = node.resource::<PeerManager>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(MilestoneResponderWorkerEvent { peer_id, request }) = receiver.next().await {
                let index = if request.index == 0 {
                    tangle.get_latest_milestone_index()
                } else {
                    request.index.into()
                };

                if let Some(message) = tangle.get_milestone(index).await {
                    Sender::<MessagePacket>::send(
                        &peer_manager,
                        &metrics,
                        &peer_id,
                        MessagePacket::new(&message.pack_new()),
                    )
                    .await;
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
