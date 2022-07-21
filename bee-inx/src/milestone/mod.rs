// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;

mod info;

pub use self::info::MilestoneInfo;
use inx::proto;

/// The [`Milestone`] type.
#[derive(Clone, Debug, PartialEq)]
pub struct Milestone {
    /// Information about the milestone.
    pub milestone_info: MilestoneInfo,
    /// The raw bytes of the milestone.
    pub milestone: Option<bee::payload::MilestonePayload>,
}

impl TryFrom<proto::Milestone> for Milestone {
    type Error = bee::InxError;

    fn try_from(value: proto::Milestone) -> Result<Self, Self::Error> {
        Ok(Milestone {
            milestone_info: value
                .milestone_info
                .ok_or(Self::Error::MissingField("milestone_info"))?
                .try_into()?,
            milestone: value.milestone.map(TryInto::try_into).transpose()?,
        })
    }
}
