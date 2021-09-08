// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::payload::{fpc::Opinion, transaction::TransactionId, MessageUnpackError};

use bee_packable::Packable;

/// Describes a vote in a given round for a transaction conflict.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
#[packable(unpack_error = MessageUnpackError)]
pub struct Conflict {
    /// Identifier of the conflicting transaction.
    transaction_id: TransactionId,
    /// The node's opinion value in a given round.
    opinion: Opinion,
    /// Voting round number.
    round: u8,
}

impl Conflict {
    /// Creates a new [`Conflict`].
    pub fn new(transaction_id: TransactionId, opinion: Opinion, round: u8) -> Self {
        Self {
            transaction_id,
            opinion,
            round,
        }
    }

    /// Returns the identifier of the conflicting transaction.
    pub fn transaction_id(&self) -> &TransactionId {
        &self.transaction_id
    }

    /// Returns the node's opinion value in a given round.
    pub fn opinion(&self) -> Opinion {
        self.opinion
    }

    /// Returns the voting round number.
    pub fn round(&self) -> u8 {
        self.round
    }
}
