// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee::payload::milestone::MilestoneIndex;
use bee_block as bee;
use inx::proto;

use crate::{maybe_missing, Raw};

/// Represents a new output in the ledger.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedgerOutput {
    pub output_id: bee::output::OutputId,
    pub block_id: bee::BlockId,
    pub milestone_index_booked: MilestoneIndex,
    pub milestone_timestamp_booked: u32,
    pub output: Raw<bee::output::Output>,
}

/// Represents a spent output in the ledger.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedgerSpent {
    pub output: LedgerOutput,
    pub transaction_id_spent: bee::payload::transaction::TransactionId,
    pub milestone_index_spent: MilestoneIndex,
    pub milestone_timestamp_spent: u32,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnspentOutput {
    pub ledger_index: MilestoneIndex,
    pub output: LedgerOutput,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Marker {
    pub milestone_index: MilestoneIndex,
    pub consumed_count: usize,
    pub created_count: usize,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LedgerUpdate {
    Consumed(LedgerSpent),
    Created(LedgerOutput),
    Begin(Marker),
    End(Marker),
}

impl From<proto::ledger_update::Marker> for Marker {
    fn from(value: proto::ledger_update::Marker) -> Self {
        Self {
            milestone_index: value.milestone_index.into(),
            consumed_count: value.consumed_count as usize,
            created_count: value.created_count as usize,
        }
    }
}

impl From<proto::ledger_update::Marker> for LedgerUpdate {
    fn from(value: proto::ledger_update::Marker) -> Self {
        use proto::ledger_update::marker::MarkerType as proto;
        match value.marker_type() {
            proto::Begin => Self::Begin(value.into()),
            proto::End => Self::End(value.into()),
        }
    }
}

impl TryFrom<proto::LedgerUpdate> for LedgerUpdate {
    type Error = bee::InxError;

    fn try_from(value: proto::LedgerUpdate) -> Result<Self, Self::Error> {
        use proto::ledger_update::Op as proto;
        Ok(match maybe_missing!(value.op) {
            proto::BatchMarker(marker) => marker.into(),
            proto::Consumed(consumed) => LedgerUpdate::Consumed(consumed.try_into()?),
            proto::Created(created) => LedgerUpdate::Created(created.try_into()?),
        })
    }
}

impl TryFrom<proto::LedgerOutput> for LedgerOutput {
    type Error = bee::InxError;

    fn try_from(value: proto::LedgerOutput) -> Result<Self, Self::Error> {
        Ok(Self {
            output_id: maybe_missing!(value.output_id).try_into()?,
            block_id: maybe_missing!(value.block_id).try_into()?,
            milestone_index_booked: value.milestone_index_booked.into(),
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
            milestone_index_spent: value.milestone_index_spent.into(),
            milestone_timestamp_spent: value.milestone_timestamp_spent,
        })
    }
}

impl TryFrom<proto::UnspentOutput> for UnspentOutput {
    type Error = bee::InxError;

    fn try_from(value: proto::UnspentOutput) -> Result<Self, Self::Error> {
        Ok(Self {
            ledger_index: value.ledger_index.into(),
            output: maybe_missing!(value.output).try_into()?,
        })
    }
}
