// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use futures::{
    io::{BufReader, BufWriter, ReadHalf, WriteHalf},
    AsyncReadExt, AsyncWriteExt, StreamExt,
};
use libp2p::{swarm::NegotiatedSubstream, PeerId};
use log::*;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    alias,
    service::event::{InternalEvent, InternalEventSender},
};

const MSG_BUFFER_LEN: usize = 32768;

/// A type alias for an unbounded channel sender.
pub type GossipSender = mpsc::UnboundedSender<Vec<u8>>;

/// A type alias for an unbounded channel receiver.
pub type GossipReceiver = UnboundedReceiverStream<Vec<u8>>;

pub fn channel() -> (GossipSender, GossipReceiver) {
    let (sender, receiver) = mpsc::unbounded_channel();
    (sender, UnboundedReceiverStream::new(receiver))
}

pub fn start_inbound_gossip_handler(
    peer_id: PeerId,
    mut inbound_gossip_rx: BufReader<ReadHalf<Box<NegotiatedSubstream>>>,
    inbound_gossip_tx: GossipSender,
    internal_event_tx: InternalEventSender,
) {
    tokio::spawn(async move {
        let mut buf = vec![0u8; MSG_BUFFER_LEN];

        loop {
            if let Some(len) = inbound_gossip_rx.read(&mut buf).await.ok().filter(|len| *len > 0) {
                if inbound_gossip_tx.send(buf[..len].to_vec()).is_err() {
                    debug!("Terminating gossip protocol with {}.", alias!(peer_id));

                    break;
                }
            } else {
                debug!("Peer {} terminated gossip protocol.", alias!(peer_id));

                // Panic: we made sure that the sender (network host) is always dropped before the receiver (service
                // host) through the worker dependencies, hence this can never panic.
                internal_event_tx
                    .send(InternalEvent::ProtocolStopped { peer_id })
                    .expect("send internal event");

                break;
            }
        }

        trace!("Dropping gossip stream reader for {}.", alias!(peer_id));
    });
}

pub fn start_outbound_gossip_handler(
    peer_id: PeerId,
    mut outbound_gossip_tx: BufWriter<WriteHalf<Box<NegotiatedSubstream>>>,
    outbound_gossip_rx: GossipReceiver,
    internal_event_tx: InternalEventSender,
) {
    tokio::spawn(async move {
        let mut outbound_gossip_rx = outbound_gossip_rx.fuse();

        // If the gossip sender dropped we end the connection.
        while let Some(message) = outbound_gossip_rx.next().await {
            // Note: Instead of polling another shutdown channel, we use an empty message
            // to signal that we want to end the connection. We use this "trick" whenever the network
            // receives the `DisconnectPeer` command to enforce that the connection will be dropped.
            if message.is_empty() {
                debug!(
                    "Terminating gossip protocol with {} (received shutdown signal).",
                    alias!(peer_id)
                );

                // Panic: we made sure that the sender (network host) is always dropped before the receiver (service
                // host) through the worker dependencies, hence this can never panic.
                internal_event_tx
                    .send(InternalEvent::ProtocolStopped { peer_id })
                    .expect("send internal event");

                break;
            } else if outbound_gossip_tx.write_all(&message).await.is_err() || outbound_gossip_tx.flush().await.is_err()
            {
                debug!("Peer {} terminated gossip protocol.", alias!(peer_id));

                break;
            }
        }

        trace!("Dropping gossip stream writer for {}.", alias!(peer_id));
    });
}
