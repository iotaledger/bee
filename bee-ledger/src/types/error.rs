// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{payload::milestone::MilestoneId, Error as MessageError};

/// Errors related to ledger types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Message error.
    #[error("Message error: {0}")]
    Message(#[from] MessageError),
    /// Unsupported output kind.
    #[error("Unsupported output kind: {0}")]
    UnsupportedOutputKind(u8),
    /// Unsupported input kind.
    #[error("Unsupported input kind: {0}")]
    UnsupportedInputKind(u8),
    /// Unsupported payload kind.
    #[error("Unsupported payload kind: {0}")]
    UnsupportedPayloadKind(u32),
    /// Treasury amount mismatch.
    #[error("Treasury amount mismatch: {0} != {1}")]
    TreasuryAmountMismatch(u64, u64),
    /// Migrated funds amount overflow.
    #[error("Migrated funds amount overflow: {0}")]
    MigratedFundsAmountOverflow(u128),
    /// Invalid migrated funds amount.
    #[error("Invalid migrated funds amount: {0}")]
    InvalidMigratedFundsAmount(u64),
    /// Consumed treasury output mismatch.
    #[error("Consumed treasury output mismatch: {0} != {1}")]
    ConsumedTreasuryOutputMismatch(MilestoneId, MilestoneId),
    /// Negative balance.
    #[error("Negative balance: {0}")]
    NegativeBalance(i64),
    /// Balance overflow.
    #[error("Balance overflow: {0}")]
    BalanceOverflow(i128),
    /// Invalid balance.
    #[error("Invalid balance: {0}")]
    InvalidBalance(u64),
    /// Balance diff overflow.
    #[error("Balance diff overflow: {0}")]
    BalanceDiffOverflow(i128),
    /// Invalid balance diff.
    #[error("Invalid balance diff: {0}")]
    InvalidBalanceDiff(i64),
    /// Packable option error happened.
    #[error("Packable option error happened")]
    PackableOption,
    #[error("Invalid snapshot kind: {0}")]
    InvalidSnapshotKind(u8),
    #[error("Unsupported snapshot version: supports {0}, read {1}")]
    UnsupportedVersion(u8, u8),
    #[error("Invalid payload kind: {0}")]
    InvalidPayloadKind(u32),
    #[error("")]
    MissingConsumedTreasury,
    #[error("Milestone length mismatch: expected {0}, got {1}")]
    MilestoneLengthMismatch(usize, usize),
}
