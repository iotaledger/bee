// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;

use crate::{inx, return_err_if_none, ProtocolParameters, Raw};

/// The [`Milestone`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Milestone {
    /// Information about the milestone.
    pub milestone_info: MilestoneInfo,
    /// The raw bytes of the milestone. Note that this is not a [`bee::payload::milestone::MilestonePayload`], but
    /// rather a [`bee::payload::Payload`] and still needs to be unpacked.
    pub milestone: Raw<bee::payload::Payload>,
}

impl TryFrom<inx::Milestone> for Milestone {
    type Error = bee::InxError;

    fn try_from(value: inx::Milestone) -> Result<Self, Self::Error> {
        Ok(Self {
            milestone_info: return_err_if_none!(value.milestone_info).try_into()?,
            milestone: return_err_if_none!(value.milestone).data.into(),
        })
    }
}

impl From<Milestone> for inx::Milestone {
    fn from(value: Milestone) -> Self {
        Self {
            milestone_info: Some(value.milestone_info.into()),
            milestone: Some(value.milestone.into()),
        }
    }
}

/// The [`Milestone`] type.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneAndProtocolParameters {
    pub milestone: Milestone,
    pub current_protocol_parameters: ProtocolParameters,
}

impl TryFrom<inx::MilestoneAndProtocolParameters> for MilestoneAndProtocolParameters {
    type Error = bee::InxError;

    fn try_from(value: inx::MilestoneAndProtocolParameters) -> Result<Self, Self::Error> {
        Ok(Self {
            milestone: return_err_if_none!(value.milestone).try_into()?,
            current_protocol_parameters: return_err_if_none!(value.current_protocol_parameters).into(),
        })
    }
}

impl From<MilestoneAndProtocolParameters> for inx::MilestoneAndProtocolParameters {
    fn from(value: MilestoneAndProtocolParameters) -> Self {
        Self {
            milestone: Some(value.milestone.into()),
            current_protocol_parameters: Some(value.current_protocol_parameters.into()),
        }
    }
}

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

impl TryFrom<inx::MilestoneInfo> for MilestoneInfo {
    type Error = bee::InxError;

    fn try_from(value: inx::MilestoneInfo) -> Result<Self, Self::Error> {
        Ok(MilestoneInfo {
            milestone_id: value.milestone_id.map(TryInto::try_into).transpose()?,
            milestone_index: value.milestone_index.into(),
            milestone_timestamp: value.milestone_timestamp,
        })
    }
}

impl From<MilestoneInfo> for inx::MilestoneInfo {
    fn from(value: MilestoneInfo) -> Self {
        Self {
            milestone_id: value.milestone_id.map(Into::into),
            milestone_index: value.milestone_index.0,
            milestone_timestamp: value.milestone_timestamp,
        }
    }
}

/// The response of a corresponding "white flag" request.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhiteFlagResponse {
    milestone_inclusion_merkle_root: Vec<u8>,
    milestone_applied_merkle_root: Vec<u8>,
}

impl From<inx::WhiteFlagResponse> for WhiteFlagResponse {
    fn from(value: inx::WhiteFlagResponse) -> Self {
        Self {
            milestone_inclusion_merkle_root: value.milestone_inclusion_merkle_root,
            milestone_applied_merkle_root: value.milestone_applied_merkle_root,
        }
    }
}

impl From<WhiteFlagResponse> for inx::WhiteFlagResponse {
    fn from(value: WhiteFlagResponse) -> Self {
        Self {
            milestone_inclusion_merkle_root: value.milestone_inclusion_merkle_root,
            milestone_applied_merkle_root: value.milestone_applied_merkle_root,
        }
    }
}
