// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod client;
mod errors;
mod server;

pub use client::*;
pub use errors::Error;
pub use server::*;

use crate::{
    interaction::events::{InternalEvent, InternalEventSender},
    peers::{self, MessageReceiver, MessageSender, PeerInfo},
    protocols::gossip::{GossipProtocol, GossipSubstream},
    PeerId, ShortId,
};

use futures::{
    io::{ReadHalf, WriteHalf},
    prelude::*,
    AsyncRead, AsyncWrite,
};
use libp2p::core::{
    muxing::{event_from_ref_and_wrap, outbound_from_ref_and_wrap, StreamMuxerBox},
    upgrade,
};
use log::*;

use std::{fmt, sync::Arc};

pub(crate) async fn upgrade_connection(
    peer_id: PeerId,
    peer_info: PeerInfo,
    muxer: StreamMuxerBox,
    origin: Origin,
    internal_event_sender: InternalEventSender,
) -> Result<(), Error> {
    let muxer = Arc::new(muxer);

    let substream = match origin {
        Origin::Outbound => {
            let outbound = outbound_from_ref_and_wrap(muxer)
                // .fuse()
                .await
                .map_err(|_| Error::CreatingOutboundSubstreamFailed(peer_id.short()))?;

            upgrade::apply_outbound(outbound, GossipProtocol, upgrade::Version::V1Lazy)
                .await
                .map_err(|_| Error::SubstreamProtocolUpgradeFailed(peer_id.short()))?
        }
        Origin::Inbound => {
            let inbound = loop {
                if let Some(inbound) = event_from_ref_and_wrap(muxer.clone())
                    .await
                    .map_err(|_| Error::CreatingInboundSubstreamFailed(peer_id.short()))?
                    .into_inbound_substream()
                {
                    break inbound;
                }
            };

            upgrade::apply_inbound(inbound, GossipProtocol)
                .await
                .map_err(|_| Error::SubstreamProtocolUpgradeFailed(peer_id.short()))?
        }
    };

    let (reader, writer) = substream.split();

    let (incoming_gossip_sender, incoming_gossip_receiver) = peers::channel();
    let (outgoing_gossip_sender, outgoing_gossip_receiver) = peers::channel();

    spawn_gossip_in_task(
        peer_id.clone(),
        reader,
        incoming_gossip_sender,
        internal_event_sender.clone(),
    );
    spawn_gossip_out_task(
        peer_id.clone(),
        writer,
        outgoing_gossip_receiver,
        internal_event_sender.clone(),
    );

    internal_event_sender
        .send(InternalEvent::ConnectionEstablished {
            peer_id,
            peer_info,
            origin,
            gossip_in: incoming_gossip_receiver,
            gossip_out: outgoing_gossip_sender,
        })
        .map_err(|_| Error::InternalEventSendFailure("ConnectionEstablished"))?;

    Ok(())
}

fn spawn_gossip_in_task(
    peer_id: PeerId,
    mut reader: ReadHalf<GossipSubstream>,
    incoming_gossip_sender: MessageSender,
    internal_event_sender: InternalEventSender,
) {
    tokio::spawn(async move {
        const MSG_BUFFER_SIZE: usize = 32768;
        let mut buffer = vec![0u8; MSG_BUFFER_SIZE];

        loop {
            if let Ok(num_read) = recv_message(&mut reader, &mut buffer, &peer_id).await {
                if incoming_gossip_sender.send(buffer[..num_read].to_vec()).is_err() {
                    // Any reason sending to this channel fails is unrecoverable (OOM or receiver dropped),
                    // hence, we will silently just end this task.
                    break;
                }
            } else {
                // NOTE: we silently ignore, if that event can't be send as this usually means, that the node shut down
                let _ = internal_event_sender.send(InternalEvent::ConnectionDropped { peer_id });

                // Connection with peer stopped due to reasons outside of our control.
                break;
            }
        }

        trace!("Exiting gossip-in processor for {}", peer_id.short());
    });
}

fn spawn_gossip_out_task(
    peer_id: PeerId,
    mut writer: WriteHalf<GossipSubstream>,
    outgoing_gossip_receiver: MessageReceiver,
    internal_event_sender: InternalEventSender,
) {
    tokio::spawn(async move {
        let mut outgoing_gossip_receiver = outgoing_gossip_receiver.fuse();

        loop {
            if let Some(message) = outgoing_gossip_receiver.next().await {
                if send_message(&mut writer, &message).await.is_err() {
                    // Any reason sending to the stream fails is considered unrecoverable, hence,
                    // we will end this task.
                    break;
                }
            } else {
                // NOTE: we silently ignore, if that event can't be send as this usually means, that the node shut down
                let _ = internal_event_sender.send(InternalEvent::ConnectionDropped { peer_id });

                break;
            }
        }

        trace!("Exiting gossip-out processor for {}", peer_id.short());
    });
}

async fn send_message<S>(stream: &mut S, message: &[u8]) -> Result<(), Error>
where
    S: AsyncWrite + Unpin,
{
    stream.write_all(message).await.map_err(|_| Error::MessageSendError)?;
    stream.flush().await.map_err(|_| Error::MessageSendError)?;

    // trace!("Wrote {} bytes to stream.", message.len());
    Ok(())
}

async fn recv_message<S>(stream: &mut S, message: &mut [u8], peer_id: &PeerId) -> Result<usize, Error>
where
    S: AsyncRead + Unpin,
{
    let num_read = stream.read(message).await.map_err(|_| Error::MessageRecvError)?;
    if num_read == 0 {
        // EOF
        trace!("Stream was closed remotely (EOF).");
        return Err(Error::StreamClosedByRemote(peer_id.short()));
    }

    // trace!("Read {} bytes from stream.", num_read);
    Ok(num_read)
}

/// Describes direction of an established connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Origin {
    /// The connection is inbound (server).
    Inbound,
    /// The connection is outbound (client).
    Outbound,
}

impl Origin {
    /// Returns whether the connection is inbound.
    pub fn is_inbound(&self) -> bool {
        *self == Origin::Inbound
    }

    /// Returns whether the connection is outbound.
    pub fn is_outbound(&self) -> bool {
        *self == Origin::Outbound
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            Origin::Outbound => "outbound",
            Origin::Inbound => "inbound",
        };
        write!(f, "{}", s)
    }
}
