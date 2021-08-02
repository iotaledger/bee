// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::payload::{transaction::TransactionId, PAYLOAD_LENGTH_MAX};

use bee_packable::{
    error::{PackPrefixError, UnpackPrefixError},
    BoundedU32, Packable, VecPrefix,
};

use alloc::vec;
use core::{
    convert::{Infallible, TryFrom},
    ops::Deref,
};

/// No [`Vec`] max length specified, so use [`PAYLOAD_LENGTH_MAX`] / length of [`Conflict`].
const PREFIXED_CONFLICTS_LENGTH_MAX: u32 =
    (PAYLOAD_LENGTH_MAX / (TransactionId::LENGTH + 2 * core::mem::size_of::<u8>())) as u32;

/// Provides a convenient collection of [`Conflict`]s.
/// Describes a vote in a given round for a transaction conflict.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(pack_error = PackPrefixError<Infallible>)]
#[packable(unpack_error = UnpackPrefixError<Infallible>)]
pub struct Conflicts {
    inner: VecPrefix<Conflict, BoundedU32<0, PREFIXED_CONFLICTS_LENGTH_MAX>>,
}

impl Default for Conflicts {
    fn default() -> Self {
        Self {
            inner: VecPrefix::try_from(vec![]).unwrap(),
        }
    }
}

impl Deref for Conflicts {
    type Target = [Conflict];

    fn deref(&self) -> &Self::Target {
        self.inner.as_slice()
    }
}

impl Conflicts {
    /// Creates a new [`Conflicts`] instance from a vector of [`Conflict`]s.
    pub fn new(inner: VecPrefix<Conflict, BoundedU32<0, PREFIXED_CONFLICTS_LENGTH_MAX>>) -> Self {
        Self { inner }
    }
}

/// Describes a vote in a given round for a transaction conflict.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct Conflict {
    /// ID of the conflicting transaction.
    transaction_id: TransactionId,
    /// The nodes opinion value in a given round.
    opinion: u8,
    /// Voting round number.
    round: u8,
}

impl Conflict {
    /// Creates a new [`Conflict`].
    pub fn new(transaction_id: TransactionId, opinion: u8, round: u8) -> Self {
        Self {
            transaction_id,
            opinion,
            round,
        }
    }

    /// Returns the ID of the conflicting transaction.
    pub fn transaction_id(&self) -> &TransactionId {
        &self.transaction_id
    }

    /// Returns the nodes opinion value in a given round.
    pub fn opinion(&self) -> u8 {
        self.opinion
    }

    /// Returns the voting round number.
    pub fn round(&self) -> u8 {
        self.round
    }
}
