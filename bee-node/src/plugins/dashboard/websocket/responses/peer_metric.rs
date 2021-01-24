// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

use crate::plugins::dashboard::workers::peer_metric::PeerMetric;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PeerMetricResponse(pub PeerMetric);

pub(crate) fn forward(event: PeerMetric) -> WsEvent {
    event.into()
}

impl From<PeerMetric> for WsEvent {
    fn from(val: PeerMetric) -> Self {
        Self::new(WsTopic::PeerMetrics, WsEventInner::PeerMetric(val.into()))
    }
}

impl From<PeerMetric> for PeerMetricResponse {
    fn from(val: PeerMetric) -> Self {
        Self(val)
    }
}
