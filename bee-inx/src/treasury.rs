// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;
use inx::proto;

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
            milestone_id: value
                .milestone_id
                .ok_or(bee::InxError::MissingField("milestone_id"))?
                .try_into()?,
            amount: value.amount,
        })
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
            created: value.created.ok_or(Self::Error::MissingField("created"))?.try_into()?,
            consumed: value
                .consumed
                .ok_or(Self::Error::MissingField("consumed"))?
                .try_into()?,
        })
    }
}
