// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{payload::milestone::MilestoneId, Error as MessageError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error {0}")]
    Io(#[from] std::io::Error),
    #[error("")]
    Message(#[from] MessageError),
    #[error("")]
    UnsupportedOutputKind(u8),
    #[error("")]
    UnsupportedInputKind(u8),
    #[error("")]
    UnsupportedPayloadKind(u32),
    #[error("Treasury amount mismatch: {0} != {1}")]
    TreasuryAmountMismatch(u64, u64),
    #[error("Invalid migrated funds amount: {0}")]
    InvalidMigratedFundsAmount(u64),
    #[error("Consumed treasury output mismatch: {0} != {1}")]
    ConsumedTreasuryOutputMismatch(MilestoneId, MilestoneId),
    #[error("")]
    // TODO
    Option,
}
