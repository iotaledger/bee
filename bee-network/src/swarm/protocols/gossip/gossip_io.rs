// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::service::{InternalEvent, InternalEventSender};

use futures::{
    io::{ReadHalf, WriteHalf},
    AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, StreamExt,
};
use libp2p::{swarm::NegotiatedSubstream, PeerId};
use log::*;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

const MSG_BUFFER_SIZE: usize = 32768;

/// A shorthand for an unbounded channel sender.
pub type GossipSender = mpsc::UnboundedSender<Vec<u8>>;

/// A shorthand for an unbounded channel receiver.
pub type GossipReceiver = UnboundedReceiverStream<Vec<u8>>;

pub fn gossip_channel() -> (GossipSender, GossipReceiver) {
    let (sender, receiver) = mpsc::unbounded_channel();
    (sender, UnboundedReceiverStream::new(receiver))
}

pub fn spawn_gossip_in_task(
    peer_id: PeerId,
    mut reader: ReadHalf<NegotiatedSubstream>,
    incoming_gossip_sender: GossipSender,
    internal_event_sender: InternalEventSender,
) {
    tokio::spawn(async move {
        let mut msg_buf = vec![0u8; MSG_BUFFER_SIZE];
        let mut msg_len = 0;

        loop {
            if recv_valid_message(&mut reader, &mut msg_buf, &mut msg_len).await {
                if incoming_gossip_sender.send(msg_buf[..msg_len].to_vec()).is_err() {
                    // The receiver of this channel was dropped, maybe due to a shutdown. There is nothing we can do to
                    // salvage this situation, hence we drop the connection.
                    break;
                }
            } else {
                trace!("Gossip-IO: Stream closed remotely.");

                // NB: The network service will not shut down before it has received the `ProtocolDropped` event from
                // all once connected peers, hence if the following send fails, then it must be
                // considered a bug.

                // The remote peer dropped the connection.
                internal_event_sender
                    .send(InternalEvent::ProtocolDropped { peer_id })
                    .expect("The service must not shutdown as long as there are gossip tasks running.");

                break;
            }
        }

        trace!("Exiting incoming gossip event loop for {}", peer_id);
    });
}

async fn recv_valid_message<S>(stream: &mut S, message: &mut [u8], message_len: &mut usize) -> bool
where
    S: AsyncRead + Unpin,
{
    if let Ok(msg_len) = stream.read(message).await {
        if msg_len == 0 {
            false
        } else {
            *message_len = msg_len;
            true
        }
    } else {
        false
    }
}

pub fn spawn_gossip_out_task(
    peer_id: PeerId,
    mut writer: WriteHalf<NegotiatedSubstream>,
    outgoing_gossip_receiver: GossipReceiver,
    internal_event_sender: InternalEventSender,
) {
    tokio::spawn(async move {
        let mut outgoing_gossip_receiver = outgoing_gossip_receiver.fuse();

        // If the gossip sender dropped we end the connection.
        while let Some(message) = outgoing_gossip_receiver.next().await {
            let msg_len = message.len();

            // NB: Instead of polling another shutdown channel, we analogously use an empty message
            // to signal that we want to end the connection. We use this "trick" whenever the network
            // receives the `DisconnectPeer` command to enforce that the connection will be dropped.

            if msg_len == 0 {
                trace!("Gossip-IO: Stream closing locally.");

                // NB: The network service will not shut down before it has received the `ConnectionDropped` event from
                // all once connected peers, hence if the following send fails, then it must be
                // considered a bug.

                internal_event_sender
                    .send(InternalEvent::ProtocolDropped { peer_id })
                    .expect("The service must not shutdown as long as there are gossip tasks running.");

                break;
            }

            // If sending to the stream fails we end the connection.
            if !send_valid_message(&mut writer, &message).await {
                break;
            }
        }

        trace!("Exiting outgoing gossip event loop for {}", peer_id);
    });
}

async fn send_valid_message<S>(stream: &mut S, message: &[u8]) -> bool
where
    S: AsyncWrite + Unpin,
{
    if stream.write_all(message).await.is_err() {
        return false;
    }

    if stream.flush().await.is_err() {
        return false;
    }

    true
}
