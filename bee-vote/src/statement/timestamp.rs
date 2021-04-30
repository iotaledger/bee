// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Timestamp statement.

use super::{
    entry::EntryType,
    opinion::{OpinionStatement, OPINION_STATEMENT_LENGTH},
};
use crate::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::prelude::{MessageId, MESSAGE_ID_LENGTH};

/// Holds a message ID and its timestamp opinion.
#[derive(Debug, PartialEq, Eq)]
pub struct Timestamp {
    /// Message ID.
    pub id: MessageId,
    /// Opinion of the message timestamp.
    pub opinion: OpinionStatement,
}

impl EntryType for Timestamp {
    type Id = MessageId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn opinion(&self) -> &OpinionStatement {
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

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let message_id = MessageId::unpack_inner::<R, CHECK>(reader)?;
        let opinion = OpinionStatement::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self {
            id: message_id,
            opinion
        })
    }
}
