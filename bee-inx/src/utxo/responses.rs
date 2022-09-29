// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{bee, inx, return_err_if_none, Raw};

/// Represents a new output in the ledger.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedgerOutput {
    pub output_id: bee::OutputId,
    pub block_id: bee::BlockId,
    pub milestone_index_booked: bee::MilestoneIndex,
    pub milestone_timestamp_booked: u32,
    pub output: Raw<bee::Output>,
}

impl TryFrom<inx::LedgerOutput> for LedgerOutput {
    type Error = bee::InxError;

    fn try_from(value: inx::LedgerOutput) -> Result<Self, Self::Error> {
        Ok(Self {
            output_id: return_err_if_none!(value.output_id).try_into()?,
            block_id: return_err_if_none!(value.block_id).try_into()?,
            milestone_index_booked: value.milestone_index_booked.into(),
            milestone_timestamp_booked: value.milestone_timestamp_booked,
            output: return_err_if_none!(value.output).into(),
        })
    }
}

impl From<LedgerOutput> for inx::LedgerOutput {
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

/// Represents a spent output in the ledger.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedgerSpent {
    pub output: LedgerOutput,
    pub transaction_id_spent: bee::TransactionId,
    pub milestone_index_spent: bee::MilestoneIndex,
    pub milestone_timestamp_spent: u32,
}

impl TryFrom<inx::LedgerSpent> for LedgerSpent {
    type Error = bee::InxError;

    fn try_from(value: inx::LedgerSpent) -> Result<Self, Self::Error> {
        Ok(Self {
            output: return_err_if_none!(value.output).try_into()?,
            transaction_id_spent: return_err_if_none!(value.transaction_id_spent).try_into()?,
            milestone_index_spent: value.milestone_index_spent.into(),
            milestone_timestamp_spent: value.milestone_timestamp_spent,
        })
    }
}

impl From<LedgerSpent> for inx::LedgerSpent {
    fn from(value: LedgerSpent) -> Self {
        Self {
            output: Some(value.output.into()),
            transaction_id_spent: Some(value.transaction_id_spent.into()),
            milestone_index_spent: value.milestone_index_spent.0,
            milestone_timestamp_spent: value.milestone_timestamp_spent,
        }
    }
}

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnspentOutput {
    pub ledger_index: bee::MilestoneIndex,
    pub output: LedgerOutput,
}

impl TryFrom<inx::UnspentOutput> for UnspentOutput {
    type Error = bee::InxError;

    fn try_from(value: inx::UnspentOutput) -> Result<Self, Self::Error> {
        Ok(Self {
            ledger_index: value.ledger_index.into(),
            output: return_err_if_none!(value.output).try_into()?,
        })
    }
}

impl From<UnspentOutput> for inx::UnspentOutput {
    fn from(value: UnspentOutput) -> Self {
        Self {
            ledger_index: value.ledger_index.0,
            output: Some(value.output.into()),
        }
    }
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

impl TryFrom<inx::LedgerUpdate> for LedgerUpdate {
    type Error = bee::InxError;

    fn try_from(value: inx::LedgerUpdate) -> Result<Self, Self::Error> {
        use crate::inx::Op;
        Ok(match return_err_if_none!(value.op) {
            Op::BatchMarker(marker) => marker.into(),
            Op::Consumed(consumed) => LedgerUpdate::Consumed(consumed.try_into()?),
            Op::Created(created) => LedgerUpdate::Created(created.try_into()?),
        })
    }
}

impl From<LedgerUpdate> for inx::LedgerUpdate {
    fn from(value: LedgerUpdate) -> Self {
        use crate::inx::Op;
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

#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Marker {
    pub milestone_index: bee::MilestoneIndex,
    pub consumed_count: usize,
    pub created_count: usize,
}

impl From<inx::ledger_update::Marker> for Marker {
    fn from(value: inx::ledger_update::Marker) -> Self {
        Self {
            milestone_index: value.milestone_index.into(),
            consumed_count: value.consumed_count as usize,
            created_count: value.created_count as usize,
        }
    }
}

impl From<inx::Marker> for LedgerUpdate {
    fn from(value: inx::Marker) -> Self {
        use crate::inx::MarkerType;
        match value.marker_type() {
            MarkerType::Begin => Self::Begin(value.into()),
            MarkerType::End => Self::End(value.into()),
        }
    }
}

impl TryFrom<LedgerUpdate> for inx::ledger_update::Marker {
    type Error = bee::InxError;

