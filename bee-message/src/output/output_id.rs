// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{output::OUTPUT_INDEX_RANGE, payload::transaction::TransactionId, util::hex_decode, Error};

use bee_packable::bounded::BoundedU16;

use core::str::FromStr;

pub(crate) type InputOutputIndex = BoundedU16<{ *OUTPUT_INDEX_RANGE.start() }, { *OUTPUT_INDEX_RANGE.end() }>;

/// The identifier of an `Output`.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, bee_packable::Packable)]
#[packable(unpack_error = Error)]
pub struct OutputId {
    transaction_id: TransactionId,
    #[packable(unpack_error_with = Error::InvalidInputOutputIndex)]
    index: InputOutputIndex,
}

impl OutputId {
    /// The length of a [`OutputId`].
    pub const LENGTH: usize = TransactionId::LENGTH + std::mem::size_of::<u16>();

    /// Creates a new [`OutputId`].
    pub fn new(transaction_id: TransactionId, index: u16) -> Result<Self, Error> {
        index
            .try_into()
            .map(|index| Self { transaction_id, index })
            .map_err(Error::InvalidInputOutputIndex)
    }

    /// Returns the `TransactionId` of an `OutputId`.
    #[inline(always)]
    pub fn transaction_id(&self) -> &TransactionId {
        &self.transaction_id
    }

    /// Returns the index of an `OutputId`.
    #[inline(always)]
    pub fn index(&self) -> u16 {
        self.index.get()
    }

    /// Splits an `OutputId` into its `TransactionId` and index.
    #[inline(always)]
    pub fn split(self) -> (TransactionId, u16) {
        (self.transaction_id, self.index())
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
        write!(f, "{}{}", self.transaction_id, hex::encode(self.index().to_le_bytes()))
    }
}

impl core::fmt::Debug for OutputId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "OutputId({})", self)
    }
}
