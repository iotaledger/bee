// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod requests;
pub mod responses;

use futures::stream::{Stream, StreamExt};

pub use self::{requests::*, responses::*};
use crate::{
    block::responses::{BlockMetadata, BlockWithMetadata},
    client::{try_from_inx_type, Inx},
    error::Error,
    inx,
};

impl Inx {
    /// TODO
    pub async fn read_milestone(&mut self, request: MilestoneRequest) -> Result<Milestone, Error> {
        Ok(self
            .client
            .read_milestone(inx::MilestoneRequest::from(request))
            .await?
            .into_inner()
            .try_into()?)
    }

    /// Listens to latest milestones.
    pub async fn listen_to_latest_milestones(&mut self) -> Result<impl Stream<Item = Result<Milestone, Error>>, Error> {
        Ok(self
            .client
            .listen_to_latest_milestones(inx::NoParams {})
            .await?
            .into_inner()
            .map(try_from_inx_type))
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

    /// TODO
    pub async fn compute_white_flag(&mut self, request: WhiteFlagRequest) -> Result<WhiteFlagResponse, Error> {
        Ok(self
            .client
            .compute_white_flag(inx::WhiteFlagRequest::from(request))
            .await?
            .into_inner()
            .into())
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

    /// TODO
    pub async fn read_milestone_cone_metadata(
        &mut self,
        request: MilestoneRequest,
    ) -> Result<impl Stream<Item = Result<BlockMetadata, Error>>, Error> {
        Ok(self
            .client
            .read_milestone_cone_metadata(inx::MilestoneRequest::from(request))
            .await?
            .into_inner()
            .map(try_from_inx_type))
    }
}
