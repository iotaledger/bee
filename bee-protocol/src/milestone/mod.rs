// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod index;
pub(crate) mod key_manager;
pub(crate) mod key_range;

pub use index::MilestoneIndex;

use bee_common::packable::{Packable, Read, Write};
use bee_message::MessageId;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error {0}")]
    IO(#[from] std::io::Error),
    #[error("MessageId error {0}")]
    MessageId(<MessageId as Packable>::Error),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub(crate) message_id: MessageId,
    pub(crate) timestamp: u64,
}

impl Milestone {
    pub fn new(message_id: MessageId, timestamp: u64) -> Self {
        Self {
            message_id,
            timestamp,
        }
    }

    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl Packable for Milestone {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.message_id.packed_len() + self.timestamp.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.message_id.pack(writer).map_err(Error::MessageId)?;
        self.timestamp.pack(writer).map_err(Error::IO)
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            message_id: MessageId::unpack(reader).map_err(Error::MessageId)?,
            timestamp: u64::unpack(reader)?,
        })
    }
}
