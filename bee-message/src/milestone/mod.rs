// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

mod index;

pub use index::MilestoneIndex;

use crate::MessageId;

use bee_common::packable::{Packable, Read, Write};

/// Defines a coordinator milestone.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    message_id: MessageId,
    timestamp: u64,
}

impl Milestone {
    /// Creates a new `Milestone`.
    pub fn new(message_id: MessageId, timestamp: u64) -> Self {
        Self { message_id, timestamp }
    }

    /// Returns the message id of a `Milestone`.
    pub fn message_id(&self) -> &MessageId {
        &self.message_id
    }

    /// Returns the timestamp of a `Milestone`.
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
        self.message_id.pack(writer)?;
        self.timestamp.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let message_id = MessageId::unpack_inner::<R, CHECK>(reader)?;
        let timestamp = u64::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self::new(message_id, timestamp))
    }
}
