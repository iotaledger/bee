// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_api_types::responses::PeersResponse;

use crate::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

impl From<PeersResponse> for WsEvent {
    fn from(val: PeersResponse) -> Self {
        Self::new(WsTopic::PeerMetrics, WsEventInner::PeerMetric(val))
    }
}
