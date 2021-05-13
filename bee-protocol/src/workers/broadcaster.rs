// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::metrics::NodeMetrics,
    workers::{packets::Message, peer::PeerManager, sender::Sender, MetricsWorker, PeerManagerResWorker},
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
        vec![TypeId::of::<MetricsWorker>(), TypeId::of::<PeerManagerResWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let metrics = node.resource::<NodeMetrics>();
        let peer_manager = node.resource::<PeerManager>();

        node.spawn::<Self, _, _>(file!(), line!(), |shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, UnboundedReceiverStream::new(rx));

            while let Some(BroadcasterWorkerEvent { source, message }) = receiver.next().await {
                // TODO bring it back
                // peer_manager.for_each_peer(|peer_id, _| {
                for (peer_id, _) in peer_manager.peers.read().await.iter() {
                    if match source {
                        Some(ref source) => source != peer_id,
                        None => true,
                    } {
                        Sender::<Message>::send(&peer_manager, &metrics, peer_id, message.clone()).await;
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
