// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    payload::transaction::{output::OutputId, TransactionId},
    Error,
};

use bee_common::packable::{Packable, Read, Write};

use core::{convert::From, str::FromStr};

#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct UTXOInput(OutputId);

string_serde_impl!(UTXOInput);

impl From<OutputId> for UTXOInput {
    fn from(id: OutputId) -> Self {
        UTXOInput(id)
    }
}

impl FromStr for UTXOInput {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UTXOInput(OutputId::from_str(s)?))
    }
}

impl UTXOInput {
    pub fn new(id: TransactionId, index: u16) -> Result<Self, Error> {
        Ok(Self(OutputId::new(id, index)?))
    }

    pub fn output_id(&self) -> &OutputId {
        &self.0
    }
}

impl core::fmt::Display for UTXOInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::fmt::Debug for UTXOInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "UTXOInput({})", self.0)
    }
}

impl Packable for UTXOInput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self(OutputId::unpack(reader)?))
    }
}
