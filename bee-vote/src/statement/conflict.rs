// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Conlict statement.

use super::{
    entry::EntryType,
    opinion::{Opinion, OPINION_STATEMENT_LENGTH},
};
use crate::error::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::payload::transaction::{TransactionId, TRANSACTION_ID_LENGTH};

/// Holds a conflicting transaction ID and its opinion.
#[derive(Debug)]
pub struct Conflict {
    /// Conflicting transaction ID.
    pub id: TransactionId,
    /// Opinion of the conflict.
    pub opinion: Opinion,
}

impl EntryType for Conflict {
    type Id = TransactionId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn opinion(&self) -> &Opinion {
        &self.opinion
    }
}

impl Packable for Conflict {
    type Error = Error;

    fn packed_len(&self) -> usize {
        TRANSACTION_ID_LENGTH + OPINION_STATEMENT_LENGTH
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.id.pack(writer)?;
        self.opinion.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let transaction_id = TransactionId::unpack(reader)?;
        let opinion = Opinion::unpack(reader)?;

        Ok(Self {
            id: transaction_id,
            opinion,
        })
    }
}
