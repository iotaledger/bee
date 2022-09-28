// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod requests;
pub mod types;

use futures::stream::{Stream, StreamExt};

pub use self::{requests::*, types::*};
use crate::{
    client::{try_from_inx_type, Inx},
    error::Error,
    inx,
    milestone::requests::MilestoneRequest,
};

impl Inx {
    /// TODO
    pub async fn read_node_status(&mut self) -> Result<NodeStatus, Error> {
        NodeStatus::try_from(self.client.read_node_status(inx::NoParams {}).await?.into_inner())
            .map_err(Error::InxError)
    }

    // TODO
    pub async fn listen_to_node_status(
        &mut self,
        request: NodeStatusRequest,
    ) -> Result<impl Stream<Item = Result<NodeStatus, Error>>, Error> {
        Ok(self
            .client
            .listen_to_node_status(inx::NodeStatusRequest::from(request))
            .await?
            .into_inner()
            .map(try_from_inx_type))
    }

    /// TODO
    pub async fn read_node_configuration(&mut self) -> Result<NodeConfiguration, Error> {
        NodeConfiguration::try_from(
            self.client
                .read_node_configuration(inx::NoParams {})
                .await?
                .into_inner(),
        )
        .map_err(Error::InxError)
    }

    /// TODO
    pub async fn read_protocol_parameters(&mut self, request: MilestoneRequest) -> Result<ProtocolParameters, Error> {
        Ok(self
            .client
            .read_protocol_parameters(inx::MilestoneRequest::from(request))
            .await?
            .into_inner()
            .into())
    }
}
