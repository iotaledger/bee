// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    alias,
    service::event::{InternalEvent, InternalEventSender},
};

use bee_runtime::task::{StandaloneSpawner, TaskSpawner};

use futures::{
    io::{BufReader, BufWriter, ReadHalf, WriteHalf},
    AsyncReadExt, AsyncWriteExt, StreamExt,
};
use libp2p::{swarm::NegotiatedSubstream, PeerId};
use log::*;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

const MSG_BUFFER_LEN: usize = 32768;

/// A type alias for an unbounded channel sender.
pub type GossipSender = mpsc::UnboundedSender<Vec<u8>>;

/// A type alias for an unbounded channel receiver.
pub type GossipReceiver = UnboundedReceiverStream<Vec<u8>>;

pub fn channel() -> (GossipSender, GossipReceiver) {
    let (sender, receiver) = mpsc::unbounded_channel();
    (sender, UnboundedReceiverStream::new(receiver))
}

pub fn start_incoming_processor(
    peer_id: PeerId,
    mut reader: BufReader<ReadHalf<Box<NegotiatedSubstream>>>,
    incoming_tx: GossipSender,
    internal_event_sender: InternalEventSender,
) {
    StandaloneSpawner::spawn(async move {
        let mut msg_buf = vec![0u8; MSG_BUFFER_LEN];

        loop {
            if let Some(len) = (&mut reader).read(&mut msg_buf).await.ok().filter(|len| *len > 0) {
                if incoming_tx.send(msg_buf[..len].to_vec()).is_err() {
                    debug!("gossip-in: receiver dropped locally.");

                    // The receiver of this channel was dropped, maybe due to a shutdown. There is nothing we can do
                    // to salvage this situation, hence we drop the connection.
                    break;
                }
            } else {
                debug!("gossip-in: stream closed remotely.");

                // NB: The network service will not shut down before it has received the `ProtocolDropped` event
                // from all once connected peers, hence if the following send fails, then it
                // must be considered a bug.

                // The remote peer dropped the connection.
                internal_event_sender
                    .send(InternalEvent::ProtocolDropped { peer_id })
                    .expect("The service must not shutdown as long as there are gossip tasks running.");

                break;
            }
        }

        // Reasons why this task might end:
        // (1) The remote dropped the TCP connection.
        // (2) The local dropped the gossip_in receiver channel.

        debug!("gossip-in: exiting gossip-in processor for {}.", alias!(peer_id));
    });
}

pub fn start_outgoing_processor(
    peer_id: PeerId,
    mut writer: BufWriter<WriteHalf<Box<NegotiatedSubstream>>>,
    outgoing_rx: GossipReceiver,
    internal_event_sender: InternalEventSender,
) {
    StandaloneSpawner::spawn(async move {
        let mut outgoing_gossip_receiver = outgoing_rx.fuse();

        // If the gossip sender dropped we end the connection.
        while let Some(message) = outgoing_gossip_receiver.next().await {
            // NB: Instead of polling another shutdown channel, we use an empty message
            // to signal that we want to end the connection. We use this "trick" whenever the network
            // receives the `DisconnectPeer` command to enforce that the connection will be dropped.

            if message.is_empty() {
                debug!("gossip-out: received shutdown message.");

                // NB: The network service will not shut down before it has received the `ConnectionDropped` event
                // from all once connected peers, hence if the following send fails, then it
                // must be considered a bug.

                internal_event_sender
                    .send(InternalEvent::ProtocolDropped { peer_id })
                    .expect("The service must not shutdown as long as there are gossip tasks running.");

                break;
            }

            // If sending to the stream fails we end the connection.
            // TODO: buffer for x milliseconds before flushing.
            if (&mut writer).write_all(&message).await.is_err() || (&mut writer).flush().await.is_err() {
                debug!("gossip-out: stream closed remotely");
                break;
            }
        }

        // Reasons why this task might end:
        // (1) The local send the shutdown message (len = 0)
        // (2) The remote dropped the TCP connection.

        debug!("gossip-out: exiting gossip-out processor for {}.", alias!(peer_id));
    });
}
