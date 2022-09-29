// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::inx;

/// A request for the node status.
#[allow(missing_docs)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeStatusRequest {
    pub cooldown_in_milliseconds: u32,
}

impl From<inx::NodeStatusRequest> for NodeStatusRequest {
    fn from(value: inx::NodeStatusRequest) -> Self {
        Self {
            cooldown_in_milliseconds: value.cooldown_in_milliseconds,
        }
    }
}

impl From<NodeStatusRequest> for inx::NodeStatusRequest {
    fn from(value: NodeStatusRequest) -> Self {
        Self {
            cooldown_in_milliseconds: value.cooldown_in_milliseconds,
        }
    }
}
