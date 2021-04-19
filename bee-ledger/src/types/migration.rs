// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{Receipt, TreasuryOutput};

pub struct Migration {
    receipt: Receipt,
    consumed_treasury: TreasuryOutput,
    created_treasury: TreasuryOutput,
}

impl Migration {
    pub fn new(receipt: Receipt, consumed_treasury: TreasuryOutput, created_treasury: TreasuryOutput) -> Self {
        Self {
            receipt,
            consumed_treasury,
            created_treasury,
        }
    }

    pub fn receipt(&self) -> &Receipt {
        &self.receipt
    }

    pub fn consumed_treasury(&self) -> &TreasuryOutput {
        &self.consumed_treasury
    }

    pub fn created_treasury(&self) -> &TreasuryOutput {
        &self.created_treasury
    }
}
