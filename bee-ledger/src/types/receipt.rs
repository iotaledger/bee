// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    constant::TOKEN_SUPPLY,
    payload::milestone::{MilestoneIndex, ReceiptMilestoneOption},
};

use crate::types::{error::Error, TreasuryOutput};

/// A type that wraps a receipt and the index of the milestone in which it was included.
#[derive(Clone, Debug, Eq, PartialEq, packable::Packable)]
pub struct Receipt {
    inner: ReceiptMilestoneOption,
    included_in: MilestoneIndex,
}

impl Receipt {
    /// Creates a new `Receipt`.
    pub fn new(inner: ReceiptMilestoneOption, included_in: MilestoneIndex) -> Self {
        Self { inner, included_in }
    }

    /// Returns the inner receipt of the `Receipt`.
    pub fn inner(&self) -> &ReceiptMilestoneOption {
        &self.inner
    }

    /// Returns the index of the milestone in which the `Receipt` was included.
    pub fn included_in(&self) -> &MilestoneIndex {
        &self.included_in
    }

    /// Semantically validates the `Receipt`.
    pub fn validate(&self, consumed_treasury_output: &TreasuryOutput) -> Result<(), Error> {
        let mut migrated_amount: u64 = 0;
        let transaction = self.inner().transaction();

        for funds in self.inner().funds() {
            migrated_amount = migrated_amount
                .checked_add(funds.amount())
                .ok_or_else(|| Error::MigratedFundsAmountOverflow(migrated_amount as u128 + funds.amount() as u128))?;
        }

        if migrated_amount > TOKEN_SUPPLY {
            return Err(Error::InvalidMigratedFundsAmount(migrated_amount));
        }

        let input = transaction.input();

        if input.milestone_id() != consumed_treasury_output.milestone_id() {
            return Err(Error::ConsumedTreasuryOutputMismatch(
                *input.milestone_id(),
                *consumed_treasury_output.milestone_id(),
            ));
        }

        let created_treasury_output = transaction.output();
        let created_amount = consumed_treasury_output
            .inner()
            .amount()
            .checked_sub(migrated_amount)
            .ok_or_else(|| {
                Error::MigratedFundsAmountOverflow(
                    (consumed_treasury_output.inner().amount() as i128 - migrated_amount as i128) as u128,
                )
            })?;

        if created_amount != created_treasury_output.amount() {
            return Err(Error::TreasuryAmountMismatch(
                created_amount,
                created_treasury_output.amount(),
            ));
        }

        Ok(())
    }
}
