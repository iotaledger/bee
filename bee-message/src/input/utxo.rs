// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{output::OutputId, payload::transaction::TransactionId, Error};

use bee_common::packable::{Packable, Read, Write};

use core::{convert::From, str::FromStr};

/// An `UtxoInput` represents an input block within a transaction payload, and references
/// the output from a previous transaction.
///
/// It is part of the transaction [Essense](crate::payload::transaction::Essence).
///
/// Spec: #iota-protocol-rfc-draft
/// <https://github.com/luca-moser/protocol-rfcs/blob/signed-tx-payload/text/0000-transaction-payload/0000-transaction-payload.md#serialized-layout>
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct UtxoInput(OutputId);

impl UtxoInput {
    /// The kind of transaction input: `0` as defined by the protocol.
    pub const KIND: u8 = 0;

    /// Construct an `UtxoInput` from a previous transaction ID, and the index of the output block.
    pub fn new(id: TransactionId, index: u16) -> Result<Self, Error> {
        Ok(Self(OutputId::new(id, index)?))
    }

    /// Return the underlying `OutputId` that this `UtxoInput` references.
    pub fn output_id(&self) -> &OutputId {
        &self.0
    }
}

#[cfg(feature = "serde")]
string_serde_impl!(UtxoInput);

impl From<OutputId> for UtxoInput {
    fn from(id: OutputId) -> Self {
        UtxoInput(id)
    }
}

impl FromStr for UtxoInput {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UtxoInput(OutputId::from_str(s)?))
    }
}

impl core::fmt::Display for UtxoInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::fmt::Debug for UtxoInput {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "UtxoInput({})", self.0)
    }
}

impl Packable for UtxoInput {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.0.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.0.pack(writer)
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self(OutputId::unpack_inner::<R, CHECK>(reader)?))
    }
}
