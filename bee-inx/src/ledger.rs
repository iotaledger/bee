// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;
use inx::proto;

use crate::{maybe_missing, Raw};

/// Represents a new output in the ledger.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedgerOutput {
    pub output_id: bee::output::OutputId,
    pub block_id: bee::BlockId,
    pub milestone_index_booked: u32,
    pub milestone_timestamp_booked: u32,
    pub output: Raw<bee::output::Output>,
}

/// Represents a spent output in the ledger.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedgerSpent {
    pub output: LedgerOutput,
    pub transaction_id_spent: bee::payload::transaction::TransactionId,
    pub milestone_index_spent: u32,
    pub milestone_timestamp_spent: u32,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnspentOutput {
    pub ledger_index: u32,
    pub output: LedgerOutput,
}

/// Represents an update to ledger.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedgerUpdate {
    pub milestone_index: u32,
    pub created: Box<[LedgerOutput]>,
    pub consumed: Box<[LedgerSpent]>,
}

impl TryFrom<proto::LedgerOutput> for LedgerOutput {
    type Error = bee::InxError;

    fn try_from(value: proto::LedgerOutput) -> Result<Self, Self::Error> {
        Ok(Self {
            output_id: maybe_missing!(value.output_id).try_into()?,
            block_id: maybe_missing!(value.block_id).try_into()?,
            milestone_index_booked: value.milestone_index_booked,
            milestone_timestamp_booked: value.milestone_timestamp_booked,
            output: maybe_missing!(value.output).into(),
        })
    }
}

impl TryFrom<proto::LedgerSpent> for LedgerSpent {
    type Error = bee::InxError;

    fn try_from(value: proto::LedgerSpent) -> Result<Self, Self::Error> {
        Ok(Self {
            output: maybe_missing!(value.output).try_into()?,
            transaction_id_spent: maybe_missing!(value.transaction_id_spent).try_into()?,
            milestone_index_spent: value.milestone_index_spent,
            milestone_timestamp_spent: value.milestone_timestamp_spent,
        })
    }
}

impl TryFrom<proto::LedgerUpdate> for LedgerUpdate {
    type Error = bee::InxError;

    fn try_from(value: proto::LedgerUpdate) -> Result<Self, Self::Error> {
        let mut created: Vec<LedgerOutput> = Vec::with_capacity(value.created.len());
        for c in value.created {
            created.push(c.try_into()?);
        }

        let mut consumed: Vec<LedgerSpent> = Vec::with_capacity(value.consumed.len());
        for c in value.consumed {
            consumed.push(c.try_into()?);
        }

        Ok(Self {
            milestone_index: value.milestone_index,
            created: created.into_boxed_slice(),
            consumed: consumed.into_boxed_slice(),
        })
    }
}

impl TryFrom<proto::UnspentOutput> for UnspentOutput {
    type Error = bee::InxError;

    fn try_from(value: proto::UnspentOutput) -> Result<Self, Self::Error> {
        Ok(Self {
            ledger_index: value.ledger_index,
            output: maybe_missing!(value.output).try_into()?,
        })
    }
}
