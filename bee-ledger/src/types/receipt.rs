// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{error::Error, TreasuryOutput};

use bee_common::packable::{Packable, Read, Write};
use bee_message::{
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

    pub fn validate(&self, consumed_treasury_output: &TreasuryOutput) -> Result<bool, Error> {
        let mut migrated_amount = 0;
        let transaction = match self.inner().transaction() {
            Payload::TreasuryTransaction(transaction) => transaction,
            payload => return Err(Error::UnsupportedPayloadKind(payload.kind())),
        };
        let created_treasury_output = match transaction.output() {
            Output::Treasury(output) => output,
            output => return Err(Error::UnsupportedOutputKind(output.kind())),
        };

        for funds in self.inner().funds() {
            // TODO check overflow
            migrated_amount += funds.output().amount();
        }

        // TODO check underflow
        if consumed_treasury_output.inner().amount() - migrated_amount != created_treasury_output.amount() {
            return Err(Error::TreasuryAmountMismatch(
                consumed_treasury_output.inner().amount() - migrated_amount,
                created_treasury_output.amount(),
            ));
        }

        // TODO useless bool ?
        Ok(true)
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