    fn try_from(value: LedgerUpdate) -> Result<Self, Self::Error> {
        use crate::inx::MarkerType;
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

/// Represents a treasury output.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreasuryOutput {
    pub milestone_id: bee::MilestoneId,
    pub amount: u64,
}

impl TryFrom<inx::TreasuryOutput> for TreasuryOutput {
    type Error = bee::InxError;

    fn try_from(value: inx::TreasuryOutput) -> Result<Self, Self::Error> {
        Ok(TreasuryOutput {
            milestone_id: return_err_if_none!(value.milestone_id).try_into()?,
            amount: value.amount,
        })
    }
}

impl From<TreasuryOutput> for inx::TreasuryOutput {
    fn from(value: TreasuryOutput) -> Self {
        Self {
            milestone_id: Some(value.milestone_id.into()),
            amount: value.amount,
        }
    }
}

/// Represents an update to the treasury.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreasuryUpdate {
    pub milestone_index: u32,
    pub created: TreasuryOutput,
    pub consumed: TreasuryOutput,
}

impl TryFrom<inx::TreasuryUpdate> for TreasuryUpdate {
    type Error = bee::InxError;

    fn try_from(value: inx::TreasuryUpdate) -> Result<Self, Self::Error> {
        Ok(Self {
            milestone_index: value.milestone_index,
            created: return_err_if_none!(value.created).try_into()?,
            consumed: return_err_if_none!(value.consumed).try_into()?,
        })
    }
}

impl From<TreasuryUpdate> for inx::TreasuryUpdate {
    fn from(value: TreasuryUpdate) -> Self {
        Self {
            milestone_index: value.milestone_index,
            created: Some(value.created.into()),
            consumed: Some(value.consumed.into()),
        }
    }
}

// message OutputResponse {
//   uint32 ledger_index = 1;
//   oneof payload {
//     LedgerOutput output = 2;
//     LedgerSpent spent = 3;
//   }
// }

/// Represents an output response.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputResponse {
    pub ledger_index: bee::MilestoneIndex,
    pub payload: Option<OutputResponsePayload>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputResponsePayload {
    LedgerOutput(LedgerOutput),
    LedgerSpent(LedgerSpent),
}

impl TryFrom<inx::OutputResponse> for OutputResponse {
    type Error = bee::InxError;

    fn try_from(value: inx::OutputResponse) -> Result<Self, Self::Error> {
        use crate::inx::output_response::Payload::*;
        Ok(Self {
            ledger_index: value.ledger_index.into(),
            payload: if let Some(payload) = value.payload {
                Some(match payload {
                    Output(ledger_output) => OutputResponsePayload::LedgerOutput(ledger_output.try_into()?),
                    Spent(ledger_spent) => OutputResponsePayload::LedgerSpent(ledger_spent.try_into()?),
                })
            } else {
                None
            },
        })
    }
}

impl From<OutputResponse> for inx::OutputResponse {
    fn from(value: OutputResponse) -> Self {
        use OutputResponsePayload::*;
        Self {
            ledger_index: value.ledger_index.0,
            payload: value.payload.map(|payload| match payload {
                LedgerOutput(ledger_output) => inx::output_response::Payload::Output(ledger_output.into()),
                LedgerSpent(ledger_spent) => inx::output_response::Payload::Spent(ledger_spent.into()),
            }),
        }
    }
}
