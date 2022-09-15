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
    pub subunit: String,
    pub decimals: u32,
    pub use_metric_prefix: bool,
}

impl From<proto::BaseToken> for BaseToken {
    fn from(value: proto::BaseToken) -> Self {
        Self {
            name: value.name,
            ticker_symbol: value.ticker_symbol,
            unit: value.unit,
            subunit: value.subunit,
            decimals: value.decimals,
            use_metric_prefix: value.use_metric_prefix,
        }
    }
}

impl From<BaseToken> for proto::BaseToken {
    fn from(value: BaseToken) -> Self {
        Self {
            name: value.name,
            ticker_symbol: value.ticker_symbol,
            unit: value.unit,
            subunit: value.subunit,
            decimals: value.decimals,
            use_metric_prefix: value.use_metric_prefix,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MilestoneKeyRange {
    pub public_key: Box<[u8]>,
    pub start_index: bee::payload::milestone::MilestoneIndex,
    pub end_index: bee::payload::milestone::MilestoneIndex,
}

impl From<proto::MilestoneKeyRange> for MilestoneKeyRange {
    fn from(value: proto::MilestoneKeyRange) -> Self {
        Self {
            public_key: value.public_key.into_boxed_slice(),
            start_index: value.start_index.into(),
            end_index: value.end_index.into(),
        }
    }
}

impl From<MilestoneKeyRange> for proto::MilestoneKeyRange {
    fn from(value: MilestoneKeyRange) -> Self {
        Self {
            public_key: value.public_key.into_vec(),
            start_index: value.start_index.0,
            end_index: value.end_index.0,
        }
    }
}

/// The [`NodeConfiguration`] type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeConfiguration {
    pub milestone_public_key_count: u32,
    pub milestone_key_ranges: Box<[MilestoneKeyRange]>,
    pub base_token: BaseToken,
    pub supported_protocol_versions: Box<[u8]>,
}

impl TryFrom<proto::NodeConfiguration> for NodeConfiguration {
    type Error = bee::InxError;

    fn try_from(value: proto::NodeConfiguration) -> Result<Self, Self::Error> {
        Ok(NodeConfiguration {
            milestone_public_key_count: value.milestone_public_key_count,
            milestone_key_ranges: value.milestone_key_ranges.into_iter().map(Into::into).collect(),
            base_token: maybe_missing!(value.base_token).into(),
            supported_protocol_versions: value.supported_protocol_versions.into_iter().map(|v| v as u8).collect(),
        })
    }
}

impl From<NodeConfiguration> for proto::NodeConfiguration {
    fn from(value: NodeConfiguration) -> Self {
        Self {
            milestone_public_key_count: value.milestone_public_key_count,
            milestone_key_ranges: value
                .milestone_key_ranges
                .into_vec()
                .into_iter()
                .map(Into::into)
                .collect(),
            base_token: Some(value.base_token.into()),
            supported_protocol_versions: value
                .supported_protocol_versions
                .into_vec()
                .into_iter()
                .map(|v| v as _)
                .collect(),
        }
    }
}
