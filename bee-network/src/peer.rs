// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that deals with peers.

use crate::{
    identity::Identity,
    message::{Message, MessageRequest, MessageType},
    consts::MAX_PACKET_SIZE,
};

use prost::bytes::{Buf, BufMut, BytesMut};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

use std::sync::atomic::{AtomicBool, Ordering};

const BUFFER_SIZE: usize = std::mem::size_of::<u32>() + MAX_PACKET_SIZE;

pub(crate) struct PeerInfo {
    identity: Identity,
    alias: String,
    healthy: AtomicBool,
}

impl PeerInfo {
    /// Creates a new connected peer.
    pub(crate) fn new(identity: Identity, alias: String) -> Self {
        Self {
            identity,
            alias,
            healthy: true.into(),
        }
    }

    /// The ID of the peer.
    pub(crate) fn id(&self) -> String {
        self.identity.id_string()
    }

    /// The alias of the peer.
    pub(crate) fn alias(&self) -> &String {
        &self.alias
    }

    /// Whether the peer, and therefore the connection is still healthy.
    pub(crate) fn healthy(&self) -> bool {
        self.healthy.load(Ordering::Relaxed)
    }
}

pub(crate) struct PeerReader {
    reader: BufReader<OwnedReadHalf>,
    // FIXME: do we need to preallocate 64Kb for every peer?
    buffer: Box<[u8; BUFFER_SIZE]>,
}

impl PeerReader {
    pub(crate) fn new(reader: BufReader<OwnedReadHalf>) -> Self {
        Self {
            reader,
            buffer: Box::new([0; BUFFER_SIZE]),
        }
    }

    pub async fn recv_msgs(&mut self, info: &PeerInfo) -> Result<Vec<(MessageType, Vec<u8>)>, Error> {
        if info.healthy() {
            // NOTE:
            // - every message is prepended by its length: see iotaledger/hive.go/netutil/buffconn/buffconn.go
            // - Bytes 0..3 encode a u32 representing the message length
            // - Byte 4     encodes the message type (Message or MessageRequest)
            // - Bytes 5..n encode the protobuf representation of the actual message

            let num_received = self
                .reader
                .read(self.buffer.as_mut())
                .await
                .map_err(Error::RecvMessage)?;

            if num_received == 0 {
                log::warn!("connection reset by peer");
                info.healthy.store(false, Ordering::Relaxed);

                Err(Error::NotHealthy)
            } else {
                log::debug!("received {} bytes", num_received);

                let mut position = 0;
                let mut messages = Vec::with_capacity(16);

                while position < num_received {
                    // Determine the length of the next message within this batch
                    let mut msg_len_buf = [0u8; 4];
                    msg_len_buf.copy_from_slice(&self.buffer[position..position + 4]);
                    let msg_len = Buf::get_u32(&mut &msg_len_buf[..]) as usize - 1;
                    // println!("Message length (excl. type specifier byte): {}.", msg_len);

                    // Determine the message type
                    let msg_type: MessageType =
                        num::FromPrimitive::from_u8(self.buffer[position + 4]).ok_or(Error::UnknownMessageType)?;
                    // println!("Message type: {:?}.", msg_type);

                    match msg_type {
                        MessageType::Message => {
                            let msg = Message::from_protobuf(&self.buffer[position + 5..position + 5 + msg_len])
                                .map_err(Error::Decode)?;
                            // println!("{:#?}", msg);

                            let data = msg.into_bytes();

                            messages.push((MessageType::Message, data));
                        }
                        MessageType::MessageRequest => {
                            let msg_req =
                                MessageRequest::from_protobuf(&self.buffer[position + 5..position + 5 + msg_len])
                                    .map_err(Error::Decode)?;
                            // println!("{:#?}", msg_req);

                            let id = msg_req.into_bytes();

                            messages.push((MessageType::MessageRequest, id))
                        }
                    }

                    position += 5 + msg_len;
                }

                Ok(messages)
            }
        } else {
            Err(Error::NotHealthy)
        }
    }
}

pub(crate) struct PeerWriter {
    writer: BufWriter<OwnedWriteHalf>,
}

impl PeerWriter {
    pub(crate) fn new(writer: BufWriter<OwnedWriteHalf>) -> Self {
        Self { writer }
    }

    async fn write_buf(&mut self, buf: &mut &[u8], info: &PeerInfo) -> Result<(), Error> {
        if let Err(e) = self.writer.write_all(buf).await {
            info.healthy.store(false, Ordering::Relaxed);
            return Err(Error::SendMessage(e));
        }

        if let Err(e) = self.writer.flush().await {
            info.healthy.store(false, Ordering::Relaxed);
            return Err(Error::SendMessage(e));
        }

        Ok(())
    }

