// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;
use inx::proto;

use crate::maybe_missing;

/// Represents a treasury output.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreasuryOutput {
    pub milestone_id: bee::payload::milestone::MilestoneId,
    pub amount: u64,
}

impl TryFrom<proto::TreasuryOutput> for TreasuryOutput {
    type Error = bee::InxError;

    fn try_from(value: proto::TreasuryOutput) -> Result<Self, Self::Error> {
        Ok(TreasuryOutput {
            milestone_id: maybe_missing!(value.milestone_id).try_into()?,
            amount: value.amount,
        })
    }
}

impl From<TreasuryOutput> for proto::TreasuryOutput {
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

impl TryFrom<proto::TreasuryUpdate> for TreasuryUpdate {
    type Error = bee::InxError;

    fn try_from(value: proto::TreasuryUpdate) -> Result<Self, Self::Error> {
        Ok(Self {
            milestone_index: value.milestone_index,
            created: maybe_missing!(value.created).try_into()?,
            consumed: maybe_missing!(value.consumed).try_into()?,
        })
    }
}

impl From<TreasuryUpdate> for proto::TreasuryUpdate {
    fn from(value: TreasuryUpdate) -> Self {
        Self {
            milestone_index: value.milestone_index,
            created: Some(value.created.into()),
            consumed: Some(value.consumed.into()),
        }
    }
}
