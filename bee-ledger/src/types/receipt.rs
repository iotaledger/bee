// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{error::Error, TreasuryOutput};

use bee_common::packable::{Packable, Read, Write};
use bee_message::{
    constants::IOTA_SUPPLY,
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
            migrated_amount = match migrated_amount.checked_add(funds.output().amount()) {
                Some(amount) => amount,
                None => {
                    return Err(Error::InvalidMigratedFundsAmount(
                        migrated_amount + funds.output().amount(),
                    ))
                }
            }
        }

        if migrated_amount > IOTA_SUPPLY {
            return Err(Error::InvalidMigratedFundsAmount(migrated_amount));
        }

        let created_treasury_output = match transaction.output() {
            Output::Treasury(output) => output,
            output => return Err(Error::UnsupportedOutputKind(output.kind())),
        };

        let created_amount = match consumed_treasury_output.inner().amount().checked_sub(migrated_amount) {
            Some(amount) => amount,
            None => {
                return Err(Error::InvalidMigratedFundsAmount(
                    consumed_treasury_output.inner().amount() - migrated_amount,
                ))
            }
        };

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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        Ok(Self::new(
            ReceiptPayload::unpack(reader)?,
            MilestoneIndex::unpack(reader)?,
        ))
    }
}
