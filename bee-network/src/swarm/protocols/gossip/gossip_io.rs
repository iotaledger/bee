use crate::service::events::{InternalEvent, InternalEventSender};

use futures::{
    io::{ReadHalf, WriteHalf},
    AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, StreamExt,
};
use libp2p::{swarm::NegotiatedSubstream, PeerId};
use log::*;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use super::errors::Error;

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
    internal_sender: InternalEventSender,
) {
    tokio::spawn(async move {
        let mut buffer = vec![0u8; MSG_BUFFER_SIZE];

        loop {
            if let Ok(num_read) = recv_message(&mut reader, &mut buffer, &peer_id).await {
                debug_assert!(num_read > 0);

                if incoming_gossip_sender.send(buffer[..num_read].to_vec()).is_err() {
                    // Any reason sending to this channel fails is unrecoverable (OOM or receiver dropped),
                    // hence, we will silently just end this task.
                    break;
                }
            } else {
                // NOTE: we silently ignore, if that event can't be send as this usually means, that the node shut down
                if internal_sender
                    .send(InternalEvent::ConnectionDropped { peer_id })
                    .is_err()
                {
                    trace!("Receiver of internal event channel already dropped.");
                }

                // Connection with peer stopped due to reasons outside of our control.
                break;
            }
        }

        trace!("Exiting gossip-in processor for {}", peer_id);
    });
}

pub fn spawn_gossip_out_task(
    peer_id: PeerId,
    mut writer: WriteHalf<NegotiatedSubstream>,
    outgoing_gossip_receiver: GossipReceiver,
    internal_sender: InternalEventSender,
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
                if internal_sender
                    .send(InternalEvent::ConnectionDropped { peer_id })
                    .is_err()
                {
                    trace!("Receiver of internal event channel already dropped.");
                }

                break;
            }
        }

        trace!("Exiting gossip-out processor for {}", peer_id);
    });
}

async fn send_message<S>(stream: &mut S, message: &[u8]) -> Result<(), Error>
where
    S: AsyncWrite + Unpin,
{
    stream.write_all(message).await.map_err(|_| Error::MessageSendError)?;

    stream.flush().await.map_err(|_| Error::MessageSendError)?;

    Ok(())
}

async fn recv_message<S>(stream: &mut S, message: &mut [u8], peer_id: &PeerId) -> Result<usize, Error>
where
    S: AsyncRead + Unpin,
{
    let num_read = stream.read(message).await.map_err(|_| Error::MessageRecvError)?;

    if num_read == 0 {
        trace!("Stream was closed remotely (EOF).");
        return Err(Error::StreamClosedByRemote(*peer_id));
    }

    Ok(num_read)
}
