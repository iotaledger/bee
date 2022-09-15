// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;

mod info;

use inx::proto;

pub use self::info::MilestoneInfo;
use crate::{maybe_missing, ProtocolParameters, Raw};

/// The [`Milestone`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Milestone {
    /// Information about the milestone.
    pub milestone_info: MilestoneInfo,
    /// The raw bytes of the milestone.
    pub milestone: Raw<bee::payload::MilestonePayload>,
}

impl TryFrom<proto::Milestone> for Milestone {
    type Error = bee::InxError;

    fn try_from(value: proto::Milestone) -> Result<Self, Self::Error> {
        Ok(Self {
            milestone_info: maybe_missing!(value.milestone_info).try_into()?,
            milestone: maybe_missing!(value.milestone).data.into(),
        })
    }
}

impl From<Milestone> for proto::Milestone {
    fn from(value: Milestone) -> Self {
        Self {
            milestone_info: Some(value.milestone_info.into()),
            milestone: Some(value.milestone.into()),
        }
    }
}

/// The [`Milestone`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneAndProtocolParameters {
    pub milestone: Milestone,
    pub current_protocol_parameters: ProtocolParameters,
}

impl TryFrom<proto::MilestoneAndProtocolParameters> for MilestoneAndProtocolParameters {
    type Error = bee::InxError;

    fn try_from(value: proto::MilestoneAndProtocolParameters) -> Result<Self, Self::Error> {
        Ok(Self {
            milestone: maybe_missing!(value.milestone).try_into()?,
            current_protocol_parameters: maybe_missing!(value.current_protocol_parameters).into(),
        })
    }
}

impl From<MilestoneAndProtocolParameters> for proto::MilestoneAndProtocolParameters {
    fn from(value: MilestoneAndProtocolParameters) -> Self {
        Self {
            milestone: Some(value.milestone.into()),
            current_protocol_parameters: Some(value.current_protocol_parameters.into()),
        }
    }
}
