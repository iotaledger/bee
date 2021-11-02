// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module providing all types required to compute and maintain the ledger state.

pub mod snapshot;

mod balance;
mod balance_diff;
mod consumed_output;
mod created_output;
mod error;
mod ledger_index;
mod migration;
mod output_diff;
mod receipt;
mod treasury_diff;
mod treasury_output;
mod unspent;

pub use balance::Balance;
pub use balance_diff::{BalanceDiff, BalanceDiffs};

pub use consumed_output::ConsumedOutput;
pub use created_output::CreatedOutput;
pub use error::Error;
pub use ledger_index::LedgerIndex;
pub use migration::Migration;
pub use output_diff::OutputDiff;
pub use receipt::Receipt;
pub use treasury_diff::TreasuryDiff;
pub use treasury_output::TreasuryOutput;
pub use unspent::Unspent;
