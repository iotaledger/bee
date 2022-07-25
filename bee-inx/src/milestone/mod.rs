// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;

mod info;

use inx::proto;

pub use self::info::MilestoneInfo;
use crate::maybe_missing;

/// The [`Milestone`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
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
            milestone_info: maybe_missing!(value.milestone_info).try_into()?,
            milestone: value.milestone.map(TryInto::try_into).transpose()?,
        })
    }
}
