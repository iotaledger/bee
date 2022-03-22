// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Serialize;

use crate::{
    websocket::{
        responses::{WsEvent, WsEventInner},
        topics::WsTopic,
    },
    workers::node_status::NodeStatus,
};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct NodeStatusResponse(pub NodeStatus);

impl From<NodeStatus> for WsEvent {
    fn from(val: NodeStatus) -> Self {
        Self::new(WsTopic::NodeStatus, WsEventInner::NodeStatus(Box::new(val.into())))
    }
}

impl From<NodeStatus> for NodeStatusResponse {
    fn from(val: NodeStatus) -> Self {
        Self(val)
    }
}
