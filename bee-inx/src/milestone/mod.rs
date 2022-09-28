// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod info;

use bee_block as bee;
use futures::stream::{Stream, StreamExt};

pub use self::info::MilestoneInfo;
use crate::{
    block::BlockWithMetadata,
    client::{try_convert_proto_msg, Inx},
    error::Error,
    inx,
    request::{MilestoneRangeRequest, MilestoneRequest},
    return_err_if_none, ProtocolParameters, Raw,
};

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

impl Inx {
    /// TODO
    pub async fn read_milestone(&mut self, request: MilestoneRequest) -> Result<Milestone, Error> {
        Milestone::try_from(
            self.client
                .read_milestone(inx::MilestoneRequest::from(request))
                .await?
                .into_inner(),
        )
        .map_err(Error::InxError)
    }

    /// Listens to confirmed milestones in a certain range.
    pub async fn listen_to_confirmed_milestones(
        &mut self,
        request: MilestoneRangeRequest,
    ) -> Result<impl Stream<Item = Result<MilestoneAndProtocolParameters, Error>>, Error> {
        Ok(self
            .client
            .listen_to_confirmed_milestones(inx::MilestoneRangeRequest::from(request))
            .await?
            .into_inner()
            .map(try_convert_proto_msg))
    }

    /// Reads the past cone of a milestone specified by a [`MilestoneRequest`].
    pub async fn read_milestone_cone(
        &mut self,
        request: MilestoneRequest,
    ) -> Result<impl Stream<Item = Result<BlockWithMetadata, Error>>, Error> {
        Ok(self
            .client
            .read_milestone_cone(inx::MilestoneRequest::from(request))
            .await?
            .into_inner()
            .map(try_convert_proto_msg))
    }
}