    pub(crate) async fn send_msg(&mut self, msg: &[u8], msg_type: MessageType, info: &PeerInfo) -> Result<(), Error> {
        if !info.healthy() {
            return Err(Error::NotHealthy);
        }

        let msg = match msg_type {
            MessageType::Message => Message::new(msg).protobuf().map_err(Error::Encode)?,
            MessageType::MessageRequest => MessageRequest::new(msg).protobuf().map_err(Error::Encode)?,
        };

        // We need to prepend the message with:
        // - the length of the message + 1 (for the message type byte)
        // - the message type byte itself

        let msg_len = msg.len() + 1;

        let mut msg_buf = BytesMut::with_capacity(std::mem::size_of::<u32>() + msg_len);

        msg_buf.put_u32(msg_len as u32);
        msg_buf.put_u8(msg_type as u8);
        msg_buf.put(&msg[..]);

        self.write_buf(&mut msg_buf.as_ref(), info).await?;

        Ok(())
    }

    pub async fn send_msgs(&mut self, msgs: &[(&[u8], MessageType)], info: &PeerInfo) -> Result<(), Error> {
        if !info.healthy() {
            return Err(Error::NotHealthy);
        }

        let mut pkt_buf = BytesMut::with_capacity(MAX_PACKET_SIZE);

        for (msg, msg_type) in msgs {
            let msg = match msg_type {
                MessageType::Message => Message::new(msg).protobuf().map_err(Error::Encode)?,
                MessageType::MessageRequest => MessageRequest::new(msg).protobuf().map_err(Error::Encode)?,
            };

            let msg_len = msg.len() + 1;

            // If we reached MAX_PACKET_SIZE we write it to the socket, and continue with a new packet.
            if pkt_buf.len() >= MAX_PACKET_SIZE {
                self.write_buf(&mut pkt_buf.as_ref(), info).await?;
                pkt_buf.clear();
            }

            pkt_buf.put_u32(msg_len as u32);
            pkt_buf.put_u8(*msg_type as u8);
            pkt_buf.put(&msg[..]);
        }

        self.write_buf(&mut pkt_buf.as_ref(), info).await?;

        Ok(())
    }
}

/// [`ConnectedPeer`] related errors.
#[derive(Debug)]
pub enum Error {
    /// A once connected peer turned unhealthy, probably due to a dropped connection. It cannot be used any longer to
    /// send or recv any mesages.
    NotHealthy,
    /// An error that may occur when trying to send a message.
    SendMessage(io::Error),
    /// An error that may occur when trying to receive a message.
    RecvMessage(io::Error),
    /// An error that may occur when trying to decode a message.
    Decode(prost::DecodeError),
    /// An error that may occur when trying to encode a message.
    Encode(prost::EncodeError),
    /// An error that may occur when dealing with a particular packet from the wire.
    PacketType(io::Error),
    /// An error that may occur when dealing with a particular message from the wire.
    MessageType(io::Error),
    /// An error that occurs when an unknown message type was received.
    UnknownMessageType,
}

/// Represents a fully connected (i.e. handshaked) peer.
pub struct ConnectedPeer {
    pub(crate) info: PeerInfo,
    pub(crate) reader: PeerReader,
    pub(crate) writer: PeerWriter,
}

impl ConnectedPeer {
    /// Creates a new connected peer.
    pub fn new(
        identity: Identity,
        alias: String,
        reader: BufReader<OwnedReadHalf>,
        writer: BufWriter<OwnedWriteHalf>,
    ) -> Self {
        Self {
            info: PeerInfo::new(identity, alias),
            reader: PeerReader::new(reader),
            writer: PeerWriter::new(writer),
        }
    }

    /// The ID of the peer.
    pub fn id(&self) -> String {
        self.info.id()
    }

    /// The alias of the peer.
    pub fn alias(&self) -> &str {
        self.info.alias()
    }

    /// Whether the peer, and therefore the connection is still healthy.
    pub fn healthy(&self) -> bool {
        self.info.healthy()
    }

    // TODO: timed flush
    /// Sends a single message to this peer.
    pub async fn send_msg(&mut self, msg: &[u8], msg_type: MessageType) -> Result<(), Error> {
        self.writer.send_msg(msg, msg_type, &self.info).await
    }

    /// Sends multiple messages to this peer (as a single packet).
    pub async fn send_msgs(&mut self, msgs: &[(&[u8], MessageType)]) -> Result<(), Error> {
        self.writer.send_msgs(msgs, &self.info).await
    }

    /// Tries to receive one or more messages from this peer.
    pub async fn recv_msgs(&mut self) -> Result<Vec<(MessageType, Vec<u8>)>, Error> {
        self.reader.recv_msgs(&self.info).await
    }
}
