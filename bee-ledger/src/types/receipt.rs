// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{error::Error, TreasuryOutput};

use bee_common::packable::{Packable, Read, Write};
use bee_message::{
    constants::IOTA_SUPPLY,
    input::Input,
    milestone::MilestoneIndex,
    output::Output,
    payload::{receipt::ReceiptPayload, Payload},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    inner: ReceiptPayload,
    included_in: MilestoneIndex,
}

impl Receipt {
    pub fn new(inner: ReceiptPayload, included_in: MilestoneIndex) -> Self {
        Self { inner, included_in }
    }

    pub fn inner(&self) -> &ReceiptPayload {
        &self.inner
    }

    pub fn included_in(&self) -> &MilestoneIndex {
        &self.included_in
    }

    pub fn validate(&self, consumed_treasury_output: &TreasuryOutput) -> Result<(), Error> {
        let mut migrated_amount: u64 = 0;
        let transaction = match self.inner().transaction() {
            Payload::TreasuryTransaction(transaction) => transaction,
            payload => return Err(Error::UnsupportedPayloadKind(payload.kind())),
        };

        for funds in self.inner().funds() {
            migrated_amount = migrated_amount
                .checked_add(funds.output().amount())
                .ok_or_else(|| Error::InvalidMigratedFundsAmount(migrated_amount + funds.output().amount()))?;
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
            input => return Err(Error::UnsupportedInputKind(input.kind())),
        };

        let created_treasury_output = match transaction.output() {
            Output::Treasury(output) => output,
            output => return Err(Error::UnsupportedOutputKind(output.kind())),
        };

        let created_amount = consumed_treasury_output
            .inner()
            .amount()
            .checked_sub(migrated_amount)
            .ok_or_else(|| {
                Error::InvalidMigratedFundsAmount(consumed_treasury_output.inner().amount() - migrated_amount)
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

impl Packable for Receipt {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.inner.packed_len() + self.inner.packed_len() + self.included_in.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.inner.pack(writer)?;
        self.included_in.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let inner = ReceiptPayload::unpack_inner::<R, CHECK>(reader)?;
        let included_in = MilestoneIndex::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self::new(inner, included_in))
    }
}
