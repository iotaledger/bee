// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    packet::{Message as MessagePacket, MessageRequest},
    peer::PeerManager,
    storage::StorageBackend,
    worker::{MetricsWorker, PeerManagerResWorker, TangleWorker},
    ProtocolMetrics, Sender,
};

use bee_common::packable::Packable;
use bee_network::PeerId;
use bee_runtime::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_tangle::MsTangle;

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

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let metrics = node.resource::<ProtocolMetrics>();
        let peer_manager = node.resource::<PeerManager>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(MessageResponderWorkerEvent { peer_id, request }) = receiver.next().await {
                if let Some(message) = tangle.get(&request.message_id.into()).await.map(|m| (*m).clone()) {
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
