// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;

use inx::proto;

/// The [`MilestoneInfo`] type.
#[derive(Clone, Debug, PartialEq)]
pub struct MilestoneInfo {
    /// The [`MilestoneId`](bee::payload::milestone::MilestoneId) of the milestone.
    pub milestone_id: Option<bee::payload::milestone::MilestoneId>,
    /// The milestone index.
    pub milestone_index: u32,
    /// The timestamp of the milestone.
    pub milestone_timestamp: u32,
}

impl TryFrom<proto::MilestoneInfo> for MilestoneInfo {
    type Error = bee::InxError;

    fn try_from(value: proto::MilestoneInfo) -> Result<Self, Self::Error> {
        Ok(MilestoneInfo {
            milestone_id: value.milestone_id.map(TryInto::try_into).transpose()?,
            milestone_index: value.milestone_index,
            milestone_timestamp: value.milestone_timestamp,
        })
    }
}
