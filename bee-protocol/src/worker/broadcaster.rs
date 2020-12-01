// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    packet::{tlv_into_bytes, Message},
    protocol::Protocol,
};

use bee_common::{node::Node, shutdown_stream::ShutdownStream, worker::Worker};
use bee_network::{Command::SendMessage, Network, PeerId};

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{info, warn};

use std::convert::Infallible;

pub(crate) struct BroadcasterWorkerEvent {
    pub(crate) source: Option<PeerId>,
    pub(crate) message: Message,
}

pub(crate) struct BroadcasterWorker {
    pub(crate) tx: flume::Sender<BroadcasterWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for BroadcasterWorker {
    type Config = ();
    type Error = Infallible;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        let network = node.resource::<Network>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            while let Some(BroadcasterWorkerEvent { source, message }) = receiver.next().await {
                let bytes = tlv_into_bytes(message);

                for peer in Protocol::get().peer_manager.peers.iter() {
                    if match source {
                        Some(ref source) => source != peer.key(),
                        None => true,
                    } {
                        match network.unbounded_send(SendMessage {
                            message: bytes.clone(),
                            to: peer.key().clone(),
                        }) {
                            Ok(_) => {
                                (*peer.value()).metrics.messages_sent_inc();
                                Protocol::get().metrics.messages_sent_inc();
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
