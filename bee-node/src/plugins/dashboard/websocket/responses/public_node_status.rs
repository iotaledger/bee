// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::{
    websocket::{
        responses::{WsEvent, WsEventInner},
        topics::WsTopic,
    },
    workers::node_status::PublicNodeStatus,
};

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PublicNodeStatusResponse(pub PublicNodeStatus);

pub(crate) fn forward(event: PublicNodeStatus) -> WsEvent {
    event.into()
}

impl From<PublicNodeStatus> for WsEvent {
    fn from(val: PublicNodeStatus) -> Self {
        Self::new(WsTopic::PublicNodeStatus, WsEventInner::PublicNodeStatus(val.into()))
    }
}

impl From<PublicNodeStatus> for PublicNodeStatusResponse {
    fn from(val: PublicNodeStatus) -> Self {
        Self(val)
    }
}
