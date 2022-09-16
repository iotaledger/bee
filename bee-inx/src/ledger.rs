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
    pub milestone_index_booked: bee::payload::milestone::MilestoneIndex,
    pub milestone_timestamp_booked: u32,
    pub output: Raw<bee::output::Output>,
}

/// Represents a spent output in the ledger.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedgerSpent {
    pub output: LedgerOutput,
    pub transaction_id_spent: bee::payload::transaction::TransactionId,
    pub milestone_index_spent: bee::payload::milestone::MilestoneIndex,
    pub milestone_timestamp_spent: u32,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnspentOutput {
    pub ledger_index: bee::payload::milestone::MilestoneIndex,
    pub output: LedgerOutput,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Marker {
    pub milestone_index: bee::payload::milestone::MilestoneIndex,
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

impl LedgerUpdate {
    /// If present, returns the contained `LedgerSpent` while consuming `self`.
    pub fn consumed(self) -> Option<LedgerSpent> {
        match self {
            Self::Consumed(ledger_spent) => Some(ledger_spent),
            _ => None,
        }
    }

    /// If present, returns the contained `LedgerOutput` while consuming `self`.
    pub fn created(self) -> Option<LedgerOutput> {
        match self {
            Self::Created(ledger_output) => Some(ledger_output),
            _ => None,
        }
    }

    /// If present, returns the `Marker` that denotes the beginning of a milestone while consuming `self`.
    pub fn begin(self) -> Option<Marker> {
        match self {
            Self::Begin(marker) => Some(marker),
            _ => None,
        }
    }

    /// If present, returns the `Marker` that denotes the end if present while consuming `self`.
    pub fn end(self) -> Option<Marker> {
        match self {
            Self::End(marker) => Some(marker),
            _ => None,
        }
    }
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

impl TryFrom<LedgerUpdate> for proto::ledger_update::Marker {
    type Error = bee::InxError;

    fn try_from(value: LedgerUpdate) -> Result<Self, Self::Error> {
        use proto::ledger_update::marker::MarkerType;
        let marker_type = match &value {
            LedgerUpdate::Begin(_) => MarkerType::Begin,
            LedgerUpdate::End(_) => MarkerType::End,
            _ => {
                return Err(Self::Error::MissingField("marker_type"));
            }
        };
        if let LedgerUpdate::Begin(marker) | LedgerUpdate::End(marker) = value {
            Ok(Self {
                milestone_index: marker.milestone_index.0,
                marker_type: marker_type.into(),
                consumed_count: marker.consumed_count as _,
                created_count: marker.created_count as _,
            })
        } else {
            unreachable!()
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

impl From<LedgerUpdate> for proto::LedgerUpdate {
    fn from(value: LedgerUpdate) -> Self {
        use proto::ledger_update::Op;
        Self {
            op: match value {
                LedgerUpdate::Consumed(consumed) => Op::Consumed(consumed.into()),
                LedgerUpdate::Created(created) => Op::Created(created.into()),
                marker => Op::BatchMarker(marker.try_into().unwrap()),
            }
            .into(),
        }
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

impl From<LedgerOutput> for proto::LedgerOutput {
    fn from(value: LedgerOutput) -> Self {
        Self {
            output_id: Some(value.output_id.into()),
            block_id: Some(value.block_id.into()),
            milestone_index_booked: value.milestone_index_booked.0,
            milestone_timestamp_booked: value.milestone_timestamp_booked,
            output: Some(value.output.into()),
        }
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

impl From<LedgerSpent> for proto::LedgerSpent {
    fn from(value: LedgerSpent) -> Self {
        Self {
            output: Some(value.output.into()),
            transaction_id_spent: Some(value.transaction_id_spent.into()),
            milestone_index_spent: value.milestone_index_spent.0,
            milestone_timestamp_spent: value.milestone_timestamp_spent,
        }
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

impl From<UnspentOutput> for proto::UnspentOutput {
    fn from(value: UnspentOutput) -> Self {
        Self {
            ledger_index: value.ledger_index.0,
            output: Some(value.output.into()),
        }
    }
}
