// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A crate that contains all types required to compute and maintain the ledger state.

#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![deny(missing_docs)]

pub mod snapshot;

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

pub use self::{
    consumed_output::ConsumedOutput, created_output::CreatedOutput, error::Error, ledger_index::LedgerIndex,
    migration::Migration, output_diff::OutputDiff, receipt::Receipt, treasury_diff::TreasuryDiff,
    treasury_output::TreasuryOutput, unspent::Unspent,
};
