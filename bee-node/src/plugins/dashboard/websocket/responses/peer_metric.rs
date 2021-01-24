// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::{
    websocket::{
        responses::{WsEvent, WsEventInner},
        topics::WsTopic,
    },
    workers::peer_metric::PeerMetrics,
};

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PeerMetricResponse(pub PeerMetrics);

pub(crate) fn forward(event: PeerMetrics) -> WsEvent {
    event.into()
}

impl From<PeerMetrics> for WsEvent {
    fn from(val: PeerMetrics) -> Self {
        Self::new(WsTopic::PeerMetrics, WsEventInner::PeerMetric(val.into()))
    }
}

impl From<PeerMetrics> for PeerMetricResponse {
    fn from(val: PeerMetrics) -> Self {
        Self(val)
    }
}
