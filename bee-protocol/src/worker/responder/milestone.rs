// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    packet::{Message as MessagePacket, MilestoneRequest},
    protocol::Sender,
    tangle::MsTangle,
    worker::TangleWorker,
};

use bee_common::{node::Node, packable::Packable, shutdown_stream::ShutdownStream, worker::Worker};
use bee_message::MessageId;
use bee_network::{Network, PeerId};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;

use std::{any::TypeId, convert::Infallible};

pub(crate) struct MilestoneResponderWorkerEvent {
    pub(crate) peer_id: PeerId,
    pub(crate) request: MilestoneRequest,
}

pub(crate) struct MilestoneResponderWorker {
    pub(crate) tx: flume::Sender<MilestoneResponderWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for MilestoneResponderWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let network = node.resource::<Network>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            while let Some(MilestoneResponderWorkerEvent { peer_id, request }) = receiver.next().await {
                let index = match request.index {
                    0 => tangle.get_latest_milestone_index(),
                    _ => request.index.into(),
                };

                if let Some(message_id) = tangle.get_milestone_message_id(index) {
                    if let Some(message) = tangle.get(&MessageId::from(message_id)).await {
                        let mut bytes = Vec::new();

                        if message.pack(&mut bytes).is_ok() {
                            Sender::<MessagePacket>::send(&network, &peer_id, MessagePacket::new(&bytes));
                        }
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
