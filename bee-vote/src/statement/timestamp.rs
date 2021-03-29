// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    entry::EntryType,
    opinion::{Opinion, OPINION_STATEMENT_LENGTH},
};
use crate::error::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::{MessageId, MESSAGE_ID_LENGTH};

/// Holds a message ID and its timestamp opinion.
#[derive(Debug)]
pub struct Timestamp {
    /// Message ID.
    pub id: MessageId,
    /// Opinion of the message timestamp.
    pub opinion: Opinion,
}

impl EntryType for Timestamp {
    type Id = MessageId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn opinion(&self) -> &Opinion {
        &self.opinion
    }
}

impl Packable for Timestamp {
    type Error = Error;

    fn packed_len(&self) -> usize {
        MESSAGE_ID_LENGTH + OPINION_STATEMENT_LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.id.pack(writer)?;
        self.opinion.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let message_id = MessageId::unpack(reader)?;
        let opinion = Opinion::unpack(reader)?;

        Ok(Self {
            id: message_id,
            opinion,
        })
    }
}
