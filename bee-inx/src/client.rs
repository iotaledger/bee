// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::ops::RangeBounds;

use bee_block::payload::milestone::{MilestoneId, MilestoneIndex};
use futures::stream::{Stream, StreamExt};
use inx::{proto, proto::{inx_client::InxClient}, tonic};

use crate::{Error, NodeStatus, NodeConfiguration};

pub struct Inx {
    inx: InxClient<inx::tonic::Channel>,
}

fn unpack_milestone_msg(msg: Result<proto::Milestone, tonic::Status>) -> Result<crate::Milestone, Error> {
    let inner = msg.map_err(Error::StatusCode)?;
    crate::Milestone::try_from(inner).map_err(Into::into)
}

fn unpack_unspent_output_msg(msg: Result<proto::UnspentOutput, tonic::Status>) -> Result<crate::UnspentOutput, Error> {
    let inner = msg.map_err(Error::StatusCode)?;
    crate::UnspentOutput::try_from(inner).map_err(Into::into)
}

impl Inx {
    pub async fn connect(address: String) -> Result<Self, Error> {
        Ok(Self {
            inx: InxClient::connect(address).await?,
        })
    }

    pub async fn listen_to_confirmed_milestones(
        &mut self,
        range: impl RangeBounds<u32>,
    ) -> Result<impl Stream<Item = Result<crate::Milestone, Error>>, Error> {
        let request = crate::to_milestone_range_request(range);
        Ok(self
            .inx
            .listen_to_confirmed_milestones(request)
            .await?
            .into_inner()
            .map(unpack_milestone_msg))
    }

    pub async fn read_node_status(
        &mut self,
    ) -> Result<NodeStatus, Error> {
        NodeStatus::try_from(self.inx.read_node_status(proto::NoParams{}).await?.into_inner()).map_err(Error::InxError)
    }

    pub async fn read_node_configuration(
        &mut self,
    ) -> Result<NodeConfiguration, Error> {
        NodeConfiguration::try_from(self.inx.read_node_configuration(proto::NoParams{}).await?.into_inner()).map_err(Error::InxError)
    }

    pub async fn read_unspent_outputs(&mut self) -> Result<impl Stream<Item = Result<crate::UnspentOutput, Error>>, Error> {
        Ok(self.inx.read_unspent_outputs(proto::NoParams{}).await?.into_inner().map(unpack_unspent_output_msg))
    }

}

pub enum MilestoneRequest {
    MilestoneIndex(MilestoneIndex),
    MilestoneId(MilestoneId),
}

impl From<MilestoneRequest> for proto::MilestoneRequest {
    fn from(value: MilestoneRequest) -> Self {
        match value {
            MilestoneRequest::MilestoneIndex(MilestoneIndex(milestone_index)) => Self {
                milestone_index,
                milestone_id: None,
            },
            MilestoneRequest::MilestoneId(milestone_id) => Self {
                milestone_index: 0,
                milestone_id: Some(milestone_id.into()),
            },
        }
    }
}
