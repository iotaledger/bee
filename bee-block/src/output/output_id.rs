// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::str::FromStr;

use crypto::hashes::{blake2b::Blake2b256, Digest};
use packable::{bounded::BoundedU16, PackableExt};

use crate::{output::OUTPUT_INDEX_RANGE, payload::transaction::TransactionId, Error};

pub(crate) type OutputIndex = BoundedU16<{ *OUTPUT_INDEX_RANGE.start() }, { *OUTPUT_INDEX_RANGE.end() }>;

/// The identifier of an [`Output`](crate::output::Output).
#[derive(Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd, packable::Packable)]
#[packable(unpack_error = Error)]
pub struct OutputId {
    transaction_id: TransactionId,
    #[packable(unpack_error_with = Error::InvalidInputOutputIndex)]
    index: OutputIndex,
}

impl OutputId {
    /// The length of a [`OutputId`].
    pub const LENGTH: usize = TransactionId::LENGTH + core::mem::size_of::<u16>();

    /// Creates a new [`OutputId`].
    pub fn new(transaction_id: TransactionId, index: u16) -> Result<Self, Error> {
        index
            .try_into()
            .map(|index| Self { transaction_id, index })
            .map_err(Error::InvalidInputOutputIndex)
    }

    /// Returns the [`TransactionId`] of an [`OutputId`].
    #[inline(always)]
    pub fn transaction_id(&self) -> &TransactionId {
        &self.transaction_id
    }

    /// Returns the index of an [`OutputId`].
    #[inline(always)]
    pub fn index(&self) -> u16 {
        self.index.get()
    }

    /// Splits an [`OutputId`] into its [`TransactionId`] and index.
    #[inline(always)]
    pub fn split(self) -> (TransactionId, u16) {
        (self.transaction_id, self.index())
    }

    /// Hash the [`OutputId`] with BLAKE2b-256.
    #[inline(always)]
    pub fn hash(self) -> [u8; 32] {
        Blake2b256::digest(&self.pack_to_vec()).into()
    }
}

#[cfg(feature = "serde")]
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
        let bytes: [u8; OutputId::LENGTH] = prefix_hex::decode(s).map_err(Error::HexError)?;
        Self::try_from(bytes)
    }
}

impl core::fmt::Display for OutputId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut buffer = [0u8; OutputId::LENGTH];
        let (transaction_id, index) = buffer.split_at_mut(TransactionId::LENGTH);
        transaction_id.copy_from_slice(self.transaction_id.as_ref());
        index.copy_from_slice(&self.index().to_le_bytes());
        write!(f, "{}", prefix_hex::encode(buffer))
    }
}

impl core::fmt::Debug for OutputId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "OutputId({})", self)
    }
}
