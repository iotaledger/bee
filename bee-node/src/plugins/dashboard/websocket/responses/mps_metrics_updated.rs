// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::plugins::dashboard::websocket::{
    responses::{WsEvent, WsEventInner},
    topics::WsTopic,
};

use bee_protocol::workers::event::MpsMetricsUpdated;

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct MpsMetricsUpdatedResponse(pub MpsMetricsUpdatedDto);

#[derive(Clone, Debug, Serialize)]
pub(crate) struct MpsMetricsUpdatedDto {
    pub incoming: u64,
    pub new: u64,
    pub known: u64,
    pub invalid: u64,
    pub outgoing: u64,
}

impl From<MpsMetricsUpdated> for WsEvent {
    fn from(event: MpsMetricsUpdated) -> Self {
        Self::new(WsTopic::MpsMetrics, WsEventInner::MpsMetricsUpdated(event.into()))
    }
}

impl From<MpsMetricsUpdated> for MpsMetricsUpdatedResponse {
    fn from(event: MpsMetricsUpdated) -> Self {
        Self(event.into())
    }
}

impl From<MpsMetricsUpdated> for MpsMetricsUpdatedDto {
    fn from(event: MpsMetricsUpdated) -> Self {
        MpsMetricsUpdatedDto {
            incoming: event.incoming,
            new: event.new,
            known: event.known,
            invalid: event.invalid,
            outgoing: event.outgoing,
        }
    }
}
