// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{error::Error, TreasuryOutput};

use bee_message::{
    constants::IOTA_SUPPLY,
    input::Input,
    milestone::MilestoneIndex,
    output::Output,
    payload::{receipt::ReceiptPayload, Payload},
};
use bee_packable::Packable;

/// A type that wraps a receipt and the index of the milestone in which it was included.
#[derive(Clone, Debug, Eq, PartialEq, Packable)]
pub struct Receipt {
    inner: ReceiptPayload,
    included_in: MilestoneIndex,
}

impl Receipt {
    /// Creates a new `Receipt`.
    pub fn new(inner: ReceiptPayload, included_in: MilestoneIndex) -> Self {
        Self { inner, included_in }
    }

    /// Returns the inner receipt of the `Receipt`.
    pub fn inner(&self) -> &ReceiptPayload {
        &self.inner
    }

    /// Returns the index of the milestone in which the `Receipt` was included.
    pub fn included_in(&self) -> &MilestoneIndex {
        &self.included_in
    }

    /// Semantically validates the `Receipt`.
    pub fn validate(&self, consumed_treasury_output: &TreasuryOutput) -> Result<(), Error> {
        let mut migrated_amount: u64 = 0;
        let transaction = match self.inner().transaction() {
            Payload::TreasuryTransaction(transaction) => transaction,
            Payload::Indexation(_) | Payload::Milestone(_) | Payload::Receipt(_) | Payload::Transaction(_) => {
                return Err(Error::UnsupportedPayloadKind(self.inner().transaction().kind()));
            }
        };

        for funds in self.inner().funds() {
            migrated_amount = migrated_amount.checked_add(funds.output().amount()).ok_or_else(|| {
                Error::MigratedFundsAmountOverflow(migrated_amount as u128 + funds.output().amount() as u128)
            })?;
        }

        if migrated_amount > IOTA_SUPPLY {
            return Err(Error::InvalidMigratedFundsAmount(migrated_amount));
        }

        match transaction.input() {
            Input::Treasury(input) => {
                if input.milestone_id() != consumed_treasury_output.milestone_id() {
                    return Err(Error::ConsumedTreasuryOutputMismatch(
                        *input.milestone_id(),
                        *consumed_treasury_output.milestone_id(),
                    ));
                }
            }
            Input::Utxo(_) => return Err(Error::UnsupportedInputKind(transaction.input().kind())),
        };

        let created_treasury_output = match transaction.output() {
            Output::Treasury(output) => output,
            Output::SignatureLockedDustAllowance(_) | Output::SignatureLockedSingle(_) => {
                return Err(Error::UnsupportedOutputKind(transaction.output().kind()));
            }
        };

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
