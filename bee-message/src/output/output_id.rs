// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{output::OUTPUT_INDEX_RANGE, payload::transaction::TransactionId, util::hex_decode, Error};

use bee_common::packable::{Packable, Read, Write};

use core::str::FromStr;

/// The identifier of an `Output`.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct OutputId {
    transaction_id: TransactionId,
    index: u16,
}

impl OutputId {
    /// The length of a [`OutputId`].
    pub const LENGTH: usize = TransactionId::LENGTH + std::mem::size_of::<u16>();

    /// Creates a new [`OutputId`].
    pub fn new(transaction_id: TransactionId, index: u16) -> Result<Self, Error> {
        if !OUTPUT_INDEX_RANGE.contains(&index) {
            return Err(Error::InvalidInputOutputIndex(index));
        }

        Ok(Self { transaction_id, index })
    }

    /// Returns the `TransactionId` of an `OutputId`.
    #[inline(always)]
    pub fn transaction_id(&self) -> &TransactionId {
        &self.transaction_id
    }

    /// Returns the index of an `OutputId`.
    #[inline(always)]
    pub fn index(&self) -> u16 {
        self.index
    }

    /// Splits an `OutputId` into its `TransactionId` and index.
    #[inline(always)]
    pub fn split(self) -> (TransactionId, u16) {
        (self.transaction_id, self.index)
    }
}

#[cfg(feature = "serde1")]
string_serde_impl!(OutputId);

impl TryFrom<[u8; OutputId::LENGTH]> for OutputId {
    type Error = Error;

    fn try_from(bytes: [u8; OutputId::LENGTH]) -> Result<Self, Self::Error> {
        let (transaction_id, index) = bytes.split_at(TransactionId::LENGTH);

        Self::new(
            // Unwrap is fine because size is already known and valid.
            From::<[u8; TransactionId::LENGTH]>::from(transaction_id.try_into().unwrap()),
            // Unwrap is fine because size is already known and valid.
            u16::from_le_bytes(index.try_into().unwrap()),
        )
    }
}

impl FromStr for OutputId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(hex_decode(s)?)
    }
}

impl core::fmt::Display for OutputId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}{}", self.transaction_id, hex::encode(self.index.to_le_bytes()))
    }
}

impl core::fmt::Debug for OutputId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "OutputId({})", self)
    }
}

impl Packable for OutputId {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.transaction_id.packed_len() + self.index.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.transaction_id.pack(writer)?;
        self.index.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let transaction_id = TransactionId::unpack_inner::<R, CHECK>(reader)?;
        let index = u16::unpack_inner::<R, CHECK>(reader)?;

        Self::new(transaction_id, index)
    }
}
