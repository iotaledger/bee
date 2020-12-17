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
    pub(crate) index: MilestoneIndex,
    pub(crate) message_id: MessageId,
}

impl Milestone {
    pub fn new(index: MilestoneIndex, message_id: MessageId) -> Self {
        Self { index, message_id }
    }

    pub fn index(&self) -> MilestoneIndex {
        self.index
    }

    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }
}

impl Packable for Milestone {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.index.packed_len() + self.message_id.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.index.pack(writer)?;
        self.message_id.pack(writer).map_err(Error::MessageId)
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            index: MilestoneIndex::unpack(reader)?,
            message_id: MessageId::unpack(reader).map_err(Error::MessageId)?,
        })
    }
}
