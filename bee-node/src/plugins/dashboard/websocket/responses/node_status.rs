// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::{
    websocket::{
        responses::{WsEvent, WsEventInner},
        topics::WsTopic,
    },
    workers::node_status::NodeStatus,
};

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct NodeStatusResponse(pub NodeStatus);

impl From<NodeStatus> for WsEvent {
    fn from(val: NodeStatus) -> Self {
        Self::new(WsTopic::NodeStatus, WsEventInner::NodeStatus(val.into()))
    }
}

impl From<NodeStatus> for NodeStatusResponse {
    fn from(val: NodeStatus) -> Self {
        Self(val)
    }
}
