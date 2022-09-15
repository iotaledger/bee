// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;
use inx::proto;

/// The [`MilestoneInfo`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneInfo {
    /// The [`MilestoneId`](bee::payload::milestone::MilestoneId) of the milestone.
    pub milestone_id: Option<bee::payload::milestone::MilestoneId>,
    /// The milestone index.
    pub milestone_index: bee::payload::milestone::MilestoneIndex,
    /// The timestamp of the milestone.
    pub milestone_timestamp: u32,
}

impl TryFrom<proto::MilestoneInfo> for MilestoneInfo {
    type Error = bee::InxError;

    fn try_from(value: proto::MilestoneInfo) -> Result<Self, Self::Error> {
        Ok(MilestoneInfo {
            milestone_id: value.milestone_id.map(TryInto::try_into).transpose()?,
            milestone_index: value.milestone_index.into(),
            milestone_timestamp: value.milestone_timestamp,
        })
    }
}

impl From<MilestoneInfo> for proto::MilestoneInfo {
    fn from(value: MilestoneInfo) -> Self {
        Self {
            milestone_id: value.milestone_id.map(Into::into),
            milestone_index: value.milestone_index.0,
            milestone_timestamp: value.milestone_timestamp,
        }
    }
}
