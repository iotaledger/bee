// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod types;

pub use self::types::*;
use crate::{client::Inx, error::Error, inx, request::MilestoneRequest};

impl Inx {
    /// TODO
    pub async fn read_node_status(&mut self) -> Result<NodeStatus, Error> {
        NodeStatus::try_from(self.client.read_node_status(inx::NoParams {}).await?.into_inner())
            .map_err(Error::InxError)
    }

    // TODO: listen_to_node_status

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
