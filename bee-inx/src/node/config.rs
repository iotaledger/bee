// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block as bee;
use inx::proto;

use crate::maybe_missing;

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

/// The [`NodeConfiguration`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeConfiguration {
    pub milestone_public_key_count: u32,
    // TODO: `milestone_key_ranges`
    pub base_token: BaseToken,
    pub supported_protocol_versions: Box<[u8]>,
}

impl TryFrom<proto::NodeConfiguration> for NodeConfiguration {
    type Error = bee::InxError;

    fn try_from(value: proto::NodeConfiguration) -> Result<Self, Self::Error> {
        Ok(NodeConfiguration {
            milestone_public_key_count: value.milestone_public_key_count,
            base_token: maybe_missing!(value.base_token).into(),
            supported_protocol_versions: value.supported_protocol_versions.into_iter().map(|v| v as u8).collect(),
        })
    }
}
