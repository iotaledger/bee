// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;
use inx::proto;

use crate::field;

/// The [`NodeConfiguration`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeConfiguration {
    pub protocol_parameters: bee::protocol::ProtocolParameters,
    pub milestone_public_key_count: u32,
    // TODO: `milestone_key_ranges`
    // TODO: `base_token`
    pub supported_protocol_versions: Box<[u8]>,
    // TODO: `pending_protocol_parameters`
}

impl TryFrom<proto::NodeConfiguration> for NodeConfiguration {
    type Error = bee::InxError;

    fn try_from(value: proto::NodeConfiguration) -> Result<Self, Self::Error> {
        Ok(NodeConfiguration {
            protocol_parameters: field!(value.protocol_parameters).try_into()?,
            milestone_public_key_count: value.milestone_public_key_count,
            supported_protocol_versions: value.supported_protocol_versions.into_iter().map(|v| v as u8).collect(),
        })
    }
}
