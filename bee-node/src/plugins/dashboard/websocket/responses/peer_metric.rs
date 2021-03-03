// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

use bee_rest_api::endpoints::api::v1::peers::PeersResponse;

pub(crate) fn forward(event: PeersResponse) -> WsEvent {
    event.into()
}

impl From<PeersResponse> for WsEvent {
    fn from(val: PeersResponse) -> Self {
        Self::new(WsTopic::PeerMetrics, WsEventInner::PeerMetric(val))
    }
}
