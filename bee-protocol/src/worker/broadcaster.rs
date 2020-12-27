// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    packet::Message,
    peer::PeerManager,
    worker::{MetricsWorker, PeerManagerWorker},
    ProtocolMetrics, Sender,
};

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_pt2::{node::Node, worker::Worker};
use bee_network::{NetworkController, PeerId};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;
use tokio::sync::mpsc;

use std::{any::TypeId, convert::Infallible};

pub(crate) struct BroadcasterWorkerEvent {
    pub(crate) source: Option<PeerId>,
    pub(crate) message: Message,
}

pub(crate) struct BroadcasterWorker {
    pub(crate) tx: mpsc::UnboundedSender<BroadcasterWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for BroadcasterWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<MetricsWorker>(), TypeId::of::<PeerManagerWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let network = node.resource::<NetworkController>();
        let metrics = node.resource::<ProtocolMetrics>();
        let peer_manager = node.resource::<PeerManager>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(BroadcasterWorkerEvent { source, message }) = receiver.next().await {
                peer_manager.for_each_peer(|peer_id, _| {
                    if match source {
                        Some(ref source) => source != peer_id,
                        None => true,
                    } {
                        Sender::<Message>::send(&network, &peer_manager, &metrics, peer_id, message.clone());
                    }
                });
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
