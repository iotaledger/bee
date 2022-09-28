// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod requests;
pub mod responses;

use futures::stream::{Stream, StreamExt};

pub use self::{requests::*, responses::*};
use crate::{
    block::responses::BlockWithMetadata,
    client::{try_from_inx_type, Inx},
    error::Error,
    inx,
};

//   rpc ReadMilestone(MilestoneRequest) returns (Milestone);
//   rpc ListenToLatestMilestones(NoParams) returns (stream Milestone);
//   rpc ListenToConfirmedMilestones(MilestoneRangeRequest) returns (stream MilestoneAndProtocolParameters);
//   rpc ComputeWhiteFlag(WhiteFlagRequest) returns (WhiteFlagResponse);
//   rpc ReadMilestoneCone(MilestoneRequest) returns (stream BlockWithMetadata);
//   rpc ReadMilestoneConeMetadata(MilestoneRequest) returns (stream BlockMetadata);

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
            .map(try_from_inx_type))
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
            .map(try_from_inx_type))
    }
}
