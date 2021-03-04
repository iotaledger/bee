// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::model::{Receipt, TreasuryOutput};

pub struct Migration {
    receipt: Receipt,
    created_treasury: TreasuryOutput,
    consumed_treasury: TreasuryOutput,
}

impl Migration {
    pub fn new(receipt: Receipt, created_treasury: TreasuryOutput, consumed_treasury: TreasuryOutput) -> Self {
        Self {
            receipt,
            created_treasury,
            consumed_treasury,
        }
    }

    pub fn receipt(&self) -> &Receipt {
        &self.receipt
    }

    pub fn created_treasury(&self) -> &TreasuryOutput {
        &self.created_treasury
    }

    pub fn consumed_treasury(&self) -> &TreasuryOutput {
        &self.consumed_treasury
    }
}
