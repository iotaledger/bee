// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee::payload::milestone::MilestoneIndex;
use bee_block as bee;
use inx::proto;

use crate::{maybe_missing, Raw};

/// The [`BaseToken`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseToken {
    pub name: String,
    pub ticker_symbol: String,
    pub unit: String,
    pub decimals: u32,
    pub use_metric_prefix: bool,
}

impl From<proto::BaseToken> for BaseToken {
    fn from(value: proto::BaseToken) -> Self {
        Self {
            name: value.name,
            ticker_symbol: value.ticker_symbol,
            unit: value.unit,
            decimals: value.decimals,
            use_metric_prefix: value.use_metric_prefix,
        }
    }
}

/// The [`PendingProtocolParameters`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingProtocolParameters {
    pub target_milestone_index: MilestoneIndex,
    pub version: u8,
    pub params: Raw<bee::protocol::ProtocolParameters>,
}

impl From<proto::PendingProtocolParameters> for PendingProtocolParameters {
    fn from(value: proto::PendingProtocolParameters) -> Self {
        PendingProtocolParameters {
            target_milestone_index: value.target_milestone_index.into(),
            version: value.version as u8,
            params: value.params.into(),
        }
    }
}

/// The [`NodeConfiguration`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeConfiguration {
    pub protocol_parameters: bee::protocol::ProtocolParameters,
    pub milestone_public_key_count: u32,
    // TODO: `milestone_key_ranges`
    pub base_token: BaseToken,
    pub supported_protocol_versions: Box<[u8]>,
    pub pending_protocol_parameters: Box<[PendingProtocolParameters]>,
}

impl TryFrom<proto::NodeConfiguration> for NodeConfiguration {
    type Error = bee::InxError;

    fn try_from(value: proto::NodeConfiguration) -> Result<Self, Self::Error> {
        let pending_protocol_parameters = value
            .pending_protocol_parameters
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();

        Ok(NodeConfiguration {
            protocol_parameters: maybe_missing!(value.protocol_parameters).try_into()?,
            milestone_public_key_count: value.milestone_public_key_count,
            base_token: maybe_missing!(value.base_token).into(),
            supported_protocol_versions: value.supported_protocol_versions.into_iter().map(|v| v as u8).collect(),
            pending_protocol_parameters: pending_protocol_parameters.into_boxed_slice(),
        })
    }
}
