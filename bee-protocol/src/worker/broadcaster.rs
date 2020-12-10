// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    packet::{tlv_into_bytes, Message},
    protocol::{Protocol, ProtocolMetrics},
    worker::MetricsWorker,
};

use bee_common::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_network::{Command::SendMessage, NetworkController, PeerId};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{info, warn};
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
        vec![TypeId::of::<MetricsWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = mpsc::unbounded_channel();

        let network = node.resource::<NetworkController>();
        let metrics = node.resource::<ProtocolMetrics>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx);

            while let Some(BroadcasterWorkerEvent { source, message }) = receiver.next().await {
                let bytes = tlv_into_bytes(message);

                for peer in Protocol::get().peer_manager.peers.iter() {
                    if match source {
                        Some(ref source) => source != peer.key(),
                        None => true,
                    } {
                        match network.send(SendMessage {
                            message: bytes.clone(),
                            to: peer.key().clone(),
                        }) {
                            Ok(_) => {
                                (*peer.value()).metrics.messages_sent_inc();
                                metrics.messages_sent_inc();
                            }
                            Err(e) => {
                                warn!("Broadcasting message to {:?} failed: {:?}.", *peer.key(), e);
                            }
                        };
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
