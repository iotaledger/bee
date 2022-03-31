// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use serde::Serialize;

use crate::{
    websocket::{
        responses::{WsEvent, WsEventInner},
        topics::WsTopic,
    },
    workers::node_status::PublicNodeStatus,
};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PublicNodeStatusResponse(pub PublicNodeStatus);

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
