// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::{
    payload::transaction::{SignatureLockedSingleOutput, UTXOInput},
    MessageId,
};

pub(crate) struct Output {
    message_id: MessageId,
    output_id: UTXOInput,
    output: SignatureLockedSingleOutput,
}

impl Packable for Output {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.message_id.packed_len() + self.output_id.packed_len() + self.output.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.message_id.pack(writer)?;
        self.output_id.pack(writer)?;
        self.output.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self {
            message_id: MessageId::unpack(reader)?,
            output_id: UTXOInput::unpack(reader)?,
            output: SignatureLockedSingleOutput::unpack(reader)?,
        })
    }
}
