// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::{convert::Infallible, num::TryFromIntError};

use bee_block::{payload::milestone::MilestoneId, Error as BlockError};

/// Errors related to ledger types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    /// Block error.
    #[error("message error: {0}")]
    Block(#[from] BlockError),
    /// Invalid output count error.
    #[error("Invalid output count: {0}")]
    InvalidOutputCount(TryFromIntError),
    /// Unsupported output kind.
    #[error("unsupported output kind: {0}")]
    UnsupportedOutputKind(u8),
    /// Unsupported input kind.
    #[error("unsupported input kind: {0}")]
    UnsupportedInputKind(u8),
    /// Unsupported payload kind.
    #[error("unsupported payload kind: {0}")]
    UnsupportedPayloadKind(u32),
    /// Invalid payload kind.
    #[error("invalid payload kind: {0}")]
    InvalidPayloadKind(u32),
    /// Treasury amount mismatch.
    #[error("treasury amount mismatch: {0} != {1}")]
    TreasuryAmountMismatch(u64, u64),
    /// Migrated funds amount overflow.
    #[error("migrated funds amount overflow: {0}")]
    MigratedFundsAmountOverflow(u128),
    /// Invalid migrated funds amount.
    #[error("invalid migrated funds amount: {0}")]
    InvalidMigratedFundsAmount(u64),
    /// Consumed treasury output mismatch.
    #[error("consumed treasury output mismatch: {0} != {1}")]
    ConsumedTreasuryOutputMismatch(MilestoneId, MilestoneId),
    /// Negative balance.
    #[error("negative balance: {0}")]
    NegativeBalance(i64),
    /// Negative dust allowance.
    #[error("negative dust allowance: {0}")]
    NegativeDustAllowance(i64),
    /// Negative dust outputs.
    #[error("negative dust outputs: {0}")]
    NegativeDustOutputs(i64),
    /// Balance overflow.
    #[error("balance overflow: {0}")]
    BalanceOverflow(i128),
    /// Invalid balance.
    #[error("invalid balance: {0}")]
    InvalidBalance(u64),
    /// Balance diff overflow.
    #[error("balance diff overflow: {0}")]
    BalanceDiffOverflow(i128),
    /// Invalid balance diff.
    #[error("invalid balance diff: {0}")]
    InvalidBalanceDiff(i64),
    /// Packable option error happened.
    #[error("packable option error happened")]
    PackableOption,
    /// Invalid snapshot kind.
    #[error("invalid snapshot kind: {0}")]
    InvalidSnapshotKind(u8),
    /// Unsupported snapshot version.
    #[error("unsupported snapshot version: supports {0}, read {1}")]
    UnsupportedVersion(u8, u8),
    /// Missing consumed treasury.
    #[error("missing consumed treasury")]
    MissingConsumedTreasury,
    /// Milestone length mismatch.
    #[error("milestone length mismatch: expected {0}, got {1}")]
    MilestoneLengthMismatch(usize, usize),
}

impl From<Infallible> for Error {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}
