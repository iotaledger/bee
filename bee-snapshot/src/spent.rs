// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{output::Output, Error};

use bee_common::packable::{Packable, Read, Write};
use bee_message::payload::transaction::TransactionId;

pub(crate) struct Spent {
    output: Output,
    transaction_id: TransactionId,
}

impl Packable for Spent {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.output.packed_len() + self.transaction_id.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.output.pack(writer)?;
        self.transaction_id.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            output: Output::unpack(reader)?,
            transaction_id: TransactionId::unpack(reader)?,
        })
    }
}
