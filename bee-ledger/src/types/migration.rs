// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{Receipt, TreasuryOutput};

/// A type to tie together the receipt, consumed treasury and created treasury of a migration.
pub struct Migration {
    receipt: Receipt,
    consumed_treasury: TreasuryOutput,
    created_treasury: TreasuryOutput,
}

impl Migration {
    /// Creates a new `Migration`.
    pub fn new(receipt: Receipt, consumed_treasury: TreasuryOutput, created_treasury: TreasuryOutput) -> Self {
        Self {
            receipt,
            consumed_treasury,
            created_treasury,
        }
    }

    /// Returns the receipt of the `Migration`.
    pub fn receipt(&self) -> &Receipt {
        &self.receipt
    }

    /// Returns the consumed treasury output of the `Migration`.
    pub fn consumed_treasury(&self) -> &TreasuryOutput {
        &self.consumed_treasury
    }

    /// Returns the created treasury output of the `Migration`.
    pub fn created_treasury(&self) -> &TreasuryOutput {
        &self.created_treasury
    }
}
