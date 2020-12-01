// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::payload::transaction::OutputId;

pub struct Unspent(OutputId);

impl From<OutputId> for Unspent {
    fn from(id: OutputId) -> Self {
        Unspent(id)
    }
}

impl Unspent {
    pub fn new(output_id: OutputId) -> Self {
        output_id.into()
    }

    pub fn id(&self) -> &OutputId {
        &self.0
    }
}

impl Packable for Unspent {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self(OutputId::unpack(reader)?))
    }
}
