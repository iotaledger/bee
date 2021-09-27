// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    message::MessageType,
    peer::{Error, GossipReader, GossipWriter, PeerInfo},
};

use backstage::core::{AbortableUnboundedChannel, Actor, ActorResult, NullChannel, Rt, StreamExt, SupHandle};

use std::sync::Arc;

/// A peer reader actor.
pub struct GossipReaderActor {
    reader: GossipReader,
    info: Arc<PeerInfo>,
}

impl GossipReaderActor {
    pub(crate) fn new(reader: GossipReader, info: Arc<PeerInfo>) -> Self {
        Self { reader, info }
    }
}

#[async_trait::async_trait]
impl<S: SupHandle<Self>> Actor<S> for GossipReaderActor {
    type Data = ();
    type Channel = NullChannel;

    async fn init(&mut self, _rt: &mut Rt<Self, S>) -> ActorResult<Self::Data> {
        Ok(())
    }

    async fn run(&mut self, _rt: &mut Rt<Self, S>, _data: Self::Data) -> ActorResult<()> {
        loop {
            match self.reader.recv_msgs(&self.info).await {
                Ok(msgs) => {
                    if msgs.is_empty() {
                        log::debug!("peer did not sent any messages");
                        continue;
                    }
                    log::debug!("received {} messages from {}", msgs.len(), self.info.id());
                }
                Err(e) => {
                    log::warn!("receive error: {:?}", e);
                    match e {
                        Error::UnknownMessageType => continue,
                        _ => break,
                    }
                }
            }
        }

        Ok(())
    }
}

/// A peer writer actor event.
pub enum GossipWriterEvent {
    /// Send a message to the peer.
    SendMessage {
        /// The bytes of the message.
        bytes: Vec<u8>,
        /// The type of message.
        msg_type: MessageType,
    },
}

/// A peer writer actor.
pub struct GossipWriterActor {
    writer: GossipWriter,
    info: Arc<PeerInfo>,
}

impl GossipWriterActor {
    pub(crate) fn new(writer: GossipWriter, info: Arc<PeerInfo>) -> Self {
        Self { writer, info }
    }
}

#[async_trait::async_trait]
impl<S: SupHandle<Self>> Actor<S> for GossipWriterActor {
    type Data = ();
    type Channel = AbortableUnboundedChannel<GossipWriterEvent>;

    async fn init(&mut self, _rt: &mut Rt<Self, S>) -> ActorResult<Self::Data> {
        Ok(())
    }

    async fn run(&mut self, rt: &mut Rt<Self, S>, _data: Self::Data) -> ActorResult<()> {
        while let Some(event) = rt.inbox_mut().next().await {
            match event {
                GossipWriterEvent::SendMessage { bytes, msg_type } => {
                    if let Err(err) = self.writer.send_msg(&bytes, msg_type, &self.info).await {
                        log::warn!("could not send message: {:?}", err);
                    }
                }
            }
        }

        Ok(())
    }
}
